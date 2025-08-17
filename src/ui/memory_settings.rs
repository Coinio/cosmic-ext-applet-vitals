use crate::core::app_configuration::{MemoryConfiguration};
use crate::fl;
use crate::ui::settings::{
    SettingsFormItem, LABEL_COLOUR_SETTING_KEY, LABEL_TEXT_SETTING_KEY,
    MAX_SAMPLES_SETTING_KEY, UPDATE_INTERVAL_SETTING_KEY,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct MemorySettings;

impl MemorySettings {
    pub fn from(configuration: &MemoryConfiguration) -> HashMap<&'static str, SettingsFormItem> {
        HashMap::from([
            (
                LABEL_TEXT_SETTING_KEY,
                SettingsFormItem {
                    label: fl!("settings-label-text"),
                    value: configuration.label_text.clone(),
                },
            ),
            (
                LABEL_COLOUR_SETTING_KEY,
                SettingsFormItem {
                    label: fl!("settings-label-colour"),
                    value: configuration.label_colour.display_rgba().to_string(),
                },
            ),
            (
                UPDATE_INTERVAL_SETTING_KEY,
                SettingsFormItem {
                    label: fl!("settings-update-interval"),
                    value: configuration.update_interval.as_millis().to_string(),
                },
            ),
            (
                MAX_SAMPLES_SETTING_KEY,
                SettingsFormItem {
                    label: fl!("settings-max-samples"),
                    value: configuration.max_samples.to_string(),
                },
            ),
        ])
    }

}
