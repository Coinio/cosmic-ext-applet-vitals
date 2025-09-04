use crate::configuration::app_configuration::{
    CPU_SETTINGS_WINDOW_ID, HIDE_INDICATOR_SETTING_KEY, MAX_SAMPLES_SETTING_KEY, UPDATE_INTERVAL_SETTING_KEY,
};
use crate::configuration::validation::ConfigurationValidation;
use crate::fl;
use crate::ui::settings_form::{SettingsForm, SettingsFormInputType, SettingsFormItem};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

/// The configuration for the CPU monitor
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CpuConfiguration {
    /// Whether to hide the CPU indicator from the panel
    pub hide_indicator: bool,
    /// The duration between each update interval, i.e. 5 seconds
    pub update_interval: Duration,
    /// The number of samples to keep and average for the final result
    pub max_samples: usize,
}

impl Default for CpuConfiguration {
    fn default() -> Self {
        CpuConfiguration {
            update_interval: Duration::from_secs(1),
            max_samples: 4,
            hide_indicator: false,
        }
    }
}

impl CpuConfiguration {
    pub fn from(&self, settings_form: &SettingsForm) -> Self {
        if settings_form.settings_window_id != CPU_SETTINGS_WINDOW_ID.clone() {
            panic!("Attempted to update CPU settings from a non-cpu settings window.")
        }

        CpuConfiguration {
            hide_indicator: ConfigurationValidation::sanitise_boolean_input(
                settings_form
                    .values
                    .get(HIDE_INDICATOR_SETTING_KEY)
                    .expect("Hide indicator missing from settings form options")
                    .value
                    .clone(),
                self.hide_indicator,
            ),
            update_interval: ConfigurationValidation::sanitise_interval_input(
                settings_form
                    .values
                    .get(UPDATE_INTERVAL_SETTING_KEY)
                    .expect("Update interval missing from settings form options")
                    .value
                    .clone(),
                self.update_interval,
            ),
            max_samples: ConfigurationValidation::sanitise_max_samples(
                settings_form
                    .values
                    .get(MAX_SAMPLES_SETTING_KEY)
                    .expect("Max samples missing from settings form options")
                    .value
                    .clone(),
                self.max_samples,
            ),
        }
    }

    pub fn to_settings_form(&self) -> SettingsForm {
        SettingsForm {
            settings_window_id: CPU_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-cpu-title"),
            values: BTreeMap::from([
                (
                    HIDE_INDICATOR_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-hide-indicator"),
                        value: self.hide_indicator.to_string(),
                        input_type: SettingsFormInputType::CheckBox,
                        validator: Some(ConfigurationValidation::is_valid_boolean),
                    },
                ),
                (
                    UPDATE_INTERVAL_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-update-interval"),
                        value: self.update_interval.as_millis().to_string(),
                        input_type: SettingsFormInputType::String,
                        validator: Some(ConfigurationValidation::is_valid_interval),
                    },
                ),
                (
                    MAX_SAMPLES_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-max-samples"),
                        value: self.max_samples.to_string(),
                        input_type: SettingsFormInputType::String,
                        validator: Some(ConfigurationValidation::is_valid_max_samples),
                    },
                ),
            ]),
        }
    }
}
