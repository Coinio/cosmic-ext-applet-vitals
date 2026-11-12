//! Reads the GPUs' usage and VRAM, without keeping a GPU that could be asleep awake.
//!
//! A GPU the kernel can put to sleep, e.g. a laptop's discrete GPU, is never read while it's asleep, as reading it
//! wakes it. That alone isn't enough though: the kernel only puts a GPU to sleep after it has been idle for a while,
//! its autosuspend delay, and every read counts as using it, so reading it every second would keep it awake forever.
//! So once such a GPU has been idle for a few reads in a row, it isn't read again until it has had the time to fall
//! asleep. If it's still awake then, something else is keeping it awake, and reading it carries on.
//!
//! Only the GPU shown in the panel is read on every tick. The others are only read while the detail popup is wanted.

use crate::configuration::app_configuration::AppConfiguration;
use crate::sensors::gpu_reader::{self, discover_gpus, GpuDevice, GpuDriver, GpuPaths, GpuReading};
use crate::sensors::nvml::Nvml;
use log::{info, warn};
use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

/// A GPU that can sleep counts as idle while its usage is below this.
const IDLE_USAGE_PERCENT: f64 = 1.0;
/// How many reads in a row a GPU that can sleep has to be idle for before it's left alone to fall asleep.
const IDLE_READS_BEFORE_PAUSE: u32 = 3;
/// How long an idle GPU is left alone for when its driver doesn't say how long it waits before putting it to sleep.
const DEFAULT_IDLE_PAUSE: Duration = Duration::from_secs(15);
/// How much longer than the autosuspend delay an idle GPU is left alone for, so it has surely fallen asleep.
const IDLE_PAUSE_MARGIN: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GpuState {
    /// Being read.
    #[default]
    Active,
    /// Idle, and left alone so it can fall asleep. Its readings are from before it was left alone.
    Idle,
    /// Asleep, so it isn't read.
    Asleep,
    /// It couldn't be read, e.g. NVML isn't installed.
    Unavailable,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GpuDeviceStats {
    pub pci_address: String,
    pub name: String,
    pub state: GpuState,
    /// The usage, averaged over the samples.
    pub usage_percent: Option<f64>,
    pub vram_used_bytes: Option<u64>,
    pub vram_total_bytes: Option<u64>,
    pub temperature_celsius: Option<f64>,
    pub power_watts: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct GpuStats {
    /// Every supported GPU, by PCI address.
    pub gpus: Vec<GpuDeviceStats>,
    /// The index of the GPU shown in the panel, `None` when there are no GPUs.
    pub panel: Option<usize>,
}

impl GpuStats {
    pub fn panel_gpu(&self) -> Option<&GpuDeviceStats> {
        self.panel.and_then(|index| self.gpus.get(index))
    }
}

/// Reads the GPUs. The tests replace it, to control what the GPUs report.
pub trait GpuReader {
    /// Whether the GPU is awake, which mustn't wake it.
    fn is_awake(&self, gpu: &GpuDevice) -> bool;
    /// Read the GPU, which wakes it if it's asleep.
    fn read(&mut self, gpu: &GpuDevice) -> Result<GpuReading, String>;
    /// Let go of the GPU, so holding on to it doesn't stop it from sleeping.
    fn release(&mut self, gpu: &GpuDevice);
}

/// Reads AMD GPUs from sysfs, and NVIDIA GPUs through NVML, which is only initialised while an NVIDIA GPU is held.
pub struct SystemGpuReader {
    nvml: Option<Nvml>,
    /// The NVIDIA GPUs being read, by PCI address. NVML is shut down when there are none.
    held: HashSet<String>,
    /// The NVIDIA GPUs that can sleep. Initialising NVML opens every NVIDIA GPU, so it's only initialised while all
    /// of them are awake.
    sleepable_nvidia_gpus: Vec<GpuDevice>,
}

impl SystemGpuReader {
    pub fn new(gpus: &[GpuDevice]) -> Self {
        SystemGpuReader {
            nvml: None,
            held: HashSet::new(),
            sleepable_nvidia_gpus: gpus
                .iter()
                .filter(|gpu| gpu.driver == GpuDriver::Nvidia && gpu.can_sleep)
                .cloned()
                .collect(),
        }
    }

    fn nvml(&mut self) -> Result<&mut Nvml, String> {
        if self.nvml.is_none() {
            if let Some(asleep) = self.sleepable_nvidia_gpus.iter().find(|gpu| !gpu_reader::is_awake(gpu)) {
                return Err(format!(
                    "NVML would wake the NVIDIA GPU at {}, which is asleep",
                    asleep.pci_address
                ));
            }
        }
        match self.nvml {
            Some(ref mut nvml) => Ok(nvml),
            None => Ok(self.nvml.insert(Nvml::init()?)),
        }
    }
}

impl GpuReader for SystemGpuReader {
    fn is_awake(&self, gpu: &GpuDevice) -> bool {
        gpu_reader::is_awake(gpu)
    }

    fn read(&mut self, gpu: &GpuDevice) -> Result<GpuReading, String> {
        match gpu.driver {
            GpuDriver::Amdgpu => Ok(gpu_reader::read_amd(gpu)),
            GpuDriver::Nvidia => {
                let reading = self.nvml()?.read(&gpu.pci_address);
                self.held.insert(gpu.pci_address.clone());
                reading
            }
        }
    }

    fn release(&mut self, gpu: &GpuDevice) {
        // NVML can't let go of a single GPU, only of all of them, when it's shut down.
        if self.held.remove(&gpu.pci_address) && self.held.is_empty() {
            self.nvml = None;
        }
    }
}

struct TrackedGpu {
    device: GpuDevice,
    usage_samples: VecDeque<f64>,
    /// How many reads in a row it has been idle for.
    idle_reads: u32,
    /// When it's left alone to fall asleep, until when.
    paused_until: Option<Instant>,
    stats: GpuDeviceStats,
}

impl TrackedGpu {
    fn idle_pause(&self) -> Duration {
        self.device
            .autosuspend_delay
            .map_or(DEFAULT_IDLE_PAUSE, |delay| delay + IDLE_PAUSE_MARGIN)
    }

    fn reset_samples(&mut self) {
        self.usage_samples.clear();
        self.idle_reads = 0;
    }
}

pub struct GpuMonitor<R: GpuReader> {
    reader: R,
    gpus: Vec<TrackedGpu>,
    panel: Option<usize>,
    max_samples: usize,
}

impl GpuMonitor<SystemGpuReader> {
    /// Find the GPUs once. A GPU added later, e.g. an eGPU, is picked up the next time the monitors start, e.g. when
    /// the configuration changes.
    pub fn new(paths: &GpuPaths, configuration: &AppConfiguration) -> Self {
        let gpus = discover_gpus(paths);
        Self::with_gpus(SystemGpuReader::new(&gpus), gpus, configuration)
    }
}

impl<R: GpuReader> GpuMonitor<R> {
    fn with_gpus(reader: R, devices: Vec<GpuDevice>, configuration: &AppConfiguration) -> Self {
        let panel = panel_gpu(&devices, configuration.gpu.device.as_deref());

        info!(
            "Creating new gpu monitor {:?}, gpus: {:?}, panel: {:?}",
            configuration.gpu,
            devices.iter().map(|gpu| (&gpu.pci_address, &gpu.name, gpu.can_sleep)).collect::<Vec<_>>(),
            panel.map(|index| &devices[index].pci_address)
        );

        let gpus = devices
            .into_iter()
            .map(|device| TrackedGpu {
                stats: GpuDeviceStats {
                    pci_address: device.pci_address.clone(),
                    name: device.name.clone(),
                    ..GpuDeviceStats::default()
                },
                device,
                usage_samples: VecDeque::with_capacity(configuration.gpu.max_samples),
                idle_reads: 0,
                paused_until: None,
            })
            .collect();

        Self {
            reader,
            gpus,
            panel,
            max_samples: configuration.gpu.max_samples.max(1),
        }
    }

    /// Read the panel's GPU, and every other GPU too when `all`, e.g. while the detail popup is wanted.
    pub fn poll(&mut self, all: bool) -> GpuStats {
        self.poll_at(all, Instant::now())
    }

    fn poll_at(&mut self, all: bool, now: Instant) -> GpuStats {
        for index in 0..self.gpus.len() {
            let gpu = &mut self.gpus[index];
            if all || Some(index) == self.panel {
                poll_gpu(&mut self.reader, gpu, self.max_samples, now);
            } else if gpu.device.can_sleep {
                // It was read while the popup was open, and won't be again until it reopens.
                self.reader.release(&gpu.device);
                gpu.reset_samples();
            }
        }

        GpuStats {
            gpus: self.gpus.iter().map(|gpu| gpu.stats.clone()).collect(),
            panel: self.panel,
        }
    }
}

fn poll_gpu<R: GpuReader>(reader: &mut R, gpu: &mut TrackedGpu, max_samples: usize, now: Instant) {
    if gpu.device.can_sleep {
        if !reader.is_awake(&gpu.device) {
            reader.release(&gpu.device);
            gpu.reset_samples();
            gpu.paused_until = None;
            gpu.stats = GpuDeviceStats {
                state: GpuState::Asleep,
                // The total doesn't change, so it can still be shown.
                vram_total_bytes: gpu.stats.vram_total_bytes,
                ..identity(&gpu.stats)
            };
            return;
        }

        if gpu.paused_until.is_some_and(|until| now < until) {
            return;
        }
        gpu.paused_until = None;
    }

    let reading = match reader.read(&gpu.device) {
        Ok(reading) => reading,
        Err(err) => {
            if gpu.stats.state != GpuState::Unavailable {
                warn!("Failed to read the GPU {}: {err}", gpu.device.pci_address);
            }
            reader.release(&gpu.device);
            gpu.reset_samples();
            gpu.stats = GpuDeviceStats {
                state: GpuState::Unavailable,
                ..identity(&gpu.stats)
            };
            return;
        }
    };

    if let Some(usage) = reading.usage_percent {
        gpu.usage_samples.push_back(usage);
        while gpu.usage_samples.len() > max_samples {
            gpu.usage_samples.pop_front();
        }
    }
    let usage_percent = (!gpu.usage_samples.is_empty())
        .then(|| gpu.usage_samples.iter().sum::<f64>() / gpu.usage_samples.len() as f64);

    let mut state = GpuState::Active;
    if gpu.device.can_sleep {
        // A GPU that doesn't report its usage can't be told to be busy, so it's left alone too.
        if reading.usage_percent.is_none_or(|usage| usage < IDLE_USAGE_PERCENT) {
            gpu.idle_reads += 1;
        } else {
            gpu.idle_reads = 0;
        }

        if gpu.idle_reads >= IDLE_READS_BEFORE_PAUSE {
            gpu.paused_until = Some(now + gpu.idle_pause());
            reader.release(&gpu.device);
            gpu.reset_samples();
            state = GpuState::Idle;
        }
    }

    gpu.stats = GpuDeviceStats {
        state,
        usage_percent,
        vram_used_bytes: reading.vram_used_bytes,
        vram_total_bytes: reading.vram_total_bytes.or(gpu.device.vram_total_bytes),
        temperature_celsius: reading.temperature_celsius,
        power_watts: reading.power_watts,
        ..identity(&gpu.stats)
    };
}

/// Stats with only the GPU's identity, which don't claim any readings.
fn identity(stats: &GpuDeviceStats) -> GpuDeviceStats {
    GpuDeviceStats {
        pci_address: stats.pci_address.clone(),
        name: stats.name.clone(),
        ..GpuDeviceStats::default()
    }
}

/// The GPU shown in the panel: the configured one, or when there isn't one, or it wasn't found, the discrete GPU.
/// NVIDIA GPUs are always discrete, and an AMD GPU with more VRAM is the discrete one beside an integrated one's
/// small carve-out.
pub fn panel_gpu(gpus: &[GpuDevice], configured: Option<&str>) -> Option<usize> {
    if let Some(configured) = configured {
        match gpus.iter().position(|gpu| gpu.pci_address == configured) {
            Some(index) => return Some(index),
            None => warn!("The configured GPU {configured} wasn't found, so the panel shows the automatic one"),
        }
    }

    // Reversed, so of equally likely GPUs, the first is picked.
    gpus.iter()
        .enumerate()
        .rev()
        .max_by_key(|(_, gpu)| (gpu.driver == GpuDriver::Nvidia, gpu.vram_total_bytes.unwrap_or(0)))
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::rc::Rc;

    #[derive(Default)]
    struct FakeState {
        asleep: HashSet<String>,
        readings: HashMap<String, Result<GpuReading, String>>,
        reads: Vec<String>,
        released: Vec<String>,
    }

    /// Reports what the test sets, and records what the monitor reads and releases.
    #[derive(Clone, Default)]
    struct FakeReader(Rc<RefCell<FakeState>>);

    impl FakeReader {
        fn set_usage(&self, pci_address: &str, usage: f64) {
            self.0.borrow_mut().readings.insert(
                pci_address.to_string(),
                Ok(GpuReading {
                    usage_percent: Some(usage),
                    vram_used_bytes: Some(1_000),
                    vram_total_bytes: Some(8_000),
                    ..GpuReading::default()
                }),
            );
        }

        fn set_asleep(&self, pci_address: &str, asleep: bool) {
            let mut state = self.0.borrow_mut();
            if asleep {
                state.asleep.insert(pci_address.to_string());
            } else {
                state.asleep.remove(pci_address);
            }
        }

        fn take_reads(&self) -> Vec<String> {
            std::mem::take(&mut self.0.borrow_mut().reads)
        }

        fn released(&self) -> Vec<String> {
            self.0.borrow().released.clone()
        }
    }

    impl GpuReader for FakeReader {
        fn is_awake(&self, gpu: &GpuDevice) -> bool {
            !self.0.borrow().asleep.contains(&gpu.pci_address)
        }

        fn read(&mut self, gpu: &GpuDevice) -> Result<GpuReading, String> {
            let mut state = self.0.borrow_mut();
            assert!(!state.asleep.contains(&gpu.pci_address), "Read {} while it was asleep", gpu.pci_address);
            state.reads.push(gpu.pci_address.clone());
            state.readings.get(&gpu.pci_address).cloned().unwrap_or_else(|| Ok(GpuReading::default()))
        }

        fn release(&mut self, gpu: &GpuDevice) {
            self.0.borrow_mut().released.push(gpu.pci_address.clone());
        }
    }

    fn gpu(pci_address: &str, driver: GpuDriver, can_sleep: bool, vram_total_bytes: Option<u64>) -> GpuDevice {
        GpuDevice {
            pci_address: pci_address.to_string(),
            driver,
            name: format!("GPU {pci_address}"),
            device_path: PathBuf::new(),
            hwmon_path: None,
            can_sleep,
            autosuspend_delay: Some(Duration::from_secs(5)),
            vram_total_bytes,
        }
    }

    const IGPU: &str = "0000:10:00.0";
    const DGPU: &str = "0000:01:00.0";

    fn monitor(reader: &FakeReader, gpus: Vec<GpuDevice>, configured: Option<&str>) -> GpuMonitor<FakeReader> {
        let mut configuration = AppConfiguration::default();
        configuration.gpu.max_samples = 2;
        configuration.gpu.device = configured.map(str::to_string);
        GpuMonitor::with_gpus(reader.clone(), gpus, &configuration)
    }

    #[test]
    fn the_panel_shows_the_discrete_gpu_unless_configured() {
        let amd_igpu = gpu(IGPU, GpuDriver::Amdgpu, false, Some(512 << 20));
        let amd_dgpu = gpu("0000:03:00.0", GpuDriver::Amdgpu, true, Some(16 << 30));
        let nvidia = gpu(DGPU, GpuDriver::Nvidia, true, None);

        assert_eq!(panel_gpu(&[nvidia.clone(), amd_igpu.clone()], None), Some(0));
        assert_eq!(panel_gpu(&[amd_igpu.clone(), amd_dgpu.clone()], None), Some(1));
        assert_eq!(panel_gpu(&[amd_igpu.clone(), amd_igpu.clone()], None), Some(0));
        assert_eq!(panel_gpu(&[nvidia.clone(), amd_igpu.clone()], Some(IGPU)), Some(1));
        assert_eq!(panel_gpu(&[nvidia, amd_igpu], Some("0000:99:00.0")), Some(0));
        assert_eq!(panel_gpu(&[], None), None);
    }

    #[test]
    fn only_reads_the_panel_gpu_unless_all_are_wanted() {
        let reader = FakeReader::default();
        reader.set_usage(DGPU, 40.0);
        reader.set_usage(IGPU, 10.0);
        let mut monitor = monitor(
            &reader,
            vec![gpu(DGPU, GpuDriver::Nvidia, false, None), gpu(IGPU, GpuDriver::Amdgpu, false, None)],
            None,
        );

        let stats = monitor.poll(false);
        assert_eq!(reader.take_reads(), vec![DGPU]);
        assert_eq!(stats.panel_gpu().unwrap().usage_percent, Some(40.0));
        assert_eq!(stats.gpus[1].usage_percent, None);

        let stats = monitor.poll(true);
        assert_eq!(reader.take_reads(), vec![DGPU, IGPU]);
        assert_eq!(stats.gpus[1].usage_percent, Some(10.0));
        assert_eq!(stats.gpus[1].vram_used_bytes, Some(1_000));
    }

    #[test]
    fn averages_the_usage_over_the_samples() {
        let reader = FakeReader::default();
        let mut monitor = monitor(&reader, vec![gpu(DGPU, GpuDriver::Nvidia, false, None)], None);

        for (usage, average) in [(10.0, 10.0), (30.0, 20.0), (50.0, 40.0)] {
            reader.set_usage(DGPU, usage);
            assert_eq!(monitor.poll(false).gpus[0].usage_percent, Some(average));
        }
    }

    #[test]
    fn never_reads_a_gpu_that_is_asleep() {
        let reader = FakeReader::default();
        reader.set_usage(DGPU, 50.0);
        let mut monitor = monitor(&reader, vec![gpu(DGPU, GpuDriver::Nvidia, true, None)], None);

        assert_eq!(monitor.poll(false).gpus[0].vram_total_bytes, Some(8_000));

        reader.set_asleep(DGPU, true);
        let stats = monitor.poll(false);

        assert_eq!(stats.gpus[0].state, GpuState::Asleep);
        assert_eq!(stats.gpus[0].usage_percent, None);
        assert_eq!(stats.gpus[0].vram_used_bytes, None);
        assert_eq!(stats.gpus[0].vram_total_bytes, Some(8_000));
        assert_eq!(reader.released(), vec![DGPU]);
    }

    #[test]
    fn leaves_an_idle_gpu_alone_so_it_can_fall_asleep() {
        let reader = FakeReader::default();
        reader.set_usage(DGPU, 0.0);
        let mut monitor = monitor(&reader, vec![gpu(DGPU, GpuDriver::Nvidia, true, None)], None);
        let start = Instant::now();
        let at = |seconds: u64| start + Duration::from_secs(seconds);

        for second in 0..IDLE_READS_BEFORE_PAUSE as u64 {
            monitor.poll_at(false, at(second));
        }
        assert_eq!(reader.take_reads().len(), IDLE_READS_BEFORE_PAUSE as usize);
        assert_eq!(reader.released(), vec![DGPU]);

        // Within the autosuspend delay and the margin, it isn't touched.
        let stats = monitor.poll_at(false, at(8));
        assert!(reader.take_reads().is_empty());
        assert_eq!(stats.gpus[0].state, GpuState::Idle);
        assert_eq!(stats.gpus[0].usage_percent, Some(0.0));

        // It fell asleep.
        reader.set_asleep(DGPU, true);
        assert_eq!(monitor.poll_at(false, at(9)).gpus[0].state, GpuState::Asleep);

        // Something woke it, so it's read again straight away.
        reader.set_asleep(DGPU, false);
        reader.set_usage(DGPU, 80.0);
        let stats = monitor.poll_at(false, at(10));
        assert_eq!(reader.take_reads(), vec![DGPU]);
        assert_eq!(stats.gpus[0].state, GpuState::Active);
        assert_eq!(stats.gpus[0].usage_percent, Some(80.0));
    }

    #[test]
    fn reads_an_idle_gpu_again_when_something_else_keeps_it_awake() {
        let reader = FakeReader::default();
        reader.set_usage(DGPU, 0.0);
        let mut monitor = monitor(&reader, vec![gpu(DGPU, GpuDriver::Nvidia, true, None)], None);
        let start = Instant::now();

        for second in 0..3 {
            monitor.poll_at(false, start + Duration::from_secs(second));
        }
        reader.take_reads();

        // The pause is the 5 second autosuspend delay plus the margin, after the third read.
        monitor.poll_at(false, start + Duration::from_secs(11));
        assert!(reader.take_reads().is_empty());
        monitor.poll_at(false, start + Duration::from_secs(12));
        assert_eq!(reader.take_reads(), vec![DGPU]);
    }

    #[test]
    fn keeps_reading_a_busy_gpu_that_can_sleep() {
        let reader = FakeReader::default();
        reader.set_usage(DGPU, 25.0);
        let mut monitor = monitor(&reader, vec![gpu(DGPU, GpuDriver::Nvidia, true, None)], None);
        let start = Instant::now();

        for second in 0..10 {
            monitor.poll_at(false, start + Duration::from_secs(second));
        }

        assert_eq!(reader.take_reads().len(), 10);
        assert!(reader.released().is_empty());
    }

    #[test]
    fn keeps_reading_an_idle_gpu_that_cant_sleep() {
        let reader = FakeReader::default();
        reader.set_usage(IGPU, 0.0);
        let mut monitor = monitor(&reader, vec![gpu(IGPU, GpuDriver::Amdgpu, false, None)], None);
        let start = Instant::now();

        for second in 0..10 {
            monitor.poll_at(false, start + Duration::from_secs(second));
        }

        assert_eq!(reader.take_reads().len(), 10);
    }

    #[test]
    fn lets_go_of_the_other_gpus_when_the_popup_closes() {
        let reader = FakeReader::default();
        reader.set_usage(DGPU, 30.0);
        reader.set_usage(IGPU, 30.0);
        let mut monitor = monitor(
            &reader,
            vec![gpu(DGPU, GpuDriver::Nvidia, true, None), gpu(IGPU, GpuDriver::Amdgpu, true, None)],
            Some(IGPU),
        );

        monitor.poll(true);
        assert!(reader.released().is_empty());

        monitor.poll(false);
        assert_eq!(reader.released(), vec![DGPU]);
    }

    #[test]
    fn a_gpu_that_cant_be_read_is_unavailable() {
        let reader = FakeReader::default();
        reader
            .0
            .borrow_mut()
            .readings
            .insert(DGPU.to_string(), Err("NVML isn't installed".to_string()));
        let mut monitor = monitor(&reader, vec![gpu(DGPU, GpuDriver::Nvidia, false, None)], None);

        let stats = monitor.poll(false);

        assert_eq!(stats.gpus[0].state, GpuState::Unavailable);
        assert_eq!(stats.gpus[0].name, format!("GPU {DGPU}"));
        assert_eq!(stats.gpus[0].usage_percent, None);
    }
}
