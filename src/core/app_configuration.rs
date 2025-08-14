use std::time::Duration;
use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub static CPU_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> = Lazy::new(|| cosmic::iced::window::Id::unique());
pub static MEMORY_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> = Lazy::new(|| cosmic::iced::window::Id::unique());

pub static SENSOR_INTERVAL_MINIMUM_IN_MS: u64 = 250;
pub static SENSOR_MAX_SAMPLES_MINIMUM: usize = 1;
pub static SENSOR_MAX_LABEL_LENGTH: usize = 16;

#[derive(Debug, Clone)]
pub enum ConfigurationValue {
    MemoryLabelText(String),
    MemoryUpdateInterval(Duration),
    MemoryMaxSamples(usize),
    CpuLabelText(String),
    CpuUpdateInterval(Duration),
    CpuMaxSamples(usize),
}

/// The configuration for the CPU monitor
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CpuConfiguration {
    /// The duration between each update interval, i.e. 5 seconds
    pub update_interval: Duration,
    /// The number of samples to keep and average for the final result
    pub max_samples: usize,
    /// The label text
    pub label_text: String,
}

impl Default for CpuConfiguration {
    fn default() -> Self {
        CpuConfiguration {
            update_interval: Duration::from_secs(1),
            max_samples: 4,
            label_text: "CPU".to_string(),
        }
    }
}

/// The configuration for the memory monitor
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryConfiguration {
    /// The duration between each update interval, i.e. 5 seconds
    pub update_interval: Duration,
    /// The number of samples to keep and average for the final result
    pub max_samples: usize,
    /// The label text
    pub label_text: String,
}

impl Default for MemoryConfiguration {
    fn default() -> Self {
        MemoryConfiguration {
            update_interval: Duration::from_secs(1),
            max_samples: 2,
            label_text: "RAM".to_string(),
        }
    }
}

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct AppConfiguration {
    pub cpu: CpuConfiguration,
    pub memory: MemoryConfiguration
}
