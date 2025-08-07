use crate::sensors::proc_stat_reader::{ProcStatStatus};
use crate::sensors::sensor_traits::SensorReader;

#[derive(Default, Clone, Debug)]
pub struct CpuStats {
    pub cpu_usage_percent: f64,
}

impl CpuStats {
    pub fn new(cpu_usage_percent: f64) -> Self {
        Self { cpu_usage_percent }
    }
}

pub struct CpuMonitor<S: SensorReader<Output = ProcStatStatus>> {
    sensor_reader: S,
    previous_idle: u64,
    previous_total: u64,
}

impl<S: SensorReader<Output = ProcStatStatus>> CpuMonitor<S> {
    pub fn new(sensor_reader: S) -> Self {
        Self {
            sensor_reader,
            previous_idle: 0,
            previous_total: 0,
        }
    }
    
    pub fn update(&mut self) -> Result<CpuStats, String> {
        let current = match self.sensor_reader.read() {
            Ok(cpu_stats) => cpu_stats,
            Err(err) => return Err(err),
        };

        let cpu_usage_percent: f64 = 100.0
            * (1.0 - (current.idle - self.previous_idle) as f64 / (current.total - self.previous_total)
            as f64);

        self.previous_idle = current.idle;
        self.previous_total = current.total;

        Ok(CpuStats::new(cpu_usage_percent))
    }
}
