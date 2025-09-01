use crate::configuration::app_configuration::{CPU_SETTINGS_WINDOW_ID, LABEL_COLOUR_SETTING_KEY, LABEL_TEXT_SETTING_KEY, MAX_SAMPLES_SETTING_KEY, UPDATE_INTERVAL_SETTING_KEY};
use crate::configuration::validation::ConfigurationValidation;
use crate::fl;
use crate::ui::settings_form::{
    SettingsForm, SettingsFormItem
};
use hex_color::HexColor;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

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

    pub fn to_settings_form(&self) -> SettingsForm {
        SettingsForm {
            settings_window_id: CPU_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-cpu-title"),
            values: BTreeMap::from([
                (
                    LABEL_TEXT_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-label-text"),
                        value: self.label_text.clone(),
                        validator: Some(ConfigurationValidation::is_valid_label_text)
                    },
                ),
                (
                    LABEL_COLOUR_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-label-colour"),
                        value: self.label_colour.display_rgba().to_string(),
                        validator: Some(ConfigurationValidation::is_valid_colour)
                    },
                ),
                (
                    UPDATE_INTERVAL_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-update-interval"),
                        value: self.update_interval.as_millis().to_string(),
                        validator: Some(ConfigurationValidation::is_valid_interval)
                    },
                ),
                (
                    MAX_SAMPLES_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-max-samples"),
                        value: self.max_samples.to_string(),
                        validator: Some(ConfigurationValidation::is_valid_max_samples)
                    },
                ),
            ]),
        }
    }
}
