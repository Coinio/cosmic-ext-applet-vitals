use rand::prelude::*;

pub struct MemoryMonitor;

#[derive(Default, Clone, Debug)]
pub struct MemoryInfo {
    pub used: u64,
    pub total: u64,
    pub free: u64,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&self) -> Result<MemoryInfo, &str> {

        let mut rng = thread_rng();

        let total = 32676;
        let used = rng.gen_range(0..32676);

        Ok(MemoryInfo {
            used,
            total,
            free: total - used,
        })
    }
}