use std::time::Duration;
use cosmic::iced::window;
use indexmap::IndexMap;
use crate::configuration::app_configuration::{CPU_SETTINGS_WINDOW_ID, DISK_SETTINGS_WINDOW_ID, HIDE_INDICATOR_SETTING_KEY, LABEL_COLOUR_SETTING_KEY, LABEL_TEXT_SETTING_KEY, MAX_SAMPLES_SETTING_KEY, MEMORY_SETTINGS_WINDOW_ID, NETWORK_SETTINGS_WINDOW_ID, UPDATE_INTERVAL_SETTING_KEY};
use crate::configuration::cpu::CpuConfiguration;
use crate::configuration::disk::DiskConfiguration;
use crate::configuration::memory::MemoryConfiguration;
use crate::configuration::network::NetworkConfiguration;
use crate::configuration::validation::ConfigurationValidation;
use crate::fl;

#[derive(Debug, Clone)]
pub enum SettingsFormEvent {
    StringFieldUpdated(SettingsFormEventValue),
    CheckBoxUpdated(SettingsFormEventValue),
}

#[derive(Debug, Clone)]
pub enum SettingsFormInputType {
    String,
    CheckBox,
    ColourPicker,
}

#[derive(Debug, Clone)]
pub struct SettingsFormEventValue {
    pub settings_window_id: window::Id,
    pub form_value_key: &'static str,
    pub value: String,
}

pub struct SettingsFormItem {
    pub label: String,
    pub value: String,
    pub input_type: SettingsFormInputType,
    pub validator: Option<fn(&str) -> Result<(), String>>,
}

pub struct SettingsForm {
    pub settings_window_id: window::Id,
    pub title: String,
    pub values: IndexMap<&'static str, SettingsFormItem>,
}

impl From<&CpuConfiguration> for SettingsForm {
    fn from(config: &CpuConfiguration) -> SettingsForm {
        let values = build_shared_settings(
            config.hide_indicator,
            config.update_interval,
            config.max_samples,
            config.label_colour.clone(),
            config.label_text.clone(),
        );

        SettingsForm {
            settings_window_id: CPU_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-cpu-title"),
            values,
        }
    }
}

impl From<&MemoryConfiguration> for SettingsForm {
    fn from(config: &MemoryConfiguration) -> SettingsForm {
        let values = build_shared_settings(
            config.hide_indicator,
            config.update_interval,
            config.max_samples,
            config.label_colour.clone(),
            config.label_text.clone(),
        );
        SettingsForm {
            settings_window_id: MEMORY_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-memory-title"),
            values,
        }
    }
}

impl From<&NetworkConfiguration> for SettingsForm {
    fn from(config: &NetworkConfiguration) -> SettingsForm {
        let values = build_shared_settings(
            config.hide_indicator,
            config.update_interval,
            config.max_samples,
            config.label_colour.clone(),
            config.label_text.clone(),
        );

        SettingsForm {
            settings_window_id: NETWORK_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-network-title"),
            values,
        }
    }
}

impl From<&DiskConfiguration> for SettingsForm {
    fn from(config: &DiskConfiguration) -> SettingsForm {
        let values = build_shared_settings(
            config.hide_indicator,
            config.update_interval,
            config.max_samples,
            config.label_colour.clone(),
            config.label_text.clone(),
        );
        SettingsForm {
            settings_window_id: DISK_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-disk-title"),
            values,
        }
    }
}

// This is used to build the shared settings between all the settings forms.
/// Extend the BTreeMap returned by this with any sensor-specific settings if required.
fn build_shared_settings(
    hide_indicator: bool,
    update_interval: Duration,
    max_samples: usize,
    label_colour: Option<String>,
    label_text: Option<String>,
) -> IndexMap<&'static str, SettingsFormItem> {
    IndexMap::from([
        (
            LABEL_TEXT_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-label-text"),
                value: label_text.clone().unwrap_or_default(),
                input_type: SettingsFormInputType::String,
                validator: Some(ConfigurationValidation::is_valid_label_text),
            },
        ),
        (
            LABEL_COLOUR_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-label-colour"),
                value: label_colour.clone().unwrap_or_default(),
                input_type: SettingsFormInputType::ColourPicker,
                validator: None,
            },
        ),
        (
            HIDE_INDICATOR_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-hide-indicator"),
                value: hide_indicator.to_string(),
                input_type: SettingsFormInputType::CheckBox,
                validator: Some(ConfigurationValidation::is_valid_boolean),
            },
        ),
        (
            UPDATE_INTERVAL_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-update-interval"),
                value: update_interval.as_millis().to_string(),
                input_type: SettingsFormInputType::String,
                validator: Some(ConfigurationValidation::is_valid_interval),
            },
        ),
        (
            MAX_SAMPLES_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-max-samples"),
                value: max_samples.to_string(),
                input_type: SettingsFormInputType::String,
                validator: Some(ConfigurationValidation::is_valid_max_samples),
            },
        ),
    ])
}
