//! Finds the GPUs the applet can read, through the DRM cards in sysfs, and reads AMD GPUs from the files the `amdgpu`
//! driver provides. NVIDIA GPUs are read with NVML instead, see [`crate::sensors::nvml`].
//!
//! Each `/sys/class/drm/cardN` links to its PCI device. The `cardN` numbers depend on the order the drivers load in,
//! so a GPU is identified by its PCI address, e.g. `0000:01:00.0`, instead.
//!
//! A GPU that the kernel has put to sleep (runtime suspended) is never read, as reading it wakes it. Whether it's
//! asleep is read from its PCI device's `power/runtime_status`, which doesn't touch the GPU.

use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Where the kernel lists the DRM cards.
pub const DRM_ROOT: &str = "/sys/class/drm";
/// Where the NVIDIA driver reports each GPU's runtime power management (RTD3) status.
pub const NVIDIA_PROC_ROOT: &str = "/proc/driver/nvidia/gpus";
/// The PCI ID database, which names the GPUs. Distributions keep it in one of these.
pub const PCI_IDS_PATHS: [&str; 2] = ["/usr/share/hwdata/pci.ids", "/usr/share/misc/pci.ids"];

/// The drivers whose GPUs can be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuDriver {
    Amdgpu,
    Nvidia,
}

impl GpuDriver {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "amdgpu" => Some(GpuDriver::Amdgpu),
            "nvidia" => Some(GpuDriver::Nvidia),
            _ => None,
        }
    }

    fn vendor(self) -> &'static str {
        match self {
            GpuDriver::Amdgpu => "AMD",
            GpuDriver::Nvidia => "NVIDIA",
        }
    }
}

/// Where to find the GPUs, which the tests point at fake trees.
#[derive(Debug, Clone)]
pub struct GpuPaths {
    pub drm_root: PathBuf,
    pub nvidia_proc_root: PathBuf,
    pub pci_ids: Vec<PathBuf>,
}

impl GpuPaths {
    pub fn system() -> Self {
        GpuPaths {
            drm_root: PathBuf::from(DRM_ROOT),
            nvidia_proc_root: PathBuf::from(NVIDIA_PROC_ROOT),
            pci_ids: PCI_IDS_PATHS.iter().map(PathBuf::from).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpuDevice {
    /// The PCI address, e.g. `0000:01:00.0`, which identifies the GPU across reboots.
    pub pci_address: String,
    pub driver: GpuDriver,
    /// The vendor and model, e.g. `NVIDIA GeForce RTX 4060 Ti`.
    pub name: String,
    /// The PCI device's directory in sysfs.
    pub device_path: PathBuf,
    /// The `amdgpu` hwmon chip of an AMD GPU, with its temperature and power.
    pub hwmon_path: Option<PathBuf>,
    /// Whether the kernel can put the GPU to sleep when it's idle, e.g. the discrete GPU of a laptop.
    pub can_sleep: bool,
    /// How long the GPU has to be idle before the kernel puts it to sleep, when the driver says.
    pub autosuspend_delay: Option<Duration>,
    /// The VRAM of an AMD GPU, which tells a discrete GPU from an integrated one's small carve-out. Reading it doesn't
    /// wake the GPU.
    pub vram_total_bytes: Option<u64>,
}

/// What was read from a GPU. Each is `None` when the GPU or its driver doesn't report it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GpuReading {
    pub usage_percent: Option<f64>,
    pub vram_used_bytes: Option<u64>,
    pub vram_total_bytes: Option<u64>,
    pub temperature_celsius: Option<f64>,
    pub power_watts: Option<f64>,
}

/// Every GPU with a supported driver, sorted by PCI address so they're listed the same way on every boot. Finding
/// them doesn't wake any that are asleep.
pub fn discover_gpus(paths: &GpuPaths) -> Vec<GpuDevice> {
    let Ok(entries) = fs::read_dir(&paths.drm_root) else {
        return Vec::new();
    };

    let mut gpus: Vec<GpuDevice> = Vec::new();

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        // The connectors, e.g. `card0-DP-1`, and the render nodes link to the same devices.
        let is_card = file_name
            .to_str()
            .and_then(|name| name.strip_prefix("card"))
            .is_some_and(|number| !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()));
        if !is_card {
            continue;
        }

        let Ok(device_path) = fs::canonicalize(entry.path().join("device")) else {
            continue;
        };
        let Some(pci_address) = device_path.file_name().and_then(|name| name.to_str()).map(str::to_string) else {
            continue;
        };
        if gpus.iter().any(|gpu| gpu.pci_address == pci_address) {
            continue;
        }

        let Some(driver) = fs::read_link(device_path.join("driver"))
            .ok()
            .and_then(|driver| driver.file_name().and_then(|name| name.to_str()).and_then(GpuDriver::from_name))
        else {
            continue;
        };

        gpus.push(read_device(paths, driver, pci_address, device_path));
    }

    gpus.sort_by(|a, b| a.pci_address.cmp(&b.pci_address));
    gpus
}

fn read_device(paths: &GpuPaths, driver: GpuDriver, pci_address: String, device_path: PathBuf) -> GpuDevice {
    let model = read_trimmed(&device_path.join("product_name")).or_else(|| {
        let vendor_id = read_hex_id(&device_path.join("vendor"))?;
        let device_id = read_hex_id(&device_path.join("device"))?;
        pci_device_name(&paths.pci_ids, vendor_id, device_id)
    });
    let name = format!("{} {}", driver.vendor(), model.as_deref().unwrap_or("GPU"));

    // Runtime power management only puts the GPU to sleep when it's set to `auto`.
    let mut can_sleep = read_trimmed(&device_path.join("power/control")).as_deref() == Some("auto");
    // The NVIDIA driver also has to support it, which only it reports. Desktop GPUs are usually set to `auto` but
    // never sleep.
    if driver == GpuDriver::Nvidia {
        if let Some(status) = nvidia_runtime_d3_status(&paths.nvidia_proc_root, &pci_address) {
            can_sleep &= status.starts_with("Enabled");
        }
    }

    // Fails to read when the driver doesn't use an autosuspend delay.
    let autosuspend_delay = read_trimmed(&device_path.join("power/autosuspend_delay_ms"))
        .and_then(|delay| delay.parse::<u64>().ok())
        .map(Duration::from_millis);

    let (hwmon_path, vram_total_bytes) = match driver {
        GpuDriver::Amdgpu => (
            find_hwmon(&device_path),
            read_integer(&device_path.join("mem_info_vram_total")),
        ),
        GpuDriver::Nvidia => (None, None),
    };

    GpuDevice {
        pci_address,
        driver,
        name,
        device_path,
        hwmon_path,
        can_sleep,
        autosuspend_delay,
        vram_total_bytes,
    }
}

/// The `Runtime D3 status` the NVIDIA driver reports for the GPU, e.g. `Enabled (fine-grained)` or `Disabled by
/// default`.
fn nvidia_runtime_d3_status(nvidia_proc_root: &Path, pci_address: &str) -> Option<String> {
    let power = fs::read_to_string(nvidia_proc_root.join(pci_address).join("power")).ok()?;
    power.lines().find_map(|line| {
        line.trim()
            .strip_prefix("Runtime D3 status:")
            .map(|status| status.trim().to_string())
    })
}

fn find_hwmon(device_path: &Path) -> Option<PathBuf> {
    let mut chips: Vec<PathBuf> = fs::read_dir(device_path.join("hwmon"))
        .ok()?
        .flatten()
        .filter(|entry| entry.file_name().to_str().is_some_and(|name| name.starts_with("hwmon")))
        .map(|entry| entry.path())
        .collect();
    chips.sort();
    chips.into_iter().next()
}

/// Whether the GPU is awake, so it can be read without waking it. A device without runtime power management is
/// always awake.
pub fn is_awake(gpu: &GpuDevice) -> bool {
    match read_trimmed(&gpu.device_path.join("power/runtime_status")) {
        // `suspending` and `resuming` are on their way to or from sleep, which reading would get in the way of.
        Some(status) => matches!(status.as_str(), "active" | "unsupported"),
        None => true,
    }
}

/// Read an AMD GPU. This wakes it if it's asleep, so check [`is_awake`] first.
pub fn read_amd(gpu: &GpuDevice) -> GpuReading {
    let device = &gpu.device_path;
    let hwmon = gpu.hwmon_path.as_deref();

    GpuReading {
        usage_percent: read_integer(&device.join("gpu_busy_percent")).map(|percent| percent.min(100) as f64),
        vram_used_bytes: read_integer(&device.join("mem_info_vram_used")),
        vram_total_bytes: read_integer(&device.join("mem_info_vram_total")),
        // The first is the `edge` temperature, which is what the GPU's own tools show.
        temperature_celsius: hwmon
            .and_then(|hwmon| read_integer(&hwmon.join("temp1_input")))
            .map(|millidegrees| millidegrees as f64 / 1000.0),
        power_watts: hwmon.and_then(read_amd_power),
    }
}

/// The board power in watts. An integrated GPU reports the power of the whole processor package instead, which isn't
/// the GPU's, so it's only read from GPUs with a power limit, which only discrete GPUs have.
fn read_amd_power(hwmon: &Path) -> Option<f64> {
    if !hwmon.join("power1_cap").exists() {
        return None;
    }
    read_integer(&hwmon.join("power1_average"))
        .or_else(|| read_integer(&hwmon.join("power1_input")))
        .map(|microwatts| microwatts as f64 / 1_000_000.0)
}

/// The model name of a PCI device from the PCI ID database, e.g. `GeForce RTX 4060 Ti` for an `AD106 [GeForce RTX
/// 4060 Ti]` entry. The first of `paths` that exists is used.
pub fn pci_device_name(paths: &[PathBuf], vendor_id: u16, device_id: u16) -> Option<String> {
    let file = paths.iter().find_map(|path| File::open(path).ok())?;
    find_pci_device_name(BufReader::new(file), vendor_id, device_id)
}

/// The database lists each vendor, `1002  Advanced Micro Devices`, followed by its devices indented by a tab,
/// `\t164e  Raphael`, and their subsystems indented by two.
fn find_pci_device_name(reader: impl BufRead, vendor_id: u16, device_id: u16) -> Option<String> {
    let vendor_prefix = format!("{vendor_id:04x} ");
    let device_prefix = format!("\t{device_id:04x} ");
    let mut in_vendor = false;

    for line in reader.split(b'\n') {
        let line = line.ok()?;
        let line = String::from_utf8_lossy(&line);
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        if !line.starts_with('\t') {
            // The vendors are listed once each, so the device isn't listed once its vendor's entries end.
            if in_vendor {
                return None;
            }
            in_vendor = line.starts_with(&vendor_prefix);
        } else if in_vendor {
            if let Some(name) = line.strip_prefix(&device_prefix) {
                return Some(model_name(name.trim()));
            }
        }
    }

    None
}

/// The marketing name, which the database gives in brackets after the chip's name, when it has one.
fn model_name(name: &str) -> String {
    name.split_once('[')
        .and_then(|(_, rest)| rest.rsplit_once(']'))
        .map(|(model, _)| model.trim())
        .filter(|model| !model.is_empty())
        .unwrap_or(name)
        .to_string()
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_integer(path: &Path) -> Option<u64> {
    read_trimmed(path)?.parse().ok()
}

/// A PCI vendor or device id, which sysfs gives as e.g. `0x10de`.
fn read_hex_id(path: &Path) -> Option<u16> {
    let value = read_trimmed(path)?;
    u16::from_str_radix(value.trim_start_matches("0x"), 16).ok()
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    const PCI_IDS: &str = "\
# The PCI ID database
1002  Advanced Micro Devices, Inc. [AMD/ATI]
\t164e  Raphael
\t\t1043 8877  Raphael (ASUS)
\t744c  Navi 31 [Radeon RX 7900 XT/7900 XTX/7900 GRE/7900M]
1022  Advanced Micro Devices, Inc. [AMD]
\t164e  Not a GPU
10de  NVIDIA Corporation
\t2803  AD106 [GeForce RTX 4060 Ti]
C 03  Display controller
";

    /// A fake sysfs, with DRM cards linking to PCI devices that link to their drivers, as the real one does.
    pub struct FakeGpus {
        pub root: tempfile::TempDir,
    }

    impl FakeGpus {
        pub fn new() -> Self {
            let fake = FakeGpus {
                root: tempfile::tempdir().unwrap(),
            };
            fs::create_dir_all(fake.root.path().join("drm")).unwrap();
            fs::create_dir_all(fake.root.path().join("nvidia")).unwrap();
            fs::write(fake.root.path().join("pci.ids"), PCI_IDS).unwrap();
            fake
        }

        pub fn paths(&self) -> GpuPaths {
            GpuPaths {
                drm_root: self.root.path().join("drm"),
                nvidia_proc_root: self.root.path().join("nvidia"),
                pci_ids: vec![self.root.path().join("missing.ids"), self.root.path().join("pci.ids")],
            }
        }

        /// Add a GPU as `card`, with `files` in its PCI device's directory. Returns the device's directory.
        pub fn gpu(&self, card: &str, pci_address: &str, driver: &str, files: &[(&str, &str)]) -> PathBuf {
            let device = self.root.path().join("devices").join(pci_address);
            fs::create_dir_all(device.join("power")).unwrap();
            let driver_path = self.root.path().join("drivers").join(driver);
            fs::create_dir_all(&driver_path).unwrap();
            symlink(&driver_path, device.join("driver")).unwrap();

            for (file, value) in files {
                let path = device.join(file);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, format!("{value}\n")).unwrap();
            }

            let card_path = self.root.path().join("drm").join(card);
            fs::create_dir_all(&card_path).unwrap();
            symlink(&device, card_path.join("device")).unwrap();
            device
        }
    }

    fn amd_files() -> Vec<(&'static str, &'static str)> {
        vec![
            ("vendor", "0x1002"),
            ("device", "0x744c"),
            ("power/control", "auto"),
            ("power/runtime_status", "active"),
            ("power/autosuspend_delay_ms", "5000"),
            ("gpu_busy_percent", "37"),
            ("mem_info_vram_used", "3221225472"),
            ("mem_info_vram_total", "25753026560"),
            ("hwmon/hwmon3/temp1_input", "52000"),
            ("hwmon/hwmon3/power1_average", "61000000"),
            ("hwmon/hwmon3/power1_cap", "303000000"),
        ]
    }

    #[test]
    fn discovers_supported_gpus_by_pci_address() {
        let fake = FakeGpus::new();
        fake.gpu("card1", "0000:10:00.0", "amdgpu", &[("vendor", "0x1002"), ("device", "0x164e")]);
        fake.gpu("card0", "0000:01:00.0", "nvidia", &[("vendor", "0x10de"), ("device", "0x2803")]);
        fake.gpu("card2", "0000:00:02.0", "i915", &[]);
        // A connector of card1, which isn't a card of its own.
        symlink(
            fake.root.path().join("devices/0000:10:00.0"),
            fake.root.path().join("drm/card1-DP-1"),
        )
        .unwrap();

        let gpus = discover_gpus(&fake.paths());

        let found: Vec<(&str, GpuDriver, &str)> = gpus
            .iter()
            .map(|gpu| (gpu.pci_address.as_str(), gpu.driver, gpu.name.as_str()))
            .collect();
        assert_eq!(
            found,
            vec![
                ("0000:01:00.0", GpuDriver::Nvidia, "NVIDIA GeForce RTX 4060 Ti"),
                ("0000:10:00.0", GpuDriver::Amdgpu, "AMD Raphael"),
            ]
        );
    }

    #[test]
    fn a_product_name_beats_the_database() {
        let fake = FakeGpus::new();
        fake.gpu(
            "card0",
            "0000:03:00.0",
            "amdgpu",
            &[("vendor", "0x1002"), ("device", "0x744c"), ("product_name", "Radeon RX 7900 XTX")],
        );
        assert_eq!(discover_gpus(&fake.paths())[0].name, "AMD Radeon RX 7900 XTX");
    }

    #[test]
    fn an_unknown_gpu_has_a_generic_name() {
        let fake = FakeGpus::new();
        fake.gpu("card0", "0000:03:00.0", "amdgpu", &[("vendor", "0x1002"), ("device", "0xffff")]);
        assert_eq!(discover_gpus(&fake.paths())[0].name, "AMD GPU");
    }

    #[test]
    fn reads_whether_an_amd_gpu_can_sleep() {
        let fake = FakeGpus::new();
        fake.gpu("card0", "0000:03:00.0", "amdgpu", &amd_files());
        fake.gpu("card1", "0000:10:00.0", "amdgpu", &[("power/control", "on")]);

        let gpus = discover_gpus(&fake.paths());

        assert!(gpus[0].can_sleep);
        assert_eq!(gpus[0].autosuspend_delay, Some(Duration::from_secs(5)));
        assert_eq!(gpus[0].vram_total_bytes, Some(25753026560));
        assert!(!gpus[1].can_sleep);
        assert_eq!(gpus[1].autosuspend_delay, None);
    }

    #[test]
    fn an_nvidia_gpu_only_sleeps_when_the_driver_supports_it() {
        let fake = FakeGpus::new();
        fake.gpu("card0", "0000:01:00.0", "nvidia", &[("power/control", "auto")]);
        fake.gpu("card1", "0000:02:00.0", "nvidia", &[("power/control", "auto")]);
        fake.gpu("card2", "0000:03:00.0", "nvidia", &[("power/control", "auto")]);
        let nvidia = fake.root.path().join("nvidia");
        fs::create_dir_all(nvidia.join("0000:01:00.0")).unwrap();
        fs::write(
            nvidia.join("0000:01:00.0/power"),
            "Runtime D3 status:          Disabled by default\nVideo Memory:               Active\n",
        )
        .unwrap();
        fs::create_dir_all(nvidia.join("0000:02:00.0")).unwrap();
        fs::write(nvidia.join("0000:02:00.0/power"), "Runtime D3 status:          Enabled (fine-grained)\n").unwrap();

        let gpus = discover_gpus(&fake.paths());

        assert!(!gpus[0].can_sleep);
        assert!(gpus[1].can_sleep);
        // Without the driver's status, the kernel's setting is all there is to go on.
        assert!(gpus[2].can_sleep);
    }

    #[test]
    fn reads_whether_a_gpu_is_awake() {
        let fake = FakeGpus::new();
        let device = fake.gpu("card0", "0000:03:00.0", "amdgpu", &[("power/runtime_status", "active")]);
        let gpu = || discover_gpus(&fake.paths()).remove(0);

        assert!(is_awake(&gpu()));
        for (status, awake) in [("suspended", false), ("suspending", false), ("resuming", false), ("unsupported", true)]
        {
            fs::write(device.join("power/runtime_status"), status).unwrap();
            assert_eq!(is_awake(&gpu()), awake, "{status}");
        }
        fs::remove_file(device.join("power/runtime_status")).unwrap();
        assert!(is_awake(&gpu()));
    }

    #[test]
    fn reads_an_amd_gpu() {
        let fake = FakeGpus::new();
        fake.gpu("card0", "0000:03:00.0", "amdgpu", &amd_files());

        let reading = read_amd(&discover_gpus(&fake.paths())[0]);

        assert_eq!(
            reading,
            GpuReading {
                usage_percent: Some(37.0),
                vram_used_bytes: Some(3221225472),
                vram_total_bytes: Some(25753026560),
                temperature_celsius: Some(52.0),
                power_watts: Some(61.0),
            }
        );
    }

    #[test]
    fn an_integrated_amd_gpu_has_no_power_reading() {
        let fake = FakeGpus::new();
        fake.gpu(
            "card0",
            "0000:10:00.0",
            "amdgpu",
            &[("gpu_busy_percent", "3"), ("hwmon/hwmon5/power1_input", "28000000")],
        );

        let reading = read_amd(&discover_gpus(&fake.paths())[0]);

        assert_eq!(reading.usage_percent, Some(3.0));
        assert_eq!(reading.power_watts, None);
        assert_eq!(reading.vram_total_bytes, None);
    }

    #[test]
    fn finds_device_names_under_their_vendor() {
        let find = |vendor, device| find_pci_device_name(PCI_IDS.as_bytes(), vendor, device);

        assert_eq!(find(0x1002, 0x164e).as_deref(), Some("Raphael"));
        assert_eq!(find(0x1002, 0x744c).as_deref(), Some("Radeon RX 7900 XT/7900 XTX/7900 GRE/7900M"));
        assert_eq!(find(0x10de, 0x2803).as_deref(), Some("GeForce RTX 4060 Ti"));
        assert_eq!(find(0x1022, 0x164e).as_deref(), Some("Not a GPU"));
        // The subsystem has the same number as a device, but isn't one.
        assert_eq!(find(0x1002, 0x1043), None);
        assert_eq!(find(0x10de, 0x164e), None);
        assert_eq!(find(0x8086, 0x1234), None);
    }

    /// Lists the GPUs of the machine the tests run on, and reads the AMD ones that are awake. Run with
    /// `cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn reads_the_gpus_of_this_machine() {
        for gpu in discover_gpus(&GpuPaths::system()) {
            println!("{gpu:?}, awake: {}", is_awake(&gpu));
            if gpu.driver == GpuDriver::Amdgpu && is_awake(&gpu) {
                let reading = read_amd(&gpu);
                println!("    {reading:?}");
                assert!(reading.usage_percent.is_some());
                assert!(reading.vram_total_bytes.is_some());
            }
        }
    }

    #[test]
    fn a_missing_root_has_no_gpus() {
        let paths = GpuPaths {
            drm_root: PathBuf::from("/nonexistent/drm"),
            nvidia_proc_root: PathBuf::from("/nonexistent/nvidia"),
            pci_ids: Vec::new(),
        };
        assert!(discover_gpus(&paths).is_empty());
    }
}
