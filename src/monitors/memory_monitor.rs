use crate::sensors::proc_meminfo_reader::ProcMemInfoStatus;
use crate::sensors::sensor_traits::SensorReader;
use std::collections::VecDeque;
use log::info;
use crate::configuration::app_configuration::AppConfiguration;

#[derive(Default, Clone, Debug)]
pub struct MemoryStats {
    pub total_kib: u64,
    pub used_kib: u64,
}

impl MemoryStats {
    pub fn new(total_kib: u64, used_kib: u64) -> Self {
        Self {
            total_kib,
            used_kib,
        }
    }
}

pub struct MemoryMonitor<S: SensorReader<Output = ProcMemInfoStatus>> {
    sensor_reader: S,
    sample_buffer: VecDeque<u64>,
    max_samples: usize,
}

impl<S: SensorReader<Output = ProcMemInfoStatus>> MemoryMonitor<S> {
    pub fn new(sensor_reader: S, configuration: &AppConfiguration) -> Self {
        info!("Creating new memory monitor {:?}", configuration.memory);
        Self {
            sensor_reader,
            sample_buffer: VecDeque::with_capacity(configuration.memory.max_samples),
            max_samples: configuration.memory.max_samples
        }
    }
    
    pub fn poll(&mut self) -> Result<MemoryStats, String> {
        let meminfo_state = match self.sensor_reader.read() {
            Ok(state) => state,
            Err(err) => return Err(err),
        };

        let current_used = meminfo_state.total_kib.saturating_sub(meminfo_state.available_kib);
        
        self.sample_buffer.push_back(current_used);
        
        if self.sample_buffer.len() > self.max_samples {
            self.sample_buffer.pop_front();       
        }
        
        let average_used = self.sample_buffer.iter().sum::<u64>() / self.sample_buffer.len() as u64;

        Ok(MemoryStats::new(meminfo_state.total_kib, average_used))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use crate::monitors::cpu_monitor::CpuMonitor;
    use super::*;

    const TOTAL_KIB: u64 = 31934904;

    struct MockProcMeminfoReader {
        readings: Vec<Result<ProcMemInfoStatus, String>>,
        index: Cell<usize>
    }

    impl MockProcMeminfoReader {
        fn new(readings: Vec<Result<ProcMemInfoStatus, String>>) -> MockProcMeminfoReader {
            Self {
                readings,
                index: Cell::new(0),
            }
        }
    }

    impl SensorReader for MockProcMeminfoReader {
        type Output = ProcMemInfoStatus;

        fn read(&self) -> Result<Self::Output, String> {
            let index = self.index.get();
            let result = self.readings[index].clone();
            self.index.set(index + 1);
            result
        }
    }

    fn make_config(max_samples: usize) -> AppConfiguration {
        let mut cfg = AppConfiguration::default();
        cfg.memory.max_samples = max_samples;
        cfg
    }

    #[test]
    fn single_poll_gives_expected_result() {

        const AVAILABLE_KIB: u64 = 22048124;

        let mock_memory_reader = MockProcMeminfoReader::new(vec![
            Ok(ProcMemInfoStatus::new(TOTAL_KIB, AVAILABLE_KIB)),
        ]);

        let mut monitor = MemoryMonitor::new(mock_memory_reader, &AppConfiguration::default());
        let result = monitor.poll();

        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result.used_kib, TOTAL_KIB - AVAILABLE_KIB);
        assert_eq!(result.total_kib, TOTAL_KIB);
    }

    #[test]
    fn multiple_poll_gives_expected_result() {

        let reading1 = 22048124;
        let reading2 = 21662400;
        let reading3 = 23576212;

        let mock_memory_reader = MockProcMeminfoReader::new(vec![
           Ok(ProcMemInfoStatus::new(TOTAL_KIB, reading1)),
           Ok(ProcMemInfoStatus::new(TOTAL_KIB, reading2)),
           Ok(ProcMemInfoStatus::new(TOTAL_KIB, reading3))
        ]);

        let mut monitor = MemoryMonitor::new(mock_memory_reader, &make_config(3));

        let result1 = monitor.poll();
        let result2 = monitor.poll();
        let result3 = monitor.poll();

        assert!(result1.is_ok());
        let result1 = result1.unwrap();
        // Only one reading in the samples buffer
        assert_eq!(result1.used_kib, TOTAL_KIB - reading1);
        assert_eq!(result1.total_kib, TOTAL_KIB);

        assert!(result2.is_ok());
        let result2 = result2.unwrap();
        // The average of the two readings in the samples buffer
        assert_eq!(result2.used_kib, TOTAL_KIB - ((reading2 + reading1) / 2));
        assert_eq!(result2.total_kib, TOTAL_KIB);

        assert!(result3.is_ok());
        let result2 = result3.unwrap();
        // The average of the three readings in the samples buffer
        assert_eq!(result2.used_kib, TOTAL_KIB - ((reading3 + reading2 + reading1) / 3));
        assert_eq!(result2.total_kib, TOTAL_KIB);
    }

    #[test]
    fn samples_buffer_trims_to_max_size() {

        let reading1 = 22048124;
        let reading2 = 21662400;
        let reading3 = 23576212;

        let mock_memory_reader = MockProcMeminfoReader::new(vec![
            Ok(ProcMemInfoStatus::new(TOTAL_KIB, reading1)),
            Ok(ProcMemInfoStatus::new(TOTAL_KIB, reading2)),
            Ok(ProcMemInfoStatus::new(TOTAL_KIB, reading3))
        ]);

        let mut monitor = MemoryMonitor::new(mock_memory_reader, &make_config(2));

        let result1 = monitor.poll();
        let result2 = monitor.poll();
        let result3 = monitor.poll();

        assert!(result1.is_ok());
        let result1 = result1.unwrap();
        // Only one reading in the samples buffer
        assert_eq!(result1.used_kib, TOTAL_KIB - reading1);
        assert_eq!(result1.total_kib, TOTAL_KIB);

        assert!(result2.is_ok());
        let result2 = result2.unwrap();
        // The average of the two readings in the samples buffer
        assert_eq!(result2.used_kib, TOTAL_KIB - ((reading2 + reading1) / 2));
        assert_eq!(result2.total_kib, TOTAL_KIB);

        assert!(result3.is_ok());
        let result3 = result3.unwrap();
        // The first reading has been discarded here, so only reading2 and reading3 are in the samples buffer
        assert_eq!(result3.used_kib, TOTAL_KIB - ((reading2 + reading3) / 2));
        assert_eq!(result3.total_kib, TOTAL_KIB);
    }

    #[test]
    fn error_is_propagated() {
        let reader =  MockProcMeminfoReader::new(vec![Err("boom".to_string())]);
        let mut monitor = MemoryMonitor::new(reader, &make_config(1));

        let err = monitor.poll().unwrap_err();
        assert_eq!(err, "boom");
    }
}