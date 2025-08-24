use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::sensors::sensor_traits::SensorReader;

const MEMORY_INFO_FILE: &str = "/proc/meminfo";
const MEMORY_INFO_TOTAL_KEY: &str = "MemTotal";
const MEMORY_INFO_AVAILABLE_KEY: &str = "MemAvailable";

#[derive(Clone)]
pub struct ProcMemInfoStatus {
    pub total_kib: u64,
    pub available_kib: u64,
}

impl ProcMemInfoStatus {
    pub fn new (total_kib: u64, available_kib: u64) -> Self {
        Self {
            total_kib,
            available_kib,
        }
    }   
}

pub struct ProcMemInfoSensorReader;

impl SensorReader for ProcMemInfoSensorReader {
    type Output = ProcMemInfoStatus;

    fn read(&self) -> Result<ProcMemInfoStatus, String> {
        let path = Path::new(MEMORY_INFO_FILE);

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(e) => return Err(format!("Unable to open file: {} {}", MEMORY_INFO_FILE, e)),
        };

        let mut contents = String::new();

        if let Err(_) = file.read_to_string(&mut contents) {
            return Err(format!("Unable to read {}", MEMORY_INFO_FILE))
        }

        let mut total = 0;
        let mut available = 0;

        for line in contents.lines() {
            let result = self.parse_proc_file_line(line);

            match result {
                Err(e) => return Err(format!("Invalid file format: {} {}", MEMORY_INFO_FILE, e)),
                Ok((key, value)) => match key {
                    MEMORY_INFO_TOTAL_KEY => total = value,
                    MEMORY_INFO_AVAILABLE_KEY => available = value,
                    _ => ()
                },
            }
        }

        Ok(ProcMemInfoStatus::new(total, available))
    }
}

impl ProcMemInfoSensorReader {

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