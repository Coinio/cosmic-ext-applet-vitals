use std::fs::File;
use std::io::Read;
use std::path::Path;

const PROC_STAT_FILE: &str = "/proc/stat";
const CPU_LINE_PREFIX: &str = "cpu ";

const PROC_STAT_IDLE_INDEX: usize = 3;

#[derive(Default, Clone, Debug)]
pub struct CpuStats {
    pub cpu_usage_percent: f64,
}

impl CpuStats {
    pub fn new(cpu_usage_percent: f64) -> Self {
        Self { cpu_usage_percent }
    }
}

#[derive(Default, Debug)]
pub struct ProcStatReader {
    previous_idle: u64,
    previous_total: u64,
}

impl ProcStatReader {
    pub fn new() -> Self {
        ProcStatReader::default()
    }

    pub fn read_cpu_stats(&mut self) -> Result<CpuStats, String> {
        let path = Path::new(PROC_STAT_FILE);

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(e) => return Err(format!("Unable to open file: {} {}", PROC_STAT_FILE, e)),
        };

        let mut contents = String::new();

        if let Err(_) = file.read_to_string(&mut contents) {
            return Err(format!("Unable to read {}", PROC_STAT_FILE));
        }

        let first_line = contents.lines().next().unwrap_or("");

        self.parse_cpu_stats_line(first_line)
    }

    fn parse_cpu_stats_line(&mut self, line: &str) -> Result<CpuStats, String> {
        let values: Vec<u64> = line
            .strip_prefix(CPU_LINE_PREFIX)
            .unwrap_or(line)
            .split_whitespace()
            .map(|v| v.parse::<u64>().unwrap_or_default())
            .collect();

        let total = values.iter().sum();
        let idle = match values.get(PROC_STAT_IDLE_INDEX) {
            Some(&value) => value,
            None => return Err(format!("{PROC_STAT_FILE} is not in a valid format.")),
        };

        let cpu_usage_percent: f64 = 100.0
            * (1.0 - (idle - self.previous_idle) as f64 / (total - self.previous_total) as f64);

        self.previous_idle = idle;
        self.previous_total = total;

        Ok(CpuStats::new(cpu_usage_percent))
    }
}
