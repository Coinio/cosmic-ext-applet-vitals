use std::collections::VecDeque;
use crate::sensors::proc_meminfo_reader::ProcMemInfoStatus;
use crate::sensors::sensor_traits::SensorReader;

#[derive(Default, Clone, Debug)]
pub struct MemoryStats {
    pub total_kibibytes: u64,
    pub used_kibibytes: u64,
}

impl MemoryStats {
    pub fn new(total: u64, used: u64) -> Self {
        Self {
            total_kibibytes: total,
            used_kibibytes: used,
        }
    }
}

pub struct MemoryMonitor<S: SensorReader<Output = ProcMemInfoStatus>> {
    sensor_reader: S,
    sample_buffer: VecDeque<u64>
}

impl<S: SensorReader<Output = ProcMemInfoStatus>> MemoryMonitor<S> {
    pub fn new(sensor_reader: S, window_size: usize) -> Self {
        Self {
            sensor_reader,
            sample_buffer: VecDeque::with_capacity(window_size)
        }
    }
    
    pub fn poll(&mut self) -> Result<MemoryStats, String> {
        let meminfo_state = match self.sensor_reader.read() {
            Ok(state) => state,
            Err(err) => return Err(err),
        };

        let current_used = meminfo_state.total.saturating_sub(meminfo_state.available);
        
        self.sample_buffer.push_back(current_used);
        
        if self.sample_buffer.len() > self.sample_buffer.capacity() {
            self.sample_buffer.pop_front();       
        }
        
        let average_used = self.sample_buffer.iter().sum::<u64>() / self.sample_buffer.len() as u64;

        Ok(MemoryStats::new(meminfo_state.total, average_used))
    }
}
