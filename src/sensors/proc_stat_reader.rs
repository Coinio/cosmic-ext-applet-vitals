use crate::sensors::proc_file;
use crate::sensors::sensor_traits::SensorReader;

const PROC_STAT_FILE: &str = "/proc/stat";
const CPU_LINE_PREFIX: &str = "cpu";

const PROC_STAT_IDLE_INDEX: usize = 3;
const PROC_STAT_IOWAIT_INDEX: usize = 4;

/// The time a CPU, or all of them, has spent in each state since boot, in clock ticks.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CpuTimes {
    pub idle: u64,
    pub iowait: u64,
    pub total: u64,
}

impl CpuTimes {
    pub fn new(idle: u64, iowait: u64, total: u64) -> Self {
        CpuTimes { idle, iowait, total }
    }
}

impl std::ops::Add for CpuTimes {
    type Output = CpuTimes;

    fn add(self, other: CpuTimes) -> CpuTimes {
        CpuTimes::new(
            self.idle.saturating_add(other.idle),
            self.iowait.saturating_add(other.iowait),
            self.total.saturating_add(other.total),
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ProcStatStatus {
    /// The times across every CPU.
    pub all: CpuTimes,
    /// The times of each online CPU, i.e. each hardware thread, by its kernel number, in order.
    pub cpus: Vec<(usize, CpuTimes)>,
}

impl ProcStatStatus {
    #[cfg(test)]
    pub fn new(idle: u64, iowait: u64, total: u64) -> Self {
        ProcStatStatus {
            all: CpuTimes::new(idle, iowait, total),
            cpus: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct ProcStatSensorReader;

impl SensorReader for ProcStatSensorReader {
    type Output = ProcStatStatus;

    fn read(&self) -> Result<ProcStatStatus, String> {
        proc_file::read(PROC_STAT_FILE, parse_proc_stat)
            .map_err(|e| format!("Unable to read file: {} {}", PROC_STAT_FILE, e))?
    }
}

/// Parse the `cpu` line, which totals every CPU, and the `cpuN` line of each online CPU.
fn parse_proc_stat(contents: &str) -> Result<ProcStatStatus, String> {
    let mut all = None;
    let mut cpus = Vec::new();

    // The CPU lines come first, so stop at the first line that isn't one.
    for line in contents.lines().take_while(|line| line.starts_with(CPU_LINE_PREFIX)) {
        let mut fields = line.split_whitespace();
        let name = fields.next().unwrap_or_default();
        let times = parse_cpu_times(fields)?;

        match &name[CPU_LINE_PREFIX.len()..] {
            "" => all = Some(times),
            number => match number.parse::<usize>() {
                Ok(number) => cpus.push((number, times)),
                Err(_) => return Err(format!("{PROC_STAT_FILE} has an unknown CPU line: {name}")),
            },
        }
    }

    let all = all.ok_or_else(|| format!("{PROC_STAT_FILE} is not in a valid format."))?;
    Ok(ProcStatStatus { all, cpus })
}

fn parse_cpu_times<'a>(fields: impl Iterator<Item = &'a str>) -> Result<CpuTimes, String> {
    let values: Vec<u64> = fields.map(|v| v.parse::<u64>().unwrap_or_default()).collect();

    let total = values.iter().sum();

    let idle = match values.get(PROC_STAT_IDLE_INDEX) {
        Some(&value) => value,
        None => return Err(format!("{PROC_STAT_FILE} is not in a valid format.")),
    };

    let iowait = match values.get(PROC_STAT_IOWAIT_INDEX) {
        Some(&value) => value,
        None => return Err(format!("{PROC_STAT_FILE} is not in a valid format.")),
    };

    Ok(CpuTimes::new(idle, iowait, total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_total_and_each_cpu() {
        let status = parse_proc_stat(
            "cpu  10 0 10 70 10 0 0 0 0 0
cpu0 5 0 5 30 10 0 0 0 0 0
cpu2 5 0 5 40 0 0 0 0 0 0
intr 12345 0 0
ctxt 6789
",
        )
        .expect("Should parse");

        assert_eq!(status.all, CpuTimes::new(70, 10, 100));
        assert_eq!(
            status.cpus,
            vec![(0, CpuTimes::new(30, 10, 50)), (2, CpuTimes::new(40, 0, 50))]
        );
    }

    #[test]
    fn a_missing_total_is_an_error() {
        assert!(parse_proc_stat("intr 12345 0 0\n").is_err());
    }

    #[test]
    fn a_short_cpu_line_is_an_error() {
        assert!(parse_proc_stat("cpu  10 0 10\n").is_err());
    }
}
