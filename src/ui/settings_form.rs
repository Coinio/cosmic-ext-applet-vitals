use crate::app::{AppState, Message};
use crate::configuration::app_configuration::*;
use crate::configuration::cpu::CpuConfiguration;
use crate::configuration::disk::DiskConfiguration;
use crate::configuration::memory::MemoryConfiguration;
use crate::configuration::network::NetworkConfiguration;
use crate::configuration::validation::ConfigurationValidation;
use crate::fl;
use crate::ui::app_colours::AppColours;
use cosmic::iced::{window, Background, Color, Radius};
use cosmic::iced_widget::{container, Container};
use cosmic::widget::{settings, Column};
use cosmic::{widget, Theme};
use indexmap::IndexMap;
use std::time::Duration;

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
        let mut values = build_shared_settings(
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
        let mut values = build_shared_settings(
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
        let mut values = build_shared_settings(
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
        let mut values = build_shared_settings(
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

impl SettingsForm {
    pub fn content(&self, app_state: &AppState) -> Container<'_, Message, Theme> {
        let mut column = widget::list_column().padding(2).spacing(0).divider_padding(2);

        let back_button = widget::button::custom(widget::icon::from_name("go-previous-symbolic").size(16).icon())
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
                SettingsFormInputType::ColourPicker => {
                    let palette = &app_state.core().system_theme().cosmic().palette;
                    let mut colour_picker_column: Column<Message> = widget::column().spacing(8);

                    let app_colors = AppColours::from(palette);

                    let mut row = widget::row().spacing(8);

                    for (key, colour) in app_colors.colours.into_iter() {
                        let is_selected = settings_form_item.value == key;

                        let mut button = widget::button::custom("").height(25).width(25).class(
                            cosmic::theme::style::Button::Custom {
                                active: Box::new(move |_, _| widget::button::Style {
                                    background: Some(Background::Color(colour.into())),
                                    text_color: None,
                                    border_radius: Radius::new(3.0),
                                    border_color: if is_selected { Color::WHITE } else { Color::TRANSPARENT },
                                    border_width: if is_selected { 3.0 } else { 0.0 },

                                    ..Default::default()
                                }),
                                disabled: Box::new(|_| Default::default()),
                                hovered: Box::new(move |_, _| widget::button::Style {
                                    background: Some(Background::Color(colour.into())),
                                    text_color: None,
                                    border_radius: Radius::new(3.0),
                                    border_color: if is_selected { Color::WHITE } else { Color::TRANSPARENT },
                                    border_width: if is_selected { 3.0 } else { 0.0 },

                                    ..Default::default()
                                }),
                                pressed: Box::new(|_, _| Default::default()),
                            },
                        );

                        button = button.on_press(Message::SettingsFormUpdate(SettingsFormEvent::StringFieldUpdated(
                            SettingsFormEventValue {
                                settings_window_id: self.settings_window_id,
                                form_value_key,
                                value: key.to_string(),
                            },
                        )));
                        row = row.push(button);
                    }

                    colour_picker_column = colour_picker_column.push(settings_form_item.label.as_str());
                    colour_picker_column = colour_picker_column.push(row.wrap());
                    column = column.add(colour_picker_column);
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
