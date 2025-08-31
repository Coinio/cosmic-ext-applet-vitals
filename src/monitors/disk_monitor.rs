use crate::configuration::app_configuration::AppConfiguration;
use crate::sensors::proc_disk_stats_reader::ProcDiskStats;
use crate::sensors::sensor_traits::SensorReader;
use log::info;
use std::cmp;
use std::collections::VecDeque;

pub const DISK_STAT_READ_INDEX: usize = 0;
pub const DISK_STAT_WRITE_INDEX: usize = 1;

#[derive(Debug, Clone, Default)]
struct DiskSample {
    pub reads: u64,
    pub writes: u64,
}

impl DiskSample {
    pub fn new(reads: u64, writes: u64) -> Self {
        Self { reads, writes }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiskDirection {
    Read,
    Write,
}

impl Default for DiskDirection {
    fn default() -> Self {
        Self::Read
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiskStats {
    pub ops: u64,
    pub direction: DiskDirection,
}

pub struct DiskMonitor<S: SensorReader<Output = ProcDiskStats>> {
    sensor_reader: S,
    sample_buffer: VecDeque<DiskSample>,
    previous_reads: u64,
    previous_writes: u64,
    max_samples: usize,
}

impl<S: SensorReader<Output = ProcDiskStats>> DiskMonitor<S> {
    pub fn new(sensor_reader: S, configuration: &AppConfiguration) -> Self {
        info!("Creating new disk monitor {:?}", configuration);
        Self {
            sensor_reader,
            sample_buffer: VecDeque::with_capacity(configuration.memory.max_samples),
            previous_reads: 0,
            previous_writes: 0,
            max_samples: configuration.memory.max_samples,
        }
    }

    pub fn sample_buffer_len(&self) -> usize {
        self.sample_buffer.len()
    }

    pub fn poll(&mut self) -> Result<[DiskStats; 2], String> {
        let current = match self.sensor_reader.read() {
            Ok(value) => value,
            Err(err) => return Err(err),
        };

        let mut current_read_total = 0u64;
        let mut current_write_total = 0u64;

        for device_status in current.device_statuses {
            if !is_logical_disk(&device_status.device_name) {
                continue;
            }

            current_read_total += device_status.reads_completed;
            current_write_total += device_status.writes_completed;
        }

        if (self.previous_reads == 0) && (self.previous_writes == 0) {
            self.previous_reads = current_read_total;
            self.previous_writes = current_write_total;
        }

        let delta_reads = cmp::max(current_read_total - self.previous_reads, 0);
        let delta_writes = cmp::max(current_write_total - self.previous_writes, 0);

        self.previous_reads = current_read_total;
        self.previous_writes = current_write_total;

        self.sample_buffer.push_back(DiskSample::new(delta_reads, delta_writes));
        if self.sample_buffer.len() > self.max_samples {
            self.sample_buffer.pop_front();
        }

        let avg_reads = self.sample_buffer.iter().map(|s| s.reads).sum::<u64>() / self.sample_buffer.len() as u64;
        let avg_writes = self.sample_buffer.iter().map(|s| s.writes).sum::<u64>() / self.sample_buffer.len() as u64;

        Ok([
            DiskStats {
                ops: avg_reads,
                direction: DiskDirection::Read,
            },
            DiskStats {
                ops: avg_writes,
                direction: DiskDirection::Write,
            },
        ])
    }
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
    use crate::monitors::network_monitor::NetworkMonitor;

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
        cfg.memory.max_samples = max_samples;
        cfg
    }

    fn make_sensor_sample(device_name: String, reads: u64, writes: u64) -> ProcDiskStats {
        ProcDiskStats::new(vec![ProcDiskStatsStatus::new(device_name, reads, writes)])
    }

    #[test]
    fn first_poll_gives_zero_result() {
        let statuses = make_sensor_sample("nvme0n1".to_string(), 1000, 1000);

        let reader = MockProcDiskStatsReader::new(vec![Ok(statuses)]);

        let mut monitor = DiskMonitor::new(reader, &make_config(3));

        let result = monitor.poll();

        assert!(result.is_ok());

        let result = result.unwrap();

        assert_eq!(result[DISK_STAT_READ_INDEX].ops, 0);
        assert_eq!(result[DISK_STAT_WRITE_INDEX].ops, 0);
    }

    #[test]
    fn second_poll_gives_expected_result() {
        let sample1 = make_sensor_sample("nvme0n1".to_string(), 1000, 1000);
        let sample2 = make_sensor_sample("nvme0n1".to_string(), 2000, 2000);

        let reader = MockProcDiskStatsReader::new(vec![Ok(sample1), Ok(sample2)]);

        let mut monitor = DiskMonitor::new(reader, &make_config(3));

        // Throw away first result, as will always be 0
        _ = monitor.poll();
        let result = monitor.poll();

        assert!(result.is_ok());

        let result = result.unwrap();

        assert_eq!(result[DISK_STAT_READ_INDEX].ops, (2000 - 1000) / 2);
        assert_eq!(result[DISK_STAT_WRITE_INDEX].ops, (2000 - 1000) / 2);
    }

    #[test]
    fn multiple_polls_gives_expected_result() {
        let sample1 = make_sensor_sample("nvme0n1".to_string(), 1000, 1000);
        let sample2 = make_sensor_sample("nvme0n1".to_string(), 2000, 2000);
        let sample3 = make_sensor_sample("nvme0n1".to_string(), 3000, 3000);

        let reader = MockProcDiskStatsReader::new(vec![Ok(sample1), Ok(sample2), Ok(sample3)]);

        let mut monitor = DiskMonitor::new(reader, &make_config(3));

        // Throw away first result, as will always be 0
        _ = monitor.poll();

        let result1 = monitor.poll();
        let result2 = monitor.poll();

        assert!(result1.is_ok());

        let result1 = result1.unwrap();

        assert_eq!(result1[DISK_STAT_READ_INDEX].ops, (2000 - 1000) / 2);
        assert_eq!(result1[DISK_STAT_WRITE_INDEX].ops, (2000 - 1000) / 2);

        assert!(result2.is_ok());

        let result2 = result2.unwrap();

        assert_eq!(result2[DISK_STAT_READ_INDEX].ops, (3000 - 1000) / 3);
        assert_eq!(result2[DISK_STAT_WRITE_INDEX].ops, (3000 - 1000) / 3);
    }

    #[test]
    fn samples_buffer_trims_to_max_size() {
        let sample1 = make_sensor_sample("nvme0n1".to_string(), 1000, 1000);
        let sample2 = make_sensor_sample("nvme0n1".to_string(), 2000, 2000);
        let sample3 = make_sensor_sample("nvme0n1".to_string(), 3000, 3000);

        let reader = MockProcDiskStatsReader::new(vec![Ok(sample1), Ok(sample2), Ok(sample3)]);

        let mut monitor = DiskMonitor::new(reader, &make_config(2));

        // Throw away first result, as will always be 0
        _ = monitor.poll();
        _ = monitor.poll();
        _ = monitor.poll();

        assert!(monitor.sample_buffer_len() == 2);
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

        let mut monitor = DiskMonitor::new(reader, &make_config(2));

        // Throw away first result, as will always be 0
        _ = monitor.poll();

        let result = monitor.poll();

        assert!(result.is_ok());

        let result = result.unwrap();

        assert_eq!(result[DISK_STAT_READ_INDEX].ops, ((2000 + 2000) - (1000 + 1000)) / 2);
        assert_eq!(result[DISK_STAT_WRITE_INDEX].ops, ((2000 + 2000) - (1000 + 1000)) / 2);
    }

    #[test]
    fn error_is_propagated() {
        let reader = MockProcDiskStatsReader::new(vec![Err("boom".to_string())]);
        let mut monitor = DiskMonitor::new(reader, &make_config(1));

        let err = monitor.poll().unwrap_err();
        assert_eq!(err, "boom");
    }
}
