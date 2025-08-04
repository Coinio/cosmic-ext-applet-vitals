use std::fs::File;
use std::io::Read;
use std::path::Path;

const MEMORY_INFO_FILE: &str = "/proc/meminfo";
const MEMORY_INFO_TOTAL_KEY: &str = "MemTotal";
const MEMORY_INFO_AVAILABLE_KEY: &str = "MemAvailable";

pub struct ProcMemInfoReader;

#[derive(Default, Clone, Debug)]
pub struct MemoryInfo {
    pub total_kibibytes: u64,
    pub used_kibibytes: u64,
}

impl MemoryInfo {
    pub fn new(total: u64, used: u64) -> Self {
        Self {
            total_kibibytes: total,
            used_kibibytes: used,
        }
    }
}

impl ProcMemInfoReader {
    pub fn new() -> Self {
        Self
    }

    pub fn get_memory_info(&self) -> Result<MemoryInfo, String> {
        let result = self.read_proc_meminfo();

        match result {
            Err(e) => Err(e),
            Ok(info) => Ok(info),
        }
    }

    fn read_proc_meminfo(&self) -> Result<MemoryInfo, String> {
        let path = Path::new(MEMORY_INFO_FILE);

        let mut file = match File::open(path) {
            Ok(file) => file,
            // TODO: Handle better, logging, etc.
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
                // TODO: Continue here or return error? We only actually care about 
                //  MemTotal/MemAvailable.
                Err(_) => continue,
                Ok((key, value)) => match key {
                    MEMORY_INFO_TOTAL_KEY => total = value,
                    MEMORY_INFO_AVAILABLE_KEY => available = value,
                    _ => ()
                },
            }
        }

        let used = total.saturating_sub(available);

        Ok(MemoryInfo::new(total, used))
    }

    fn parse_proc_file_line<'a>(&self, line: &'a str) -> Result<(&'a str, u64), String> {
        let parts: Vec<&str> = line.split(':').collect();

        if parts.len() != 2 {
            return Err(format!("Invalid line format on line: {line}"));
        }

        let value_parts: Vec<&str> = parts[1].split_whitespace().collect();

        if value_parts.len() != 2 {
            return Err(format!("Invalid line format on line: {line}"));
        }

        let key = parts[0].trim();
        let value = value_parts[0].parse().unwrap_or_default();

        Ok((key, value))
    }
}