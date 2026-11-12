use crate::configuration::app_configuration::AppConfiguration;
use crate::monitors::per_second;
use crate::sensors::proc_disk_stats_reader::{has_media, is_disk, is_disk_in, mounted_devices, read_disk_model, ProcDiskStats};
use crate::sensors::proc_mounts_reader::MountEntry;
use crate::sensors::sensor_traits::SensorReader;
use log::info;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const SECTOR_SIZE_BYTES: u64 = 512;

#[derive(Debug, Clone, Default)]
struct DiskSample {
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub io_time_ms: u64,
    /// The time the sample was counted over.
    pub elapsed: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct DiskStats {
    /// The bytes written per second to all the disks, averaged over the samples.
    pub avg_bytes_written: u64,
    /// The bytes read per second from all the disks, averaged over the samples.
    pub avg_bytes_read: u64,
    /// Each disk, by name.
    pub devices: Vec<DiskDeviceStats>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiskDeviceStats {
    pub name: String,
    pub model: Option<String>,
    /// Whether the disk, or one of its partitions, is mounted. Also true when the mounts couldn't be read.
    pub is_mounted: bool,
    /// The bytes read per second, averaged over the samples.
    pub bytes_read: u64,
    /// The bytes written per second, averaged over the samples.
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

pub struct DiskMonitor<S: SensorReader<Output = ProcDiskStats>, M: SensorReader<Output = Vec<MountEntry>>> {
    sensor_reader: S,
    mounts_reader: M,
    sys_block_root: PathBuf,
    disks: HashMap<String, DiskHistory>,
    /// The block devices that aren't disks, e.g. partitions and loop devices, which are only checked once.
    not_disks: HashSet<String>,
    max_samples: usize,
    /// When the totals were last read.
    last_poll: Option<Instant>,
}

impl<S: SensorReader<Output = ProcDiskStats>, M: SensorReader<Output = Vec<MountEntry>>> DiskMonitor<S, M> {
    /// `sys_block_root` is where the disks, and each disk's model, are read from.
    pub fn new(sensor_reader: S, mounts_reader: M, configuration: &AppConfiguration, sys_block_root: &Path) -> Self {
        info!("Creating new disk monitor {:?}", configuration.disk);
        Self {
            sensor_reader,
            mounts_reader,
            sys_block_root: sys_block_root.to_path_buf(),
            disks: HashMap::new(),
            not_disks: HashSet::new(),
            max_samples: configuration.disk.max_samples,
            last_poll: None,
        }
    }

    pub fn poll(&mut self) -> Result<DiskStats, String> {
        self.poll_at(Instant::now())
    }

    /// Poll, with the totals read at `now`.
    fn poll_at(&mut self, now: Instant) -> Result<DiskStats, String> {
        let current = self.sensor_reader.read()?;
        let elapsed = self.last_poll.map_or(Duration::ZERO, |last| now.saturating_duration_since(last));
        self.last_poll = Some(now);

        // Without the mounts, every disk is taken to be mounted rather than saying they all aren't.
        let mounted = self
            .mounts_reader
            .read()
            .ok()
            .map(|mounts| mounted_devices(&self.sys_block_root, &mounts));

        let mut devices = Vec::new();
        let mut previous_disks = std::mem::take(&mut self.disks);
        let mut previous_not_disks = std::mem::take(&mut self.not_disks);

        for device in current.device_statuses {
            let name = device.device_name;
            if previous_not_disks.remove(&name) || !is_disk(&self.sys_block_root, &name) {
                self.not_disks.insert(name);
                continue;
            }
            // Checked every poll, as the media can be put in or taken out.
            if !has_media(&self.sys_block_root, &name) {
                continue;
            }

            let totals = DiskSample {
                bytes_read: device.sectors_read.saturating_mul(SECTOR_SIZE_BYTES),
                bytes_written: device.sectors_written.saturating_mul(SECTOR_SIZE_BYTES),
                io_time_ms: device.io_time_ms,
                elapsed: Duration::ZERO,
            };

            // A new disk starts from its current totals, so its first sample is 0 over no time, rather than
            // everything since boot. Its model doesn't change, so it's only read now.
            let (mut history, elapsed) = match previous_disks.remove(&name) {
                Some(history) => (history, elapsed),
                None => (
                    DiskHistory {
                        model: read_disk_model(&self.sys_block_root, &name),
                        previous: totals.clone(),
                        samples: VecDeque::with_capacity(self.max_samples),
                    },
                    Duration::ZERO,
                ),
            };

            // Saturating, as a disk that's removed and added again starts its totals from 0.
            history.samples.push_back(DiskSample {
                bytes_read: totals.bytes_read.saturating_sub(history.previous.bytes_read),
                bytes_written: totals.bytes_written.saturating_sub(history.previous.bytes_written),
                io_time_ms: totals.io_time_ms.saturating_sub(history.previous.io_time_ms),
                elapsed,
            });
            while history.samples.len() > self.max_samples {
                history.samples.pop_front();
            }
            history.previous = totals;

            let elapsed: Duration = history.samples.iter().map(|sample| sample.elapsed).sum();
            devices.push(DiskDeviceStats {
                name: name.clone(),
                model: history.model.clone(),
                is_mounted: mounted
                    .as_ref()
                    .is_none_or(|mounted| is_disk_in(&self.sys_block_root, &name, mounted)),
                bytes_read: per_second(history.samples.iter().map(|sample| sample.bytes_read).sum(), elapsed),
                bytes_written: per_second(history.samples.iter().map(|sample| sample.bytes_written).sum(), elapsed),
                busy_percent: busy_percent(history.samples.iter().map(|sample| sample.io_time_ms).sum(), elapsed),
            });

            self.disks.insert(name, history);
        }

        devices.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(DiskStats {
            avg_bytes_read: devices.iter().map(|device| device.bytes_read).sum(),
            avg_bytes_written: devices.iter().map(|device| device.bytes_written).sum(),
            devices,
        })
    }
}

/// The share of `elapsed` that the disk was busy for.
fn busy_percent(busy_ms: u64, elapsed: Duration) -> f64 {
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    if elapsed_ms <= 0.0 {
        return 0.0;
    }
    (busy_ms as f64 / elapsed_ms * 100.0).clamp(0.0, 100.0)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::sensors::proc_disk_stats_reader::ProcDiskStatsStatus;
    use std::cell::Cell;
    use std::fs;
    use tempfile::TempDir;

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

    /// The same mounts every poll.
    struct MockMountsReader(Result<Vec<MountEntry>, String>);

    impl SensorReader for MockMountsReader {
        type Output = Vec<MountEntry>;

        fn read(&self) -> Result<Self::Output, String> {
            self.0.clone()
        }
    }

    type TestMonitor = DiskMonitor<MockProcDiskStatsReader, MockMountsReader>;

    impl TestMonitor {
        /// Poll a second after the last poll, as at the default update interval.
        fn poll_next_second(&mut self) -> Result<DiskStats, String> {
            let now = self.last_poll.map_or_else(Instant::now, |last| last + Duration::from_secs(1));
            self.poll_at(now)
        }
    }

    fn make_config(max_samples: usize) -> AppConfiguration {
        let mut cfg = AppConfiguration::default();
        cfg.disk.max_samples = max_samples;
        cfg
    }

    /// A fake `/sys/block` with the disks the tests use, sda with a partition.
    fn sys_block() -> TempDir {
        let root = tempfile::tempdir().unwrap();
        for disk in ["nvme0n1", "sda", "sdb", "mmcblk0", "vda"] {
            add_disk(root.path(), disk, 1000);
        }
        fs::create_dir_all(root.path().join("sda").join("sda1")).unwrap();
        fs::write(root.path().join("sda").join("sda1").join("partition"), "1\n").unwrap();
        root
    }

    fn add_disk(root: &Path, name: &str, size: u64) {
        fs::create_dir_all(root.join(name).join("device")).unwrap();
        fs::write(root.join(name).join("size"), format!("{size}\n")).unwrap();
    }

    fn mount(source: &str) -> MountEntry {
        MountEntry {
            source: source.to_string(),
            mount_point: "/".to_string(),
            fs_type: "ext4".to_string(),
        }
    }

    /// A monitor of the disks in `root`, with nothing mounted.
    fn make_monitor(reader: MockProcDiskStatsReader, max_samples: usize, root: &TempDir) -> TestMonitor {
        DiskMonitor::new(reader, MockMountsReader(Ok(Vec::new())), &make_config(max_samples), root.path())
    }

    fn disk_status(name: &str, sectors_read: u64, sectors_written: u64, io_time_ms: u64) -> ProcDiskStatsStatus {
        ProcDiskStatsStatus {
            device_name: name.to_string(),
            sectors_read,
            sectors_written,
            io_time_ms,
        }
    }

    fn make_sensor_sample(device_name: &str, sectors_read: u64, sectors_written: u64) -> ProcDiskStats {
        ProcDiskStats::new(vec![ProcDiskStatsStatus::new(device_name.to_string(), sectors_read, sectors_written)])
    }

    #[test]
    fn first_poll_gives_zero_result() {
        let root = sys_block();
        let reader = MockProcDiskStatsReader::new(vec![Ok(make_sensor_sample("nvme0n1", 1000, 1000))]);
        let mut monitor = make_monitor(reader, 3, &root);

        let result = monitor.poll_next_second().unwrap();

        assert_eq!(result.avg_bytes_read, 0);
        assert_eq!(result.avg_bytes_written, 0);
        assert_eq!(result.devices.len(), 1);
    }

    #[test]
    fn multiple_polls_gives_expected_result() {
        let root = sys_block();
        let reader = MockProcDiskStatsReader::new(vec![
            Ok(make_sensor_sample("nvme0n1", 1000, 1000)),
            Ok(make_sensor_sample("nvme0n1", 2000, 2000)),
            Ok(make_sensor_sample("nvme0n1", 4000, 3000)),
        ]);
        let mut monitor = make_monitor(reader, 3, &root);

        _ = monitor.poll_next_second();

        let result1 = monitor.poll_next_second().unwrap();
        assert_eq!(result1.avg_bytes_read, (2000 - 1000) * 512);
        assert_eq!(result1.avg_bytes_written, (2000 - 1000) * 512);

        // Over 2 seconds.
        let result2 = monitor.poll_next_second().unwrap();
        assert_eq!(result2.avg_bytes_read, ((4000 - 1000) * 512) / 2);
        assert_eq!(result2.avg_bytes_written, ((3000 - 1000) * 512) / 2);
    }

    /// Rates are per second whatever the update interval, and however late the poll is.
    #[test]
    fn rates_are_per_second_of_elapsed_time() {
        let root = sys_block();
        let reader = MockProcDiskStatsReader::new(vec![
            Ok(ProcDiskStats::new(vec![disk_status("sda", 0, 0, 0)])),
            Ok(ProcDiskStats::new(vec![disk_status("sda", 4000, 2000, 500)])),
            Ok(ProcDiskStats::new(vec![disk_status("sda", 5000, 2500, 1000)])),
        ]);
        let mut monitor = make_monitor(reader, 1, &root);

        let start = Instant::now();
        _ = monitor.poll_at(start);
        let result = monitor.poll_at(start + Duration::from_secs(2)).unwrap();
        assert_eq!((result.avg_bytes_read, result.avg_bytes_written), (2000 * 512, 1000 * 512));
        assert_eq!(result.devices[0].busy_percent, 25.0);

        let result = monitor.poll_at(start + Duration::from_millis(2500)).unwrap();
        assert_eq!((result.avg_bytes_read, result.avg_bytes_written), (2000 * 512, 1000 * 512));
        assert_eq!(result.devices[0].busy_percent, 100.0);
    }

    #[test]
    fn samples_buffer_trims_to_max_size() {
        let root = sys_block();
        let reader = MockProcDiskStatsReader::new(vec![
            Ok(make_sensor_sample("nvme0n1", 1000, 1000)),
            Ok(make_sensor_sample("nvme0n1", 2000, 2000)),
            Ok(make_sensor_sample("nvme0n1", 3000, 3000)),
        ]);
        let mut monitor = make_monitor(reader, 2, &root);

        _ = monitor.poll_next_second();
        _ = monitor.poll_next_second();
        _ = monitor.poll_next_second();

        assert_eq!(monitor.disks["nvme0n1"].samples.len(), 2);
    }

    #[test]
    fn uses_the_disk_max_samples() {
        let mut config = AppConfiguration::default();
        config.disk.max_samples = 7;
        config.memory.max_samples = 1;

        let monitor = DiskMonitor::new(
            MockProcDiskStatsReader::new(vec![]),
            MockMountsReader(Ok(Vec::new())),
            &config,
            Path::new("/nonexistent"),
        );

        assert_eq!(monitor.max_samples, 7);
    }

    #[test]
    fn reports_each_disk_with_its_model() {
        let root = sys_block();
        fs::write(root.path().join("sda").join("device").join("model"), "Samsung SSD 870\n").unwrap();

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
        let mounts = MockMountsReader(Ok(vec![mount("/dev/nonexistent/sda1")]));
        let mut monitor = DiskMonitor::new(reader, mounts, &make_config(1), root.path());

        _ = monitor.poll_next_second();
        let result = monitor.poll_next_second().unwrap();

        assert_eq!(
            result.devices,
            [
                DiskDeviceStats {
                    name: "nvme0n1".to_string(),
                    model: None,
                    is_mounted: false,
                    bytes_read: 0,
                    bytes_written: 1000 * 512,
                    busy_percent: 100.0,
                },
                DiskDeviceStats {
                    name: "sda".to_string(),
                    model: Some("Samsung SSD 870".to_string()),
                    is_mounted: true,
                    bytes_read: 2000 * 512,
                    bytes_written: 0,
                    busy_percent: 25.0,
                },
            ]
        );
        assert_eq!(result.avg_bytes_read, 2000 * 512);
        assert_eq!(result.avg_bytes_written, 1000 * 512);
    }

    /// Without the mounts, it's unknown which disks are mounted, which isn't reported as them not being mounted.
    #[test]
    fn disks_are_mounted_when_the_mounts_are_unknown() {
        let root = sys_block();
        let reader = MockProcDiskStatsReader::new(vec![Ok(make_sensor_sample("sda", 0, 0))]);
        let mounts = MockMountsReader(Err("boom".to_string()));
        let mut monitor = DiskMonitor::new(reader, mounts, &make_config(1), root.path());

        assert!(monitor.poll_next_second().unwrap().devices[0].is_mounted);
    }

    #[test]
    fn busy_is_averaged_over_the_samples_and_capped() {
        let root = sys_block();
        let reader = MockProcDiskStatsReader::new(vec![
            Ok(ProcDiskStats::new(vec![disk_status("sda", 0, 0, 1000)])),
            Ok(ProcDiskStats::new(vec![disk_status("sda", 0, 0, 1600)])),
            Ok(ProcDiskStats::new(vec![disk_status("sda", 0, 0, 2800)])),
        ]);
        let mut monitor = make_monitor(reader, 2, &root);

        _ = monitor.poll_next_second();
        assert_eq!(monitor.poll_next_second().unwrap().devices[0].busy_percent, 60.0);
        assert_eq!(monitor.poll_next_second().unwrap().devices[0].busy_percent, 90.0);

        // The busy time can be a little more than the time between reading it.
        assert_eq!(busy_percent(1500, Duration::from_secs(1)), 100.0);
    }

    /// Unplugging a USB disk removes its totals, which mustn't make the totals go backwards.
    #[test]
    fn removed_disk_is_dropped() {
        let root = sys_block();
        let reader = MockProcDiskStatsReader::new(vec![
            Ok(ProcDiskStats::new(vec![disk_status("sda", 1000, 1000, 0), disk_status("sdb", 90000, 90000, 0)])),
            Ok(ProcDiskStats::new(vec![disk_status("sda", 2000, 1000, 0)])),
        ]);
        let mut monitor = make_monitor(reader, 1, &root);

        _ = monitor.poll_next_second();
        let result = monitor.poll_next_second().unwrap();

        assert_eq!((result.avg_bytes_read, result.avg_bytes_written), (1000 * 512, 0));
        assert_eq!(result.devices.len(), 1);
    }

    /// Partitions and virtual devices would count the disks' I/O again.
    #[test]
    fn include_only_disks() {
        let root = sys_block();
        add_disk(root.path(), "sdc", 0);
        // eMMC boot partitions are listed as disks.
        add_disk(root.path(), "mmcblk0boot0", 1000);
        fs::create_dir_all(root.path().join("dm-0")).unwrap();
        fs::create_dir_all(root.path().join("loop0")).unwrap();

        let devices = |sectors: u64| {
            let names = [
                "nvme0n1", "nvme0n1p1", "sda", "sda1", "mmcblk0", "mmcblk0p1", "mmcblk0boot0", "vda", "vda1",
                "dm-0", "loop0", "zram0", "sdc",
            ];
            ProcDiskStats::new(names.iter().map(|name| ProcDiskStatsStatus::new(name.to_string(), sectors, 0)).collect())
        };
        let reader = MockProcDiskStatsReader::new(vec![Ok(devices(1000)), Ok(devices(2000))]);
        let mut monitor = make_monitor(reader, 1, &root);

        _ = monitor.poll_next_second();
        let result = monitor.poll_next_second().unwrap();

        let names: Vec<&str> = result.devices.iter().map(|device| device.name.as_str()).collect();
        // sdc is an empty card reader.
        assert_eq!(names, ["mmcblk0", "nvme0n1", "sda", "vda"]);
        assert_eq!(result.avg_bytes_read, 4 * 1000 * 512);
    }

    #[test]
    fn error_is_propagated() {
        let root = sys_block();
        let reader = MockProcDiskStatsReader::new(vec![Err("boom".to_string())]);
        let mut monitor = make_monitor(reader, 1, &root);

        let err = monitor.poll().unwrap_err();
        assert_eq!(err, "boom");
    }
}
