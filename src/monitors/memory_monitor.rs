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
}

impl<S: SensorReader<Output = ProcMemInfoStatus>> MemoryMonitor<S> {
    pub fn new(sensor_reader: S) -> Self {
        Self {
            sensor_reader
        }
    }
    
    pub fn update(&mut self) -> Result<MemoryStats, String> {
        let meminfo_state = match self.sensor_reader.read() {
            Ok(state) => state,
            Err(err) => return Err(err),
        };

        let used = meminfo_state.total.saturating_sub(meminfo_state.available);

        Ok(MemoryStats::new(meminfo_state.total, used))
    }
}
