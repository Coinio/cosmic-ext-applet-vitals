use std::collections::HashMap;
use crate::configuration::app_configuration::{CpuConfiguration, CPU_SETTINGS_WINDOW_ID};
use crate::configuration::validation::ConfigurationValidation;
use crate::fl;
use crate::ui::settings::{SettingsForm, SettingsFormItem, LABEL_COLOUR_SETTING_KEY, LABEL_TEXT_SETTING_KEY, MAX_SAMPLES_SETTING_KEY, UPDATE_INTERVAL_SETTING_KEY};

#[derive(Debug, Clone, Default)]
pub struct CpuSettings;

impl CpuSettings {

    pub fn from(configuration: &CpuConfiguration) -> SettingsForm {
        SettingsForm {
            settings_window_id: CPU_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-cpu-title"),
            values: HashMap::from([
                (
                    LABEL_TEXT_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-label-text"),
                        value: configuration.label_text.clone(),
                        validator: Some(ConfigurationValidation::is_valid_label_text)
                    },
                ),
                (
                    LABEL_COLOUR_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-label-colour"),
                        value: configuration.label_colour.display_rgba().to_string(),
                        validator: Some(ConfigurationValidation::is_valid_colour)
                    },
                ),
                (
                    UPDATE_INTERVAL_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-update-interval"),
                        value: configuration.update_interval.as_millis().to_string(),
                        validator: Some(ConfigurationValidation::is_valid_interval)
                    },
                ),
                (
                    MAX_SAMPLES_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-max-samples"),
                        value: configuration.max_samples.to_string(),
                        validator: Some(ConfigurationValidation::is_valid_max_samples)
                    },
                ),
            ])
        }
    }
}
