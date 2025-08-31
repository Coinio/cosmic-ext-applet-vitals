use crate::sensors::sensor_traits::SensorReader;
use std::fs::File;
use std::io::Read;
use std::path::Path;

const PROC_DISK_STATS_FILE: &str = "/proc/diskstats";

const DISK_NAME_INDEX: usize = 2;
const SECTORS_READ_INDEX: usize = 5;
const SECTORS_WRITTEN_INDEX: usize = 9;

#[derive(Clone, Debug, Default)]
pub struct ProcDiskStatsStatus {
    pub device_name: String,
    pub sectors_read: u64,
    pub sectors_written: u64,
}

impl ProcDiskStatsStatus {
    pub fn new(device_name: String, sectors_read: u64, sectors_written: u64) -> Self {
        Self {
            device_name,
            sectors_read,
            sectors_written,
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

        Ok(ProcDiskStatsStatus::new(device_name, sectors_read, sectors_written))
    }
}
