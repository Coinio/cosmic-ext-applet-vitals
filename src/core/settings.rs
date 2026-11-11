use std::time::Duration;
use cosmic::iced::window;
use indexmap::IndexMap;
use crate::configuration::app_configuration::{CPU_SETTINGS_WINDOW_ID, DISK_SETTINGS_WINDOW_ID, GENERAL_SETTINGS_WINDOW_ID, HIDE_INDICATOR_SETTING_KEY, HIDE_ICON_SETTING_KEY, ICON_COLOUR_SETTING_KEY, MAX_SAMPLES_SETTING_KEY, MEMORY_SETTINGS_WINDOW_ID, NETWORK_SETTINGS_WINDOW_ID, UPDATE_INTERVAL_SETTING_KEY, FIX_INDICATOR_SIZE_SETTING_KEY, FONT_SETTING_KEY, FONT_SIZE_SETTING_KEY};
use crate::configuration::cpu::CpuConfiguration;
use crate::configuration::disk::DiskConfiguration;
use crate::configuration::general::{
    FontSizeSlot, GeneralConfiguration, INDICATOR_FONT_SIZE_MAXIMUM, INDICATOR_FONT_SIZE_MINIMUM,
};
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
    /// The value is an encoded `IndicatorFont`.
    FontPicker,
    /// The value is an integer within the inclusive bounds.
    SpinButton { min: u16, max: u16 },
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
    pub helper_text: Option<String>,
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
            Some((config.hide_icon, config.icon_colour.clone())),
            config.update_interval,
            config.max_samples,
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
            Some((config.hide_icon, config.icon_colour.clone())),
            config.update_interval,
            config.max_samples,
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
            None,
            config.update_interval,
            config.max_samples,
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
            None,
            config.update_interval,
            config.max_samples,
        );
        SettingsForm {
            settings_window_id: DISK_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-disk-title"),
            values,
        }
    }
}

impl SettingsForm {
    /// The font size field is only shown when the applet is in a named panel/dock size, and edits that size only.
    pub fn from_general(config: &GeneralConfiguration, font_size_slot: Option<FontSizeSlot>) -> SettingsForm {
        let mut values: IndexMap<&'static str, SettingsFormItem> = IndexMap::new();
        values.insert(
            FONT_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-font"),
                value: config.font.to_form_value(),
                input_type: SettingsFormInputType::FontPicker,
                validator: None,
                helper_text: Some(fl!("settings-font-helper")),
            },
        );
        if let Some(slot) = font_size_slot {
            values.insert(
                FONT_SIZE_SETTING_KEY,
                SettingsFormItem {
                    label: fl!("settings-font-size"),
                    value: config.font_size(slot).to_string(),
                    input_type: SettingsFormInputType::SpinButton {
                        min: INDICATOR_FONT_SIZE_MINIMUM,
                        max: INDICATOR_FONT_SIZE_MAXIMUM,
                    },
                    validator: None,
                    helper_text: Some(fl!("settings-font-size-helper", size = slot.size.label())),
                },
            );
        }
        values.insert(
            FIX_INDICATOR_SIZE_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-fix-indicator-size"),
                value: config.fix_indicator_size.to_string(),
                input_type: SettingsFormInputType::CheckBox,
                validator: Some(ConfigurationValidation::is_valid_boolean),
                helper_text: Some(fl!("settings-fix-indicator-size-helper")),
            },
        );
        values.insert(
            crate::configuration::app_configuration::USE_IEC_UNITS_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-use-iec-units"),
                value: config.use_iec_units.to_string(),
                input_type: SettingsFormInputType::CheckBox,
                validator: Some(ConfigurationValidation::is_valid_boolean),
                helper_text: Some(fl!("settings-use-iec-units-helper")),
            },
        );

        SettingsForm {
            settings_window_id: GENERAL_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-general-title"),
            values,
        }
    }
}

// This is used to build the shared settings between all the settings forms.
/// Extend the map returned by this with any sensor-specific settings if required.
/// `icon` is the indicator's (hide icon, icon colour) settings, for indicators that have an icon.
fn build_shared_settings(
    hide_indicator: bool,
    icon: Option<(bool, Option<String>)>,
    update_interval: Duration,
    max_samples: usize,
) -> IndexMap<&'static str, SettingsFormItem> {
    let mut settings = IndexMap::from([(
        HIDE_INDICATOR_SETTING_KEY,
        SettingsFormItem {
            label: fl!("settings-hide-indicator"),
            value: hide_indicator.to_string(),
            input_type: SettingsFormInputType::CheckBox,
            validator: Some(ConfigurationValidation::is_valid_boolean),
            helper_text: None,
        },
    )]);

    if let Some((hide_icon, icon_colour)) = icon {
        settings.insert(
            HIDE_ICON_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-hide-icon"),
                value: hide_icon.to_string(),
                input_type: SettingsFormInputType::CheckBox,
                validator: Some(ConfigurationValidation::is_valid_boolean),
                helper_text: None,
            },
        );
        settings.insert(
            ICON_COLOUR_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-icon-colour"),
                value: icon_colour.unwrap_or_default(),
                input_type: SettingsFormInputType::ColourPicker,
                validator: None,
                helper_text: None,
            },
        );
    }

    settings.extend([
        (
            UPDATE_INTERVAL_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-update-interval"),
                value: update_interval.as_millis().to_string(),
                input_type: SettingsFormInputType::String,
                validator: Some(ConfigurationValidation::is_valid_interval),
                helper_text: None,
            },
        ),
        (
            MAX_SAMPLES_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-max-samples"),
                value: max_samples.to_string(),
                input_type: SettingsFormInputType::String,
                validator: Some(ConfigurationValidation::is_valid_max_samples),
                helper_text: None,
            },
        ),
    ]);

    settings
}
