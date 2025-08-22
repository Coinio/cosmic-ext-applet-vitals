use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::sensors::sensor_traits::SensorReader;

const PROC_STAT_FILE: &str = "/proc/stat";
const CPU_LINE_PREFIX: &str = "cpu ";

const PROC_STAT_IDLE_INDEX: usize = 3;
const PROC_STAT_IOWAIT_INDEX: usize = 4;


#[derive(Default, Clone)]
pub struct ProcStatStatus {
    pub idle: u64,
    pub iowait: u64,
    pub total: u64,
}

impl ProcStatStatus {
    pub fn new(idle: u64, iowait: u64, total: u64) -> Self {
        ProcStatStatus { idle, iowait, total }
    }
}

#[derive(Default)]
pub struct ProcStatSensorReader;

impl SensorReader for ProcStatSensorReader {
    type Output = ProcStatStatus;

    fn read(&self) -> Result<ProcStatStatus, String> {
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
}

impl ProcStatSensorReader {

    fn parse_cpu_stats_line(&self, line: &str) -> Result<ProcStatStatus, String> {
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

        let iowait = match values.get(PROC_STAT_IOWAIT_INDEX) {
            Some(&value) => value,
            None => return Err(format!("{PROC_STAT_FILE} is not in a valid format.")),
        };

        Ok(ProcStatStatus::new(idle, iowait, total))
    }
}
