use rand::prelude::*;

pub struct MemoryMonitor;

#[derive(Default, Clone, Debug)]
pub struct MemoryInfo {
    pub total_kilobytes: u64,
    pub free_kilobytes: u64,
    pub available_kilobytes: u64,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&self) -> Result<MemoryInfo, &str> {

        let mut rng = thread_rng();

        let total = 32676;        
        let available = rng.gen_range(0..32676);

        Ok(MemoryInfo {
            total_kilobytes: total,
            free_kilobytes: total - available,
            available_kilobytes: available,
        })
    }
}