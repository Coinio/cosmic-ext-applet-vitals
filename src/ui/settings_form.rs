use crate::app::Message;
use crate::configuration::app_configuration::*;
use crate::configuration::cpu::CpuConfiguration;
use crate::configuration::validation::ConfigurationValidation;
use crate::fl;
use cosmic::iced::window;
use cosmic::iced_widget::{container, Container};
use cosmic::widget::{settings};
use cosmic::{widget, Theme};
use std::collections::BTreeMap;
use std::time::Duration;
use crate::configuration::disk::DiskConfiguration;
use crate::configuration::memory::MemoryConfiguration;
use crate::configuration::network::NetworkConfiguration;

#[derive(Debug, Clone)]
pub enum SettingsFormEvent {
    StringFieldUpdated(SettingsFormEventValue),
    CheckBoxUpdated(SettingsFormEventValue),
}

#[derive(Debug, Clone)]
pub enum SettingsFormInputType {
    String,
    CheckBox,
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
    pub values: BTreeMap<&'static str, SettingsFormItem>,
}

impl SettingsForm {
    pub fn from_cpu_config(config: &CpuConfiguration) -> SettingsForm {
        SettingsForm {
            settings_window_id: CPU_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-cpu-title"),
            values: build_shared_settings(
                config.hide_indicator,
                config.update_interval,
                config.max_samples,
            ),
        }
    }

    pub fn from_memory_config(config: &MemoryConfiguration) -> SettingsForm {
        SettingsForm {
            settings_window_id: MEMORY_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-memory-title"),
            values: build_shared_settings(
                config.hide_indicator,
                config.update_interval,
                config.max_samples,
            ),
        }
    }

    pub fn from_network_config(config: &NetworkConfiguration) -> SettingsForm {
        SettingsForm {
            settings_window_id: NETWORK_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-network-title"),
            values: build_shared_settings(
                config.hide_indicator,
                config.update_interval,
                config.max_samples,
            ),
        }
    }

    pub fn from_disk_config(config: &DiskConfiguration) -> SettingsForm {
        SettingsForm {
            settings_window_id: DISK_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-disk-title"),
            values: build_shared_settings(
                config.hide_indicator,
                config.update_interval,
                config.max_samples,
            ),
        }
    }

    pub fn content(&self) -> Container<'_, Message, Theme> {
        let mut column =
            widget::list_column()
                .padding(2)
                .spacing(0)
                .divider_padding(2);

        let back_button = widget::button::custom(widget::icon::from_name("go-previous-symbolic")
            .size(16).icon())
            .on_press(Message::SettingsPopupOpened(MAIN_SETTINGS_WINDOW_ID.clone()));

        column = column.add(settings::item(self.title.clone(), back_button));

        for (form_value_key, settings_form_item) in self.values.iter() {
            match settings_form_item.input_type {
                SettingsFormInputType::String => {
                    let text_input =
                        widget::text_input(fl!("settings-empty"), &settings_form_item.value).on_input(|new_value| {
                            Message::SettingsFormUpdate(SettingsFormEvent::StringFieldUpdated(SettingsFormEventValue {
                                settings_window_id: self.settings_window_id,
                                form_value_key,
                                value: new_value,
                            }))
                        });

                    let validator = settings_form_item.validator.unwrap_or(|_| Ok(()));

                    let field = match validator(&settings_form_item.value) {
                        Ok(_) => text_input.width(150),
                        Err(error_text) => text_input
                            .width(150)
                            .error(error_text.clone())
                            .helper_text(error_text.clone()),
                    };

                    column = column.add(settings::item(settings_form_item.label.clone(), field));
                }
                SettingsFormInputType::CheckBox => {
                    let converted_value = settings_form_item.value.parse::<bool>().unwrap_or(false);

                    let checkbox_input = widget::checkbox("", converted_value).on_toggle(|new_value| {
                        Message::SettingsFormUpdate(SettingsFormEvent::CheckBoxUpdated(SettingsFormEventValue {
                            settings_window_id: self.settings_window_id,
                            form_value_key,
                            value: new_value.to_string(),
                        }))
                    });

                    column = column.add(settings::item(settings_form_item.label.clone(), checkbox_input));
                }
            };
        }

        container(column)
    }
}

/// This is used to build the shared settings between all the settings forms.
/// Extend the BTreeMap returned by this with any sensor-specific settings if required.
fn build_shared_settings(
    hide_indicator: bool,
    update_interval: Duration,
    max_samples: usize,
) -> BTreeMap<&'static str, SettingsFormItem> {
    BTreeMap::from([
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
