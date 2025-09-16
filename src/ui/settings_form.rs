use crate::app::{AppState, Message};
use crate::configuration::app_configuration::*;
use crate::configuration::cpu::CpuConfiguration;
use crate::configuration::disk::DiskConfiguration;
use crate::configuration::memory::MemoryConfiguration;
use crate::configuration::network::NetworkConfiguration;
use crate::configuration::validation::ConfigurationValidation;
use crate::fl;
use cosmic::cosmic_theme::palette::Srgba;
use cosmic::iced::{window, Background, Radius};
use cosmic::iced_widget::{container, Container};
use cosmic::widget::{settings};
use cosmic::{widget, Theme};
use std::collections::BTreeMap;
use std::time::Duration;
use crate::ui::color_util::ColorUtil;

#[derive(Debug, Clone)]
pub enum SettingsFormEvent {
    StringFieldUpdated(SettingsFormEventValue),
    CheckBoxUpdated(SettingsFormEventValue),
}

#[derive(Debug, Clone)]
pub enum SettingsFormInputType {
    String,
    CheckBox,
    ColorPicker,
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
        let mut values = build_shared_settings(config.hide_indicator, config.update_interval, config.max_samples);
        values.insert(
            LABEL_COLOUR_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-label-colour"),
                value: config.label_colour.clone().unwrap_or_default(),
                input_type: SettingsFormInputType::ColorPicker,
                validator: None,
            },
        );
        SettingsForm {
            settings_window_id: CPU_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-cpu-title"),
            values,
        }
    }

    pub fn from_memory_config(config: &MemoryConfiguration) -> SettingsForm {
        let mut values = build_shared_settings(config.hide_indicator, config.update_interval, config.max_samples);
        values.insert(
            LABEL_COLOUR_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-label-colour"),
                value: config.label_colour.clone().unwrap_or_default(),
                input_type: SettingsFormInputType::ColorPicker,
                validator: None,
            },
        );
        SettingsForm {
            settings_window_id: MEMORY_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-memory-title"),
            values,
        }
    }

    pub fn from_network_config(config: &NetworkConfiguration) -> SettingsForm {
        let mut values = build_shared_settings(config.hide_indicator, config.update_interval, config.max_samples);
        values.insert(
            NETWORK_RX_COLOUR_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-network-rx-colour"),
                value: config.label_colour_rx.clone().unwrap_or_default(),
                input_type: SettingsFormInputType::ColorPicker,
                validator: None,
            },
        );
        values.insert(
            NETWORK_TX_COLOUR_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-network-tx-colour"),
                value: config.label_colour_tx.clone().unwrap_or_default(),
                input_type: SettingsFormInputType::ColorPicker,
                validator: None,
            },
        );
        SettingsForm {
            settings_window_id: NETWORK_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-network-title"),
            values,
        }
    }

    pub fn from_disk_config(config: &DiskConfiguration) -> SettingsForm {
        let mut values = build_shared_settings(config.hide_indicator, config.update_interval, config.max_samples);
        values.insert(
            DISK_READ_COLOUR_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-disk-read-colour"),
                value: config.label_colour_read.clone().unwrap_or_default(),
                input_type: SettingsFormInputType::ColorPicker,
                validator: None,
            },
        );
        values.insert(
            DISK_WRITE_COLOUR_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-disk-write-colour"),
                value: config.label_colour_write.clone().unwrap_or_default(),
                input_type: SettingsFormInputType::ColorPicker,
                validator: None,
            },
        );
        SettingsForm {
            settings_window_id: DISK_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-disk-title"),
            values,
        }
    }

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
                SettingsFormInputType::ColorPicker => {
                    let palette = &app_state.core().system_theme().cosmic().palette;
                    // Available palette keys and display names
                    let options: [(&str, Srgba); 7] = [
                        ("Default", palette.accent_green),
                        ("indigo", palette.accent_indigo),
                        ("orange", palette.accent_orange),
                        ("green", palette.accent_green),
                        ("red", palette.accent_red),
                        ("purple", palette.accent_purple),
                        ("warm_grey", palette.accent_warm_grey),
                    ];

                    let mut row = widget::row();
                    row = row.spacing(8);

                    for (key, colour) in options {
                        let is_selected = settings_form_item.value == key;
                        let mut button = widget::button::custom("").height(25).width(25).class
                            (cosmic::theme::style::Button::Custom {
                            active: Box::new(move |_, t| {
                                cosmic::widget::button::Style {
                                    background: Some(Background::Color(colour.into())),
                                    text_color: Some(t.cosmic().palette.neutral_8.into()),
                                    border_radius: Radius::new(2.0),
                                    //border: Some(Border::rounded(1.0)),
                                    ..Default::default()
                                }
                            }),
                            disabled: Box::new(|_| Default::default()),
                            hovered: Box::new(|_, t| Default::default()),
                            pressed: Box::new(|_, t| Default::default()),
                        });
                        // Highlight selected option
                        if is_selected {
                            button = button.class(cosmic::theme::style::Button::Custom {
                                active: Box::new(move |_, t| {
                                    cosmic::widget::button::Style {
                                        background: Some(Background::Color(t.cosmic().accent.base.into())),
                                        text_color: Some(t.cosmic().palette.neutral_8.into()),
                                        //border: Some(Border::rounded(1.0)),
                                        ..Default::default()
                                    }
                                }),
                                disabled: Box::new(|_| Default::default()),
                                hovered: Box::new(|_, t| Default::default()),
                                pressed: Box::new(|_, t| Default::default()),
                            });
                        }

                        let b = button.on_press(Message::SettingsFormUpdate(SettingsFormEvent::StringFieldUpdated(
                            SettingsFormEventValue {
                                settings_window_id: self.settings_window_id,
                                form_value_key,
                                value: ColorUtil::convert_srgba_to_hex_string(colour),
                            },
                        )));
                        row = row.push(b);
                    }

                    column = column.add(row);
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
