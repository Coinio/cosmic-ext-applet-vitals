use crate::configuration::app_configuration::{
    DISK_SETTINGS_WINDOW_ID, MAX_SAMPLES_SETTING_KEY, UPDATE_INTERVAL_SETTING_KEY,
};
use crate::configuration::validation::ConfigurationValidation;
use crate::fl;
use crate::ui::settings_form::{SettingsForm, SettingsFormItem};
use hex_color::HexColor;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

/// The configuration for the memory monitor
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiskConfiguration {
    /// The duration between each update interval, i.e. 5 seconds
    pub update_interval: Duration,
    /// The number of samples to keep and average for the final result
    pub max_samples: usize,
}

impl Default for DiskConfiguration {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_secs(1),
            max_samples: 3
        }
    }
}

impl DiskConfiguration {
    pub fn from(&self, settings_form: &SettingsForm) -> Self {
        if settings_form.settings_window_id != DISK_SETTINGS_WINDOW_ID.clone() {
            panic!("Attempted to update disk settings from a non-disk settings window.")
        }

        DiskConfiguration {
            update_interval: ConfigurationValidation::sanitise_interval_input(
                settings_form
                    .values
                    .get(crate::configuration::app_configuration::UPDATE_INTERVAL_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
                self.update_interval,
            ),
            max_samples: ConfigurationValidation::sanitise_max_samples(
                settings_form.values.get(MAX_SAMPLES_SETTING_KEY).unwrap().value.clone(),
                self.max_samples,
            ),
        }
    }

    pub fn to_settings_form(&self) -> SettingsForm {
        SettingsForm {
            settings_window_id: DISK_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-disk-title"),
            values: BTreeMap::from([
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
