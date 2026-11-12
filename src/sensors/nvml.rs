//! Reads NVIDIA GPUs through NVML, the management library that ships with the NVIDIA driver and that `nvidia-smi`
//! uses. The driver doesn't put the usage or VRAM in sysfs, as `amdgpu` does.
//!
//! The library is loaded when an NVIDIA GPU is first read, rather than linked, so the applet runs without the NVIDIA
//! driver installed, and costs nothing on systems without an NVIDIA GPU. Only the few functions the applet uses are
//! declared, from `nvml.h`.
//!
//! Initialising NVML opens every NVIDIA GPU, which wakes any that are asleep, and NVML holds on to them until it's
//! shut down, which can stop them from sleeping. So an [`Nvml`] is only created while the NVIDIA GPUs are awake, only
//! kept while one is being read, and shuts NVML down when it's dropped. (NVML has a flag to initialise without
//! opening the GPUs, but then it can't find any of them.)

use crate::sensors::gpu_reader::GpuReading;
use libc::{c_char, c_int, c_uint, c_void};
use log::{info, warn};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::OnceLock;

/// The library's soname, then its development symlink, which some distributions package without the soname.
const LIBRARY_NAMES: [&CStr; 2] = [c"libnvidia-ml.so.1", c"libnvidia-ml.so"];

const NVML_SUCCESS: c_int = 0;
const NVML_TEMPERATURE_GPU: c_uint = 0;

type DeviceHandle = *mut c_void;

#[repr(C)]
#[derive(Default)]
struct Utilization {
    /// The percentage of the last sample period that a kernel was running on the GPU.
    gpu: c_uint,
    memory: c_uint,
}

#[repr(C)]
#[derive(Default)]
struct Memory {
    total: u64,
    free: u64,
    used: u64,
}

/// The functions used from the library, which is never unloaded.
struct Library {
    init: unsafe extern "C" fn() -> c_int,
    shutdown: unsafe extern "C" fn() -> c_int,
    error_string: unsafe extern "C" fn(c_int) -> *const c_char,
    handle_by_pci_bus_id: unsafe extern "C" fn(*const c_char, *mut DeviceHandle) -> c_int,
    utilization_rates: unsafe extern "C" fn(DeviceHandle, *mut Utilization) -> c_int,
    memory_info: unsafe extern "C" fn(DeviceHandle, *mut Memory) -> c_int,
    temperature: unsafe extern "C" fn(DeviceHandle, c_uint, *mut c_uint) -> c_int,
    power_usage: unsafe extern "C" fn(DeviceHandle, *mut c_uint) -> c_int,
}

static LIBRARY: OnceLock<Option<Library>> = OnceLock::new();

impl Library {
    /// The library, loaded the first time it's needed. `None` when it isn't installed.
    fn get() -> Option<&'static Library> {
        LIBRARY.get_or_init(Library::load).as_ref()
    }

    fn load() -> Option<Library> {
        // SAFETY: dlopen is given NUL terminated names. The library is never closed, so the functions looked up in it
        // stay valid for the life of the process.
        let handle = LIBRARY_NAMES.iter().find_map(|name| {
            let handle = unsafe { libc::dlopen(name.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
            (!handle.is_null()).then_some(handle)
        });
        let Some(handle) = handle else {
            info!("NVML (libnvidia-ml.so.1) isn't installed, so NVIDIA GPUs can't be read");
            return None;
        };

        /// Look up a function. `T` must be the function's signature, as a function pointer type.
        unsafe fn symbol<T: Copy>(handle: *mut c_void, name: &CStr) -> Option<T> {
            let pointer = unsafe { libc::dlsym(handle, name.as_ptr()) };
            // SAFETY: function pointers are the size of a data pointer on the platforms dlsym exists on.
            (!pointer.is_null()).then(|| unsafe { std::mem::transmute_copy::<*mut c_void, T>(&pointer) })
        }

        // SAFETY: each type is the signature of the function in nvml.h.
        let library = unsafe {
            Some(Library {
                init: symbol(handle, c"nvmlInit_v2")?,
                shutdown: symbol(handle, c"nvmlShutdown")?,
                error_string: symbol(handle, c"nvmlErrorString")?,
                handle_by_pci_bus_id: symbol(handle, c"nvmlDeviceGetHandleByPciBusId_v2")?,
                utilization_rates: symbol(handle, c"nvmlDeviceGetUtilizationRates")?,
                memory_info: symbol(handle, c"nvmlDeviceGetMemoryInfo")?,
                temperature: symbol(handle, c"nvmlDeviceGetTemperature")?,
                power_usage: symbol(handle, c"nvmlDeviceGetPowerUsage")?,
            })
        };

        if library.is_none() {
            warn!("NVML is missing functions the applet uses, so NVIDIA GPUs can't be read");
        }
        library
    }

    fn check(&self, result: c_int, call: &str) -> Result<(), String> {
        if result == NVML_SUCCESS {
            return Ok(());
        }
        // SAFETY: NVML returns a static string for every result.
        let message = unsafe { CStr::from_ptr((self.error_string)(result)) };
        Err(format!("{call} failed: {}", message.to_string_lossy()))
    }
}

/// A GPU's handle, which stays valid until NVML is shut down.
struct Device(DeviceHandle);

// SAFETY: NVML is thread safe, and a handle is only an id for a GPU that NVML owns.
unsafe impl Send for Device {}

/// NVML, initialised. Dropping it shuts NVML down, which lets go of the GPUs it opened.
pub struct Nvml {
    library: &'static Library,
    /// The GPUs opened so far, by PCI address.
    devices: HashMap<String, Device>,
}

impl Nvml {
    /// Initialise NVML, which opens every NVIDIA GPU, waking any that are asleep.
    pub fn init() -> Result<Self, String> {
        let library = Library::get().ok_or_else(|| "NVML (libnvidia-ml.so.1) isn't installed".to_string())?;

        // SAFETY: takes no arguments.
        let result = unsafe { (library.init)() };
        library.check(result, "nvmlInit")?;
        info!("Initialised NVML");

        Ok(Nvml {
            library,
            devices: HashMap::new(),
        })
    }

    /// Read the GPU at `pci_address`, e.g. `0000:01:00.0`, opening it the first time. This wakes it if it's asleep.
    pub fn read(&mut self, pci_address: &str) -> Result<GpuReading, String> {
        let library = self.library;
        let device = self.device(pci_address)?;

        let mut utilization = Utilization::default();
        let mut memory = Memory::default();
        let mut temperature: c_uint = 0;
        let mut milliwatts: c_uint = 0;

        // SAFETY: each is given a valid handle and pointers to values of the types nvml.h declares. A GPU doesn't
        // support every reading, so each one's failure only leaves that reading out.
        let (usage_ok, memory_ok, temperature_ok, power_ok) = unsafe {
            (
                (library.utilization_rates)(device, &mut utilization) == NVML_SUCCESS,
                (library.memory_info)(device, &mut memory) == NVML_SUCCESS,
                (library.temperature)(device, NVML_TEMPERATURE_GPU, &mut temperature) == NVML_SUCCESS,
                (library.power_usage)(device, &mut milliwatts) == NVML_SUCCESS,
            )
        };

        if !usage_ok && !memory_ok {
            return Err(format!("NVML couldn't read the GPU at {pci_address}"));
        }

        Ok(GpuReading {
            usage_percent: usage_ok.then(|| f64::from(utilization.gpu.min(100))),
            vram_used_bytes: memory_ok.then_some(memory.used),
            vram_total_bytes: memory_ok.then_some(memory.total),
            temperature_celsius: temperature_ok.then(|| f64::from(temperature)),
            power_watts: power_ok.then(|| f64::from(milliwatts) / 1000.0),
        })
    }

    fn device(&mut self, pci_address: &str) -> Result<DeviceHandle, String> {
        if let Some(device) = self.devices.get(pci_address) {
            return Ok(device.0);
        }

        let bus_id = CString::new(pci_address).map_err(|_| format!("Invalid PCI address {pci_address:?}"))?;
        let mut handle: DeviceHandle = std::ptr::null_mut();
        // SAFETY: given a NUL terminated bus id and a pointer to a handle to fill in.
        let result = unsafe { (self.library.handle_by_pci_bus_id)(bus_id.as_ptr(), &mut handle) };
        self.library.check(result, "nvmlDeviceGetHandleByPciBusId")?;

        self.devices.insert(pci_address.to_string(), Device(handle));
        Ok(handle)
    }
}

impl Drop for Nvml {
    fn drop(&mut self) {
        // SAFETY: NVML was initialised by this value, and the handles are dropped with it.
        let result = unsafe { (self.library.shutdown)() };
        match self.library.check(result, "nvmlShutdown") {
            Ok(()) => info!("Shut down NVML"),
            Err(err) => warn!("{err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::gpu_reader::{discover_gpus, GpuDriver, GpuPaths};

    /// Reads the NVIDIA GPUs of the machine the tests run on. Run with `cargo test -- --ignored` on a machine with
    /// one.
    #[test]
    #[ignore]
    fn reads_the_nvidia_gpus_of_this_machine() {
        let gpus: Vec<_> = discover_gpus(&GpuPaths::system())
            .into_iter()
            .filter(|gpu| gpu.driver == GpuDriver::Nvidia)
            .collect();
        assert!(!gpus.is_empty(), "This machine has no NVIDIA GPU");

        let mut nvml = Nvml::init().expect("NVML should initialise");
        for gpu in gpus {
            let reading = nvml.read(&gpu.pci_address).expect("The GPU should be read");
            println!("{}: {reading:?}", gpu.name);
            assert!(reading.vram_total_bytes.is_some_and(|total| total > 0));
            assert!(reading.usage_percent.is_some());
        }
    }
}
