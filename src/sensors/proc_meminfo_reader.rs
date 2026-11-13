use crate::sensors::proc_file;
use crate::sensors::sensor_traits::SensorReader;

const MEMORY_INFO_FILE: &str = "/proc/meminfo";
const MEMORY_INFO_TOTAL_KEY: &str = "MemTotal";
const MEMORY_INFO_AVAILABLE_KEY: &str = "MemAvailable";
const MEMORY_INFO_BUFFERS_KEY: &str = "Buffers";
const MEMORY_INFO_CACHED_KEY: &str = "Cached";
const MEMORY_INFO_SRECLAIMABLE_KEY: &str = "SReclaimable";
const MEMORY_INFO_SWAP_TOTAL_KEY: &str = "SwapTotal";
const MEMORY_INFO_SWAP_FREE_KEY: &str = "SwapFree";

#[derive(Clone, Debug, Default)]
pub struct ProcMemInfoStatus {
    pub total_kib: u64,
    pub available_kib: u64,
    /// The page cache, buffers and reclaimable kernel caches, which `free` reports as buff/cache.
    pub cached_kib: u64,
    pub swap_total_kib: u64,
    pub swap_free_kib: u64,
}

impl ProcMemInfoStatus {
    #[cfg(test)]
    pub fn new (total_kib: u64, available_kib: u64) -> Self {
        Self {
            total_kib,
            available_kib,
            ..Default::default()
        }
    }
}

pub struct ProcMemInfoSensorReader;

impl SensorReader for ProcMemInfoSensorReader {
    type Output = ProcMemInfoStatus;

    fn read(&self) -> Result<ProcMemInfoStatus, String> {
        proc_file::read(MEMORY_INFO_FILE, |contents| self.parse(contents))
            .map_err(|e| format!("Unable to read {}: {}", MEMORY_INFO_FILE, e))?
    }
}

impl ProcMemInfoSensorReader {
    fn parse(&self, contents: &str) -> Result<ProcMemInfoStatus, String> {
        let mut status = ProcMemInfoStatus::default();

        for line in contents.lines() {
            let result = self.parse_proc_file_line(line);

            match result {
                Err(e) => return Err(format!("Invalid file format: {} {}", MEMORY_INFO_FILE, e)),
                Ok((key, value)) => match key {
                    MEMORY_INFO_TOTAL_KEY => status.total_kib = value,
                    MEMORY_INFO_AVAILABLE_KEY => status.available_kib = value,
                    MEMORY_INFO_BUFFERS_KEY | MEMORY_INFO_CACHED_KEY | MEMORY_INFO_SRECLAIMABLE_KEY => {
                        status.cached_kib += value
                    }
                    MEMORY_INFO_SWAP_TOTAL_KEY => status.swap_total_kib = value,
                    MEMORY_INFO_SWAP_FREE_KEY => status.swap_free_kib = value,
                    _ => ()
                },
            }
        }

        Ok(status)
    }

    fn parse_proc_file_line<'a>(&self, line: &'a str) -> Result<(&'a str, u64), String> {
        let parts: Vec<&str> = line.split(':').collect();

        if parts.len() != 2 {
            return Err(format!("Invalid line format on line: {line}"));
        }

        let value_parts: Vec<&str> = parts[1].split_whitespace().collect();

        if value_parts.len() < 1 {
            return Err(format!("Invalid line format on line: {line}"));
        }

        let key = parts[0].trim();
        let value = value_parts[0].parse().unwrap_or_default();

        Ok((key, value))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    const MEMINFO: &str = "\
MemTotal:       31934904 kB
MemFree:         1234567 kB
MemAvailable:   22048124 kB
Buffers:          400000 kB
Cached:          8000000 kB
SwapCached:        10000 kB
Shmem:            500000 kB
SReclaimable:     600000 kB
SUnreclaim:       200000 kB
SwapTotal:       8388604 kB
SwapFree:        8000000 kB
";

    #[test]
    fn reads_memory_cache_and_swap() {
        let status = ProcMemInfoSensorReader.parse(MEMINFO).unwrap();

        assert_eq!(status.total_kib, 31934904);
        assert_eq!(status.available_kib, 22048124);
        // Buffers + Cached + SReclaimable, but not SwapCached.
        assert_eq!(status.cached_kib, 400000 + 8000000 + 600000);
        assert_eq!(status.swap_total_kib, 8388604);
        assert_eq!(status.swap_free_kib, 8000000);
    }

    #[test]
    fn missing_swap_and_cache_read_as_zero() {
        let status = ProcMemInfoSensorReader.parse("MemTotal: 1000 kB\nMemAvailable: 500 kB\n").unwrap();

        assert_eq!(status.swap_total_kib, 0);
        assert_eq!(status.cached_kib, 0);
    }
}
