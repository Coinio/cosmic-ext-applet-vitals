use crate::core::app_configuration::AppConfiguration;
use crate::sensors::proc_stat_reader::ProcStatStatus;
use crate::sensors::sensor_traits::SensorReader;
use std::collections::VecDeque;

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
    sample_buffer: VecDeque<f64>,
    max_samples: usize,
}

impl<S: SensorReader<Output = ProcStatStatus>> CpuMonitor<S> {
    pub fn new(sensor_reader: S, configuration: &AppConfiguration) -> Self {
        Self {
            sensor_reader,
            previous_idle: 0,
            previous_total: 0,
            sample_buffer: VecDeque::with_capacity(configuration.cpu.max_samples),
            max_samples: configuration.cpu.max_samples
        }
    }
    
    pub fn poll(&mut self) -> Result<CpuStats, String> {
        let current = match self.sensor_reader.read() {
            Ok(cpu_stats) => cpu_stats,
            Err(err) => return Err(err),
        };

        let total_idle = current.idle + current.iowait;

        let current_usage_percent: f64 = 100.0
            * (1.0 - (total_idle - self.previous_idle) as f64 / (current.total - self.previous_total)
            as f64);

        self.sample_buffer.push_back(current_usage_percent);

        self.previous_idle = total_idle;
        self.previous_total = current.total;

        if self.sample_buffer.len() > self.max_samples {
            self.sample_buffer.pop_front();
        }

        let average_cpu_usage = self.sample_buffer.iter().sum::<f64>() / self.sample_buffer.len() as f64;

        Ok(CpuStats::new(average_cpu_usage))
    }
}
