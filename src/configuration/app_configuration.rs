use crate::fl;
use crate::ui::settings::{
    SettingsForm, SettingsFormItem, LABEL_COLOUR_SETTING_KEY, LABEL_TEXT_SETTING_KEY, MAX_SAMPLES_SETTING_KEY,
    UPDATE_INTERVAL_SETTING_KEY,
};
use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use hex_color::HexColor;
use log::error;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use crate::configuration::validation::ConfigurationValidation;

pub static CPU_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> = Lazy::new(|| cosmic::iced::window::Id::unique());
pub static MEMORY_SETTINGS_WINDOW_ID: Lazy<cosmic::iced::window::Id> = Lazy::new(|| cosmic::iced::window::Id::unique());

pub static SENSOR_INTERVAL_MINIMUM_IN_MS: u64 = 250;
pub static SENSOR_MAX_SAMPLES_MINIMUM: usize = 1;
pub static SENSOR_MAX_LABEL_LENGTH: usize = 16;

/// The configuration for the CPU monitor
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CpuConfiguration {
    /// The duration between each update interval, i.e. 5 seconds
    pub update_interval: Duration,
    /// The number of samples to keep and average for the final result
    pub max_samples: usize,
    /// The label text
    pub label_text: String,
    /// The label colour in hex format
    pub label_colour: HexColor,
}

impl Default for CpuConfiguration {
    fn default() -> Self {
        CpuConfiguration {
            update_interval: Duration::from_secs(1),
            max_samples: 4,
            label_text: "CPU".to_string(),
            label_colour: "#029BAC".parse().unwrap(),
        }
    }
}

impl CpuConfiguration {
    pub fn from(&self, settings_form: &SettingsForm) -> Self {
        if settings_form.settings_window_id != CPU_SETTINGS_WINDOW_ID.clone() {
            panic!("Attempted to update CPU settings from a non-cpu settings window.")
        }

        CpuConfiguration {
            update_interval: ConfigurationValidation::sanitise_interval_input(
                settings_form
                    .values
                    .get(UPDATE_INTERVAL_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
                self.update_interval,
            ),
            max_samples: ConfigurationValidation::sanitise_max_samples(
                settings_form.values.get(MAX_SAMPLES_SETTING_KEY).unwrap().value.clone(),
                self.max_samples,
            ),
            label_text: ConfigurationValidation::sanitise_label_text(
                settings_form.values.get(LABEL_TEXT_SETTING_KEY).unwrap().value.clone(),
            ),
            label_colour: ConfigurationValidation::sanitise_label_colour(
                settings_form
                    .values
                    .get(LABEL_COLOUR_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
                self.label_colour,
            ),
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
    /// The label color in hex format
    pub label_colour: HexColor,
}

impl Default for MemoryConfiguration {
    fn default() -> Self {
        MemoryConfiguration {
            update_interval: Duration::from_secs(1),
            max_samples: 2,
            label_text: "RAM".to_string(),
            label_colour: "#029BAC".parse().unwrap(),
        }
    }
}

impl MemoryConfiguration {
    pub fn from(&self, settings_form: &SettingsForm) -> Self {
        if settings_form.settings_window_id != MEMORY_SETTINGS_WINDOW_ID.clone() {
            panic!("Attempted to update memory settings from a non-memory settings window.")
        }

        MemoryConfiguration {
            update_interval: ConfigurationValidation::sanitise_interval_input(
                settings_form
                    .values
                    .get(UPDATE_INTERVAL_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
                self.update_interval,
            ),
            max_samples: ConfigurationValidation::sanitise_max_samples(
                settings_form.values.get(MAX_SAMPLES_SETTING_KEY).unwrap().value.clone(),
                self.max_samples,
            ),
            label_text: ConfigurationValidation::sanitise_label_text(
                settings_form.values.get(LABEL_TEXT_SETTING_KEY).unwrap().value.clone(),
            ),
            label_colour: ConfigurationValidation::sanitise_label_colour(
                settings_form
                    .values
                    .get(LABEL_COLOUR_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
                self.label_colour,
            ),
        }
    }
}

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct AppConfiguration {
    pub cpu: CpuConfiguration,
    pub memory: MemoryConfiguration,
}
