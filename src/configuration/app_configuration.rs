use crate::configuration::cpu::CpuConfiguration;
use crate::configuration::memory::MemoryConfiguration;
use crate::configuration::network::NetworkConfiguration;
use crate::ui::settings_form::SettingsForm;
use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use cosmic::iced::window;
use once_cell::sync::Lazy;
use std::collections::BTreeMap;
use crate::configuration::disk::DiskConfiguration;

pub static MAIN_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> = Lazy::new(|| cosmic::iced::window::Id::unique());
pub static CPU_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> = Lazy::new(|| cosmic::iced::window::Id::unique());
pub static MEMORY_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> = Lazy::new(|| cosmic::iced::window::Id::unique());
pub static NETWORK_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> =
    Lazy::new(|| cosmic::iced::window::Id::unique());
pub static DISK_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> = Lazy::new(|| cosmic::iced::window::Id::unique());

pub const SENSOR_INTERVAL_MINIMUM_IN_MS: u64 = 250;
pub const SENSOR_MAX_SAMPLES_MINIMUM: usize = 1;
pub const SENSOR_MAX_LABEL_LENGTH: usize = 16;
pub const LABEL_TEXT_SETTING_KEY: &'static str = "settings-label-text";
pub const LABEL_COLOUR_SETTING_KEY: &'static str = "settings-label-colour";
pub const NETWORK_RX_COLOUR_SETTING_KEY: &'static str = "settings-network-rx-colour";
pub const NETWORK_TX_COLOUR_SETTING_KEY: &'static str = "settings-network-tx-colour";
pub const DISK_READ_COLOUR_SETTING_KEY: &'static str = "settings-disk-read-colour";
pub const DISK_WRITE_COLOUR_SETTING_KEY: &'static str = "settings-disk-write-colour";

pub const HIDE_INDICATOR_SETTING_KEY: &'static str = "settings-hide-indicator";
pub const UPDATE_INTERVAL_SETTING_KEY: &'static str = "settings-update-interval";
pub const MAX_SAMPLES_SETTING_KEY: &'static str = "settings-max-samples";


#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct AppConfiguration {
    pub cpu: CpuConfiguration,
    pub memory: MemoryConfiguration,
    pub network: NetworkConfiguration,
    pub disk: DiskConfiguration
}

impl AppConfiguration {
    pub fn settings_form_options(&self) -> BTreeMap<window::Id, SettingsForm> {
        BTreeMap::from([
            (CPU_SETTINGS_WINDOW_ID.clone(), SettingsForm::from_cpu_config(&self.cpu)),
            (MEMORY_SETTINGS_WINDOW_ID.clone(), SettingsForm::from_memory_config(&self.memory)),
            (NETWORK_SETTINGS_WINDOW_ID.clone(), SettingsForm::from_network_config(&self.network)),        
            (DISK_SETTINGS_WINDOW_ID.clone(), SettingsForm::from_disk_config(&self.disk))
        ])
    }
}
