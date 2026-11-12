use crate::configuration::app_configuration::AppConfiguration;
use crate::sensors::proc_disk_stats_reader::{read_disk_model, ProcDiskStats};
use crate::sensors::sensor_traits::SensorReader;
use log::info;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::time::Duration;

const SECTOR_SIZE_BYTES: u64 = 512;

#[derive(Debug, Clone, Default)]
struct DiskSample {
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub io_time_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct DiskStats {
    /// The bytes written to all the disks, averaged over the samples.
    pub avg_bytes_written: u64,
    /// The bytes read from all the disks, averaged over the samples.
    pub avg_bytes_read: u64,
    /// Each disk, by name.
    pub devices: Vec<DiskDeviceStats>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiskDeviceStats {
    pub name: String,
    pub model: Option<String>,
    /// The bytes read, averaged over the samples.
    pub bytes_read: u64,
    /// The bytes written, averaged over the samples.
    pub bytes_written: u64,
    /// The share of the time the disk had I/O in flight over the samples, from 0 to 100.
    pub busy_percent: f64,
}

/// The samples of one disk, which are kept apart so a disk coming or going doesn't skew the others.
#[derive(Debug, Default)]
struct DiskHistory {
    model: Option<String>,
    previous: DiskSample,
    samples: VecDeque<DiskSample>,
}

pub struct DiskMonitor<S: SensorReader<Output = ProcDiskStats>> {
    sensor_reader: S,
    sys_block_root: PathBuf,
    disks: HashMap<String, DiskHistory>,
    max_samples: usize,
    update_interval: Duration,
}

impl<S: SensorReader<Output = ProcDiskStats>> DiskMonitor<S> {
    /// `sys_block_root` is where each disk's model is read from.
    pub fn new(sensor_reader: S, configuration: &AppConfiguration, sys_block_root: &Path) -> Self {
        info!("Creating new disk monitor {:?}", configuration.disk);
        Self {
            sensor_reader,
            sys_block_root: sys_block_root.to_path_buf(),
            disks: HashMap::new(),
            max_samples: configuration.disk.max_samples,
            update_interval: configuration.disk.update_interval,
        }
    }

    pub fn poll(&mut self) -> Result<DiskStats, String> {
        let current = self.sensor_reader.read()?;

        let mut devices = Vec::new();
        let mut previous_disks = std::mem::take(&mut self.disks);

        for device in current.device_statuses {
            if !is_logical_disk(&device.device_name) {
                continue;
            }

            let totals = DiskSample {
                bytes_read: device.sectors_read.saturating_mul(SECTOR_SIZE_BYTES),
                bytes_written: device.sectors_written.saturating_mul(SECTOR_SIZE_BYTES),
                io_time_ms: device.io_time_ms,
            };

            // A new disk starts from its current totals, so its first sample is 0 rather than everything since boot.
            // Its model doesn't change, so it's only read now.
            let mut history = previous_disks.remove(&device.device_name).unwrap_or_else(|| DiskHistory {
                model: read_disk_model(&self.sys_block_root, &device.device_name),
                previous: totals.clone(),
                samples: VecDeque::with_capacity(self.max_samples),
            });

            // Saturating, as a disk that's removed and added again starts its totals from 0.
            history.samples.push_back(DiskSample {
                bytes_read: totals.bytes_read.saturating_sub(history.previous.bytes_read),
                bytes_written: totals.bytes_written.saturating_sub(history.previous.bytes_written),
                io_time_ms: totals.io_time_ms.saturating_sub(history.previous.io_time_ms),
            });
            while history.samples.len() > self.max_samples {
                history.samples.pop_front();
            }
            history.previous = totals;

            devices.push(DiskDeviceStats {
                name: device.device_name.clone(),
                model: history.model.clone(),
                bytes_read: average(history.samples.iter().map(|sample| sample.bytes_read)),
                bytes_written: average(history.samples.iter().map(|sample| sample.bytes_written)),
                busy_percent: busy_percent(&history.samples, self.update_interval),
            });

            self.disks.insert(device.device_name, history);
        }

        devices.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(DiskStats {
            avg_bytes_read: devices.iter().map(|device| device.bytes_read).sum(),
            avg_bytes_written: devices.iter().map(|device| device.bytes_written).sum(),
            devices,
        })
    }
}

fn average(values: impl ExactSizeIterator<Item = u64>) -> u64 {
    let count = values.len() as u64;
    if count == 0 {
        return 0;
    }
    values.sum::<u64>() / count
}

/// The share of the samples' time the disk was busy. Each sample covers one update interval, as the data rates do.
fn busy_percent(samples: &VecDeque<DiskSample>, update_interval: Duration) -> f64 {
    let elapsed_ms = update_interval.as_millis() as f64 * samples.len() as f64;
    if elapsed_ms <= 0.0 {
        return 0.0;
    }
    let busy_ms: u64 = samples.iter().map(|sample| sample.io_time_ms).sum();
    (busy_ms as f64 / elapsed_ms * 100.0).clamp(0.0, 100.0)
}

fn is_logical_disk(name: &str) -> bool {
    if name.starts_with("nvme") && !name.contains("p") {
        return true;
    }
    if name.starts_with("sd") && !name.chars().last().unwrap_or_default().is_numeric() {
        return true;
    }
    if name.starts_with("hd") && !name.chars().last().unwrap_or_default().is_numeric() {
        return true;
    }

    false
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::sensors::proc_disk_stats_reader::ProcDiskStatsStatus;
    use std::cell::Cell;

    struct MockProcDiskStatsReader {
        pub samples: VecDeque<Result<ProcDiskStats, String>>,
        pub index: Cell<usize>,
    }

    impl MockProcDiskStatsReader {
        pub fn new(samples: Vec<Result<ProcDiskStats, String>>) -> Self {
            Self {
                samples: samples.into(),
                index: Cell::new(0),
            }
        }
    }

    impl SensorReader for MockProcDiskStatsReader {
        type Output = ProcDiskStats;

        fn read(&self) -> Result<Self::Output, String> {
            let i = self.index.get();
            let result = self.samples[i].clone();
            self.index.set(i + 1);
            result
        }
    }

    fn make_config(max_samples: usize) -> AppConfiguration {
        let mut cfg = AppConfiguration::default();
        cfg.disk.max_samples = max_samples;
        cfg.disk.update_interval = Duration::from_secs(1);
        cfg
    }

    /// A monitor without models to read.
    fn make_monitor(reader: MockProcDiskStatsReader, max_samples: usize) -> DiskMonitor<MockProcDiskStatsReader> {
        DiskMonitor::new(reader, &make_config(max_samples), Path::new("/nonexistent"))
    }

    fn disk_status(name: &str, sectors_read: u64, sectors_written: u64, io_time_ms: u64) -> ProcDiskStatsStatus {
        ProcDiskStatsStatus {
            device_name: name.to_string(),
            sectors_read,
            sectors_written,
            io_time_ms,
        }
    }

    fn make_sensor_sample(device_name: String, sectors_read: u64, sectors_written: u64) -> ProcDiskStats {
        ProcDiskStats::new(vec![ProcDiskStatsStatus::new(
            device_name,
            sectors_read,
            sectors_written,
        )])
    }

    #[test]
    fn first_poll_gives_zero_result() {
        let statuses = make_sensor_sample("nvme0n1".to_string(), 1000, 1000);

        let reader = MockProcDiskStatsReader::new(vec![Ok(statuses)]);

        let mut monitor = make_monitor(reader, 3);

        let result = monitor.poll();

        assert!(result.is_ok());

        let result = result.unwrap();

        assert_eq!(result.avg_bytes_read, 0);
        assert_eq!(result.avg_bytes_written, 0);
    }

    #[test]
    fn second_poll_gives_expected_result() {
        let sample1 = make_sensor_sample("nvme0n1".to_string(), 1000, 1000);
        let sample2 = make_sensor_sample("nvme0n1".to_string(), 2000, 2000);

        let reader = MockProcDiskStatsReader::new(vec![Ok(sample1), Ok(sample2)]);

        let mut monitor = make_monitor(reader, 3);

        // Throw away first result, as will always be 0
        _ = monitor.poll();
        let result = monitor.poll();

        assert!(result.is_ok());

        let result = result.unwrap();

        assert_eq!(result.avg_bytes_read, ((2000 - 1000) * 512) / 2);
        assert_eq!(result.avg_bytes_written, ((2000 - 1000) * 512) / 2);
    }

    #[test]
    fn multiple_polls_gives_expected_result() {
        let sample1 = make_sensor_sample("nvme0n1".to_string(), 1000, 1000);
        let sample2 = make_sensor_sample("nvme0n1".to_string(), 2000, 2000);
        let sample3 = make_sensor_sample("nvme0n1".to_string(), 3000, 3000);

        let reader = MockProcDiskStatsReader::new(vec![Ok(sample1), Ok(sample2), Ok(sample3)]);

        let mut monitor = make_monitor(reader, 3);

        // Throw away first result, as will always be 0
        _ = monitor.poll();

        let result1 = monitor.poll();
        let result2 = monitor.poll();

        assert!(result1.is_ok());

        let result1 = result1.unwrap();

        assert_eq!(result1.avg_bytes_read, ((2000 - 1000) * 512) / 2);
        assert_eq!(result1.avg_bytes_written, ((2000 - 1000) * 512) / 2);

        assert!(result2.is_ok());

        let result2 = result2.unwrap();

        assert_eq!(result2.avg_bytes_read, ((3000 - 1000) * 512) / 3);
        assert_eq!(result2.avg_bytes_written, ((3000 - 1000) * 512) / 3);
    }

    #[test]
    fn samples_buffer_trims_to_max_size() {
        let sample1 = make_sensor_sample("nvme0n1".to_string(), 1000, 1000);
        let sample2 = make_sensor_sample("nvme0n1".to_string(), 2000, 2000);
        let sample3 = make_sensor_sample("nvme0n1".to_string(), 3000, 3000);

        let reader = MockProcDiskStatsReader::new(vec![Ok(sample1), Ok(sample2), Ok(sample3)]);

        let mut monitor = make_monitor(reader, 2);

        // Throw away first result, as will always be 0
        _ = monitor.poll();
        _ = monitor.poll();
        _ = monitor.poll();

        assert_eq!(monitor.disks["nvme0n1"].samples.len(), 2);
    }

    #[test]
    fn uses_the_disk_max_samples() {
        let mut config = AppConfiguration::default();
        config.disk.max_samples = 7;
        config.memory.max_samples = 1;

        let monitor = DiskMonitor::new(MockProcDiskStatsReader::new(vec![]), &config, Path::new("/nonexistent"));

        assert_eq!(monitor.max_samples, 7);
    }

    #[test]
    fn reports_each_disk_with_its_model() {
        let root = tempfile::tempdir().unwrap();
        let device = root.path().join("sda").join("device");
        std::fs::create_dir_all(&device).unwrap();
        std::fs::write(device.join("model"), "Samsung SSD 870\n").unwrap();

        let sample1 = ProcDiskStats::new(vec![
            disk_status("sda", 1000, 1000, 0),
            disk_status("sda1", 1000, 1000, 0),
            disk_status("nvme0n1", 1000, 1000, 0),
        ]);
        let sample2 = ProcDiskStats::new(vec![
            disk_status("sda", 3000, 1000, 250),
            disk_status("sda1", 3000, 1000, 250),
            disk_status("nvme0n1", 1000, 2000, 1000),
        ]);

        let reader = MockProcDiskStatsReader::new(vec![Ok(sample1), Ok(sample2)]);
        let mut monitor = DiskMonitor::new(reader, &make_config(1), root.path());

        _ = monitor.poll();
        let result = monitor.poll().unwrap();

        assert_eq!(
            result.devices,
            [
                DiskDeviceStats {
                    name: "nvme0n1".to_string(),
                    model: None,
                    bytes_read: 0,
                    bytes_written: 1000 * 512,
                    busy_percent: 100.0,
                },
                DiskDeviceStats {
                    name: "sda".to_string(),
                    model: Some("Samsung SSD 870".to_string()),
                    bytes_read: 2000 * 512,
                    bytes_written: 0,
                    busy_percent: 25.0,
                },
            ]
        );
        assert_eq!(result.avg_bytes_read, 2000 * 512);
        assert_eq!(result.avg_bytes_written, 1000 * 512);
    }

    #[test]
    fn busy_is_averaged_over_the_samples_and_capped() {
        let reader = MockProcDiskStatsReader::new(vec![
            Ok(ProcDiskStats::new(vec![disk_status("sda", 0, 0, 1000)])),
            Ok(ProcDiskStats::new(vec![disk_status("sda", 0, 0, 1600)])),
            // More than the interval, as polls aren't exactly an interval apart.
            Ok(ProcDiskStats::new(vec![disk_status("sda", 0, 0, 2800)])),
        ]);
        let mut monitor = make_monitor(reader, 2);

        _ = monitor.poll();
        assert_eq!(monitor.poll().unwrap().devices[0].busy_percent, 30.0);
        assert_eq!(monitor.poll().unwrap().devices[0].busy_percent, 90.0);

        let mut samples = VecDeque::new();
        samples.push_back(DiskSample { io_time_ms: 1500, ..Default::default() });
        assert_eq!(busy_percent(&samples, Duration::from_secs(1)), 100.0);
    }

    /// Unplugging a USB disk removes its totals, which mustn't make the totals go backwards.
    #[test]
    fn removed_disk_is_dropped() {
        let reader = MockProcDiskStatsReader::new(vec![
            Ok(ProcDiskStats::new(vec![disk_status("sda", 1000, 1000, 0), disk_status("sdb", 90000, 90000, 0)])),
            Ok(ProcDiskStats::new(vec![disk_status("sda", 2000, 1000, 0)])),
        ]);
        let mut monitor = make_monitor(reader, 1);

        _ = monitor.poll();
        let result = monitor.poll().unwrap();

        assert_eq!((result.avg_bytes_read, result.avg_bytes_written), (1000 * 512, 0));
        assert_eq!(result.devices.len(), 1);
    }

    #[test]
    fn include_only_logical_disks() {
        let samples1 = ProcDiskStats::new(vec![
            ProcDiskStatsStatus::new("nvme0n1".to_string(), 1000, 1000),
            ProcDiskStatsStatus::new("nvme0n1p1".to_string(), 1000, 1000),
            ProcDiskStatsStatus::new("nvme0n1p2".to_string(), 1000, 1000),
            ProcDiskStatsStatus::new("nvme0n1p3".to_string(), 1000, 1000),
            ProcDiskStatsStatus::new("nvme0n1p133".to_string(), 1000, 1000),
            ProcDiskStatsStatus::new("sda".to_string(), 1000, 1000),
            ProcDiskStatsStatus::new("sda1".to_string(), 1000, 1000),
            ProcDiskStatsStatus::new("dm-0".to_string(), 1000, 1000),
            ProcDiskStatsStatus::new("loop0".to_string(), 1000, 1000),
        ]);

        let samples2 = ProcDiskStats::new(vec![
            ProcDiskStatsStatus::new("nvme0n1".to_string(), 2000, 2000),
            ProcDiskStatsStatus::new("nvme0n1p1".to_string(), 2000, 2000),
            ProcDiskStatsStatus::new("nvme0n1p2".to_string(), 2000, 2000),
            ProcDiskStatsStatus::new("nvme0n1p3".to_string(), 2000, 2000),
            ProcDiskStatsStatus::new("nvme0n1p133".to_string(), 1000, 1000),
            ProcDiskStatsStatus::new("sda".to_string(), 2000, 2000),
            ProcDiskStatsStatus::new("sda1".to_string(), 2000, 2000),
            ProcDiskStatsStatus::new("dm-0".to_string(), 2000, 2000),
            ProcDiskStatsStatus::new("loop0".to_string(), 2000, 2000),
        ]);

        // Add the samples twice, as first sample will be ignored as no delta to compare against.
        let reader = MockProcDiskStatsReader::new(vec![Ok(samples1), Ok(samples2)]);

        let mut monitor = make_monitor(reader, 2);

        // Throw away first result, as will always be 0
        _ = monitor.poll();

        let result = monitor.poll();

        assert!(result.is_ok());

        let result = result.unwrap();

        assert_eq!(
            result.avg_bytes_read,
            (((2000 + 2000) - (1000 + 1000)) * 512) / 2
        );
        assert_eq!(
            result.avg_bytes_written,
            (((2000 + 2000) - (1000 + 1000)) * 512) / 2
        );
    }

    #[test]
    fn error_is_propagated() {
        let reader = MockProcDiskStatsReader::new(vec![Err("boom".to_string())]);
        let mut monitor = make_monitor(reader, 1);

        let err = monitor.poll().unwrap_err();
        assert_eq!(err, "boom");
    }
}
