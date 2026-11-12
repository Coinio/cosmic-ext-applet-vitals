use crate::sensors::sensor_traits::SensorReader;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

const PROC_DISK_STATS_FILE: &str = "/proc/diskstats";

/// Where the kernel lists block devices, including each disk's model.
pub const SYS_BLOCK_ROOT: &str = "/sys/block";

const DISK_NAME_INDEX: usize = 2;
const SECTORS_READ_INDEX: usize = 5;
const SECTORS_WRITTEN_INDEX: usize = 9;
const IO_TIME_MS_INDEX: usize = 12;

#[derive(Clone, Debug, Default)]
pub struct ProcDiskStatsStatus {
    pub device_name: String,
    pub sectors_read: u64,
    pub sectors_written: u64,
    /// The milliseconds the device has spent with I/O in flight, which is how long it's been busy.
    pub io_time_ms: u64,
}

impl ProcDiskStatsStatus {
    #[cfg(test)]
    pub fn new(device_name: String, sectors_read: u64, sectors_written: u64) -> Self {
        Self {
            device_name,
            sectors_read,
            sectors_written,
            io_time_ms: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcDiskStats {
    pub device_statuses: Vec<ProcDiskStatsStatus>
}

impl ProcDiskStats {
    pub fn new(device_statuses: Vec<ProcDiskStatsStatus>) -> Self {
        Self {
            device_statuses
        }
    }
}

#[derive(Default, Debug)]
pub struct ProcDiskStatsReader;

impl SensorReader for ProcDiskStatsReader {
    type Output = ProcDiskStats;

    fn read(&self) -> Result<Self::Output, String> {
        let path = Path::new(PROC_DISK_STATS_FILE);

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(e) => return Err(format!("Unable to open file: {} {}", PROC_DISK_STATS_FILE, e)),
        };

        let mut contents = String::new();

        if let Err(_) = file.read_to_string(&mut contents) {
            return Err(format!("Unable to read {}", PROC_DISK_STATS_FILE));
        }

        let mut statuses = Vec::new();

        for line in contents.lines() {
            if line.trim().is_empty() { continue; }
            let device_status = self.parse_disk_stats_line(line)?;

            statuses.push(device_status);
        }

        Ok(ProcDiskStats::new(statuses))
    }
}

impl ProcDiskStatsReader {
    fn parse_disk_stats_line(&self, line: &str) -> Result<ProcDiskStatsStatus, String> {
        let values: Vec<&str> = line
            .split_whitespace()
            .collect();

        if values.len() < 20 {
            return Err(format!("Invalid file format {}", PROC_DISK_STATS_FILE));
        }

        let device_name = values[DISK_NAME_INDEX].to_string();
        let sectors_read = values[SECTORS_READ_INDEX].parse::<u64>().unwrap_or_default();
        let sectors_written = values[SECTORS_WRITTEN_INDEX].parse::<u64>().unwrap_or_default();
        let io_time_ms = values[IO_TIME_MS_INDEX].parse::<u64>().unwrap_or_default();

        Ok(ProcDiskStatsStatus {
            device_name,
            sectors_read,
            sectors_written,
            io_time_ms,
        })
    }
}

/// The disk's model, e.g. "Samsung SSD 980 1TB", where the driver reports one. SATA drivers pad the vendor and model
/// to fixed widths, so runs of spaces are collapsed.
pub fn read_disk_model(sys_block_root: &Path, device_name: &str) -> Option<String> {
    let model = fs::read_to_string(sys_block_root.join(device_name).join("device").join("model")).ok()?;
    let model = model.split_whitespace().collect::<Vec<_>>().join(" ");
    (!model.is_empty()).then_some(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_disk_stats_line() {
        let line = " 259       0 nvme0n1 120 5 3000 40 200 7 4000 90 0 1234 130 0 0 0 0 10 2";
        let status = ProcDiskStatsReader.parse_disk_stats_line(line).unwrap();

        assert_eq!(status.device_name, "nvme0n1");
        assert_eq!(status.sectors_read, 3000);
        assert_eq!(status.sectors_written, 4000);
        assert_eq!(status.io_time_ms, 1234);
    }

    #[test]
    fn reads_the_trimmed_model() {
        let root = tempfile::tempdir().unwrap();
        let device = root.path().join("sda").join("device");
        fs::create_dir_all(&device).unwrap();
        fs::write(device.join("model"), "WDC  WDS100T2B0A    \n").unwrap();
        fs::create_dir_all(root.path().join("nvme0n1").join("device")).unwrap();
        fs::write(root.path().join("nvme0n1").join("device").join("model"), "  \n").unwrap();

        assert_eq!(read_disk_model(root.path(), "sda").as_deref(), Some("WDC WDS100T2B0A"));
        assert_eq!(read_disk_model(root.path(), "nvme0n1"), None);
        assert_eq!(read_disk_model(root.path(), "sdb"), None);
    }
}
