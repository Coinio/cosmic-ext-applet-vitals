use crate::configuration::cpu::CpuConfiguration;
use crate::configuration::disk::DiskConfiguration;
use crate::configuration::general::{FontSizeSlot, GeneralConfiguration};
use crate::configuration::memory::MemoryConfiguration;
use crate::configuration::network::NetworkConfiguration;
use crate::configuration::storage::StorageConfiguration;
use crate::core::settings::SettingsForm;
use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use cosmic::iced::window;
use once_cell::sync::Lazy;
use std::collections::BTreeMap;

pub static GENERAL_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> =
    Lazy::new(|| cosmic::iced::window::Id::unique());
pub static CPU_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> = Lazy::new(|| cosmic::iced::window::Id::unique());
pub static MEMORY_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> = Lazy::new(|| cosmic::iced::window::Id::unique());
pub static NETWORK_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> =
    Lazy::new(|| cosmic::iced::window::Id::unique());
pub static DISK_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> = Lazy::new(|| cosmic::iced::window::Id::unique());
pub static STORAGE_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> =
    Lazy::new(|| cosmic::iced::window::Id::unique());

pub const SENSOR_INTERVAL_MINIMUM_IN_MS: u64 = 250;
pub const SENSOR_MAX_SAMPLES_MINIMUM: usize = 1;
pub const ICON_COLOUR_SETTING_KEY: &'static str = "settings-icon-colour";
pub const DOWNLOAD_COLOUR_SETTING_KEY: &'static str = "settings-download-colour";
pub const UPLOAD_COLOUR_SETTING_KEY: &'static str = "settings-upload-colour";
pub const READ_COLOUR_SETTING_KEY: &'static str = "settings-read-colour";
pub const WRITE_COLOUR_SETTING_KEY: &'static str = "settings-write-colour";
pub const HIDE_INDICATOR_SETTING_KEY: &'static str = "settings-hide-indicator";
pub const HIDE_ICON_SETTING_KEY: &'static str = "settings-hide-icon";
pub const UPDATE_INTERVAL_SETTING_KEY: &'static str = "settings-update-interval";
pub const MAX_SAMPLES_SETTING_KEY: &'static str = "settings-max-samples";
pub const FIX_INDICATOR_SIZE_SETTING_KEY: &'static str = "settings-fix-indicator-size";
pub const USE_IEC_UNITS_SETTING_KEY: &'static str = "settings-use-iec-units";
pub const SHOW_INDICATOR_SEPARATORS_SETTING_KEY: &'static str = "settings-show-indicator-separators";
pub const FONT_SETTING_KEY: &'static str = "settings-font";
pub const FONT_SIZE_SETTING_KEY: &'static str = "settings-font-size";
pub const STORAGE_MOUNTS_SETTING_KEY: &'static str = "settings-storage-mounts";
pub const CPU_COMBINE_CORES_SETTING_KEY: &'static str = "settings-cpu-combine-cores";
pub const CPU_TEMPERATURE_SENSOR_SETTING_KEY: &'static str = "settings-cpu-temperature-sensor";
pub const CPU_FAN_SENSOR_SETTING_KEY: &'static str = "settings-cpu-fan-sensor";
pub const CPU_USE_FAHRENHEIT_SETTING_KEY: &'static str = "settings-cpu-use-fahrenheit";

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct AppConfiguration {
    pub general: GeneralConfiguration,
    pub cpu: CpuConfiguration,
    pub memory: MemoryConfiguration,
    pub network: NetworkConfiguration,
    pub disk: DiskConfiguration,
    pub storage: StorageConfiguration,
}

impl AppConfiguration {
    /// `font_size_slot` is the panel/dock size the applet is running in, whose font size the general form edits.
    pub fn settings_form_options(&self, font_size_slot: Option<FontSizeSlot>) -> BTreeMap<window::Id, SettingsForm> {
        BTreeMap::from([
            (
                GENERAL_SETTINGS_WINDOW_ID.clone(),
                SettingsForm::from_general(&self.general, font_size_slot),
            ),
            (CPU_SETTINGS_WINDOW_ID.clone(), SettingsForm::from(&self.cpu)),
            (MEMORY_SETTINGS_WINDOW_ID.clone(), SettingsForm::from(&self.memory)),
            (NETWORK_SETTINGS_WINDOW_ID.clone(), SettingsForm::from(&self.network)),
            (DISK_SETTINGS_WINDOW_ID.clone(), SettingsForm::from(&self.disk)),
            (STORAGE_SETTINGS_WINDOW_ID.clone(), SettingsForm::from(&self.storage)),
        ])
    }
}
