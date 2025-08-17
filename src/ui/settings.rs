use crate::app::{Message};
use crate::fl;
use cosmic::iced::window;
use cosmic::iced_widget::{column, container, Container};
use cosmic::widget::settings;
use cosmic::{widget, Theme};
use std::collections::HashMap;

pub const LABEL_TEXT_SETTING_KEY: &'static str = "settings-label-text";
pub const LABEL_COLOUR_SETTING_KEY: &'static str = "settings-label-colour";
pub const UPDATE_INTERVAL_SETTING_KEY: &'static str = "settings-update-interval";
pub const MAX_SAMPLES_SETTING_KEY: &'static str = "settings-max-samples";

#[derive(Debug, Clone)]
pub enum SettingsFormEvent {
    StringFieldUpdated(SettingsFormEventValue),
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
}

pub struct SettingsForm {
    pub settings_window_id: window::Id,
    pub values: HashMap<&'static str, SettingsFormItem>,
}

impl SettingsForm {
    pub fn new(settings_window_id: window::Id, values: HashMap<&'static str, SettingsFormItem>) -> Self {
        Self { settings_window_id, values }
    }

    pub fn content(&self) -> Container<'_, Message, Theme> {
        let title = fl!("settings-cpu-title");

        let mut column = column![widget::text(title).font(cosmic::iced::Font {
            weight: cosmic::iced::font::Weight::ExtraBold,
            ..Default::default()
        })];

        for (form_value_key, settings_form) in self.values.iter() {
            column = column.push(settings::item(
                settings_form.label.clone(),
                widget::text_input(fl!("settings-empty"), &settings_form.value).on_input(|new_value| {
                    Message::SettingsFormUpdate(SettingsFormEvent::StringFieldUpdated(SettingsFormEventValue {
                        settings_window_id: self.settings_window_id,
                        form_value_key,
                        value: new_value,
                    }))
                }),
            ));
        }

        container(column)
    }
}
