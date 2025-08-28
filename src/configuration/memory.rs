use crate::configuration::app_configuration::{
    LABEL_COLOUR_SETTING_KEY, LABEL_TEXT_SETTING_KEY, MAX_SAMPLES_SETTING_KEY, MEMORY_SETTINGS_WINDOW_ID,
    UPDATE_INTERVAL_SETTING_KEY,
};
use crate::configuration::validation::ConfigurationValidation;
use crate::fl;
use crate::ui::settings_form::{SettingsForm, SettingsFormItem};
use hex_color::HexColor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

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

    pub fn to_settings_form(&self) -> SettingsForm {
        SettingsForm {
            settings_window_id: MEMORY_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-memory-title"),
            values: HashMap::from([
                (
                    LABEL_TEXT_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-label-text"),
                        value: self.label_text.clone(),
                        validator: Some(ConfigurationValidation::is_valid_label_text),
                    },
                ),
                (
                    LABEL_COLOUR_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-label-colour"),
                        value: self.label_colour.display_rgba().to_string(),
                        validator: Some(ConfigurationValidation::is_valid_colour),
                    },
                ),
                (
                    UPDATE_INTERVAL_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-update-interval"),
                        value: self.update_interval.as_millis().to_string(),
                        validator: Some(ConfigurationValidation::is_valid_interval),
                    },
                ),
                (
                    MAX_SAMPLES_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-max-samples"),
                        value: self.max_samples.to_string(),
                        validator: Some(ConfigurationValidation::is_valid_max_samples),
                    },
                ),
            ]),
        }
    }
}
