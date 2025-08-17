use std::collections::HashMap;
use crate::app::{AppState, Message};
use crate::fl;
use cosmic::iced_widget::{container, Container};
use cosmic::{widget, Theme};
use cosmic::widget::settings;
use crate::core::app_configuration::{CpuConfiguration};
use crate::ui::settings::{SettingsFormEvent, SettingsFormEventValue, SettingsFormItem, LABEL_COLOUR_SETTING_KEY, LABEL_TEXT_SETTING_KEY, MAX_SAMPLES_SETTING_KEY, UPDATE_INTERVAL_SETTING_KEY};

pub const CPU_SETTINGS_FORM_KEY: &str = "cpu_settings_form";

#[derive(Debug, Clone, Default)]
pub struct CpuSettingsForm {
    pub label_text: String,
    pub label_colour: String,
    pub update_interval: String,
    pub max_samples: String,
}

impl CpuSettingsForm {

    pub fn from(configuration: &CpuConfiguration) -> HashMap<&'static str, SettingsFormItem> {
        HashMap::from([
            (
                LABEL_TEXT_SETTING_KEY,
                SettingsFormItem {
                    form_value_key: LABEL_TEXT_SETTING_KEY,
                    label: fl!("settings-label-text"),
                    value: configuration.label_text.clone(),
                },
            ),
            (
                LABEL_COLOUR_SETTING_KEY,
                SettingsFormItem {
                    form_value_key: LABEL_COLOUR_SETTING_KEY,
                    label: fl!("settings-label-colour"),
                    value: configuration.label_colour.display_rgba().to_string(),
                },
            ),
            (
                UPDATE_INTERVAL_SETTING_KEY,
                SettingsFormItem {
                    form_value_key: UPDATE_INTERVAL_SETTING_KEY,
                    label: fl!("settings-update-interval"),
                    value: configuration.update_interval.as_millis().to_string(),
                },
            ),
            (
                MAX_SAMPLES_SETTING_KEY,
                SettingsFormItem {
                    form_value_key: MAX_SAMPLES_SETTING_KEY,
                    label: fl!("settings-max-samples"),
                    value: configuration.max_samples.to_string(),
                },
            ),
        ])
    }
}
