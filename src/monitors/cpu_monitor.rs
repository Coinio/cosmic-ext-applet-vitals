use crate::sensors::proc_stat_reader::ProcStatStatus;
use crate::sensors::sensor_traits::SensorReader;
use std::collections::VecDeque;
use log::info;
use crate::configuration::app_configuration::AppConfiguration;

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
        info!("Creating new cpu monitor {:?}", configuration.cpu);
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

#[cfg(test)]
mod cpu_monitor_tests {
    use super::*;
    use std::cell::Cell;

    struct MockProcStatReader {
        // Fix set of readings to return from the reader.
        readings: Vec<Result<ProcStatStatus, String>>,
        // Tracks the index of the next reading to return.
        index: Cell<usize>,
    }

    impl MockProcStatReader {
        fn new(readings: Vec<Result<ProcStatStatus, String>>) -> Self {
            Self {
                readings,
                index: Cell::new(0),
            }
        }
    }

    impl SensorReader for MockProcStatReader {
        type Output = ProcStatStatus;

        fn read(&self) -> Result<Self::Output, String> {
            let read_index = self.index.get();
            let reading = self.readings[read_index].clone();

            self.index.set(read_index + 1);
            reading
        }
    }

    fn make_config(max_samples: usize) -> AppConfiguration {
        let mut cfg = AppConfiguration::default();
        cfg.cpu.max_samples = max_samples;
        cfg
    }

    #[test]
    fn single_poll_computes_expected_usage() {

        let reader = MockProcStatReader::new(vec![Ok(ProcStatStatus::new(100, 50, 500))]);
        let mut monitor = CpuMonitor::new(reader, &make_config(4));

        let stats = monitor.poll().expect("poll should succeed");

        assert!(eq_to_three_decimal_places(stats.cpu_usage_percent, 70.000));
    }

    #[test]
    fn multiple_polls_average_and_buffering() {
        let reader = MockProcStatReader::new(vec![
            Ok(ProcStatStatus::new(100, 50, 500)),
            Ok(ProcStatStatus::new(200, 60, 700)),
        ]);
        let mut monitor = CpuMonitor::new(reader, &make_config(4));

        let reading1 = monitor.poll().unwrap();
        assert!(eq_to_three_decimal_places(reading1.cpu_usage_percent, 70.000));

        let reading2 = monitor.poll().unwrap();
        assert!(eq_to_three_decimal_places(reading2.cpu_usage_percent, 57.500));
    }

    #[test]
    fn sample_buffer_trims_to_max_samples() {
        let reader = MockProcStatReader::new(vec![
            Ok(ProcStatStatus::new(0, 0, 100)),
            Ok(ProcStatStatus::new(50, 0, 200)),
            Ok(ProcStatStatus::new(100, 0, 300)),
        ]);
        let mut monitor = CpuMonitor::new(reader, &make_config(2));

        // Throw away first reading, now buffer contains [50,100]
        let _ = monitor.poll().unwrap();
                 
        let reading2 = monitor.poll().unwrap();
        assert!(eq_to_three_decimal_places(reading2.cpu_usage_percent, 75.000));

        let reading3 = monitor.poll().unwrap(); 
        assert!(eq_to_three_decimal_places(reading3.cpu_usage_percent, 50.000));
    }

    #[test]
    fn error_is_propagated() {
        let reader = MockProcStatReader::new(vec![Err("boom".to_string())]);
        let mut monitor = CpuMonitor::new(reader, &make_config(1));

        let err = monitor.poll().unwrap_err();
        assert_eq!(err, "boom");
    }

    fn eq_to_three_decimal_places(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.0005
    }
}