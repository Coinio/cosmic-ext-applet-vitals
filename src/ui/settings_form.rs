use crate::app::Message;
use crate::configuration::app_configuration::{
    LABEL_COLOUR_SETTING_KEY, LABEL_TEXT_SETTING_KEY, MAX_SAMPLES_SETTING_KEY, UPDATE_INTERVAL_SETTING_KEY,
};
use crate::configuration::network::{
    NETWORK_RX_LABEL_COLOUR_SETTING_KEY, NETWORK_RX_LABEL_TEXT_SETTING_KEY, NETWORK_TX_LABEL_COLOUR_SETTING_KEY,
    NETWORK_TX_LABEL_TEXT_SETTING_KEY,
};
use crate::fl;
use cosmic::iced::window;
use cosmic::iced_widget::{container, Container};
use cosmic::widget::settings;
use cosmic::{widget, Theme};
use std::collections::HashMap;

// Define the explicit UI order for settings
const ORDERED_KEYS: [&'static str; 8] = [
    LABEL_TEXT_SETTING_KEY,
    NETWORK_RX_LABEL_TEXT_SETTING_KEY,
    NETWORK_RX_LABEL_COLOUR_SETTING_KEY,
    NETWORK_TX_LABEL_TEXT_SETTING_KEY,
    NETWORK_TX_LABEL_COLOUR_SETTING_KEY,
    LABEL_COLOUR_SETTING_KEY,
    UPDATE_INTERVAL_SETTING_KEY,
    MAX_SAMPLES_SETTING_KEY,
];

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
    pub validator: Option<fn(&str) -> Result<(), String>>,
}

pub struct SettingsForm {
    pub settings_window_id: window::Id,
    pub title: String,
    pub values: HashMap<&'static str, SettingsFormItem>,
}

impl SettingsForm {
    pub fn content(&self) -> Container<'_, Message, Theme> {
        let mut column =
            widget::list_column()
                .padding(2)
                .spacing(0)
                .divider_padding(2)
                .add(widget::text(&self.title).font(cosmic::iced::Font {
                    weight: cosmic::iced::font::Weight::ExtraBold,
                    ..Default::default()
                }));

        for &form_value_key in ORDERED_KEYS.iter() {
            if let Some(settings_form) = self.values.get(form_value_key) {
                let text_input =
                    widget::text_input(fl!("settings-empty"), &settings_form.value).on_input(|new_value| {
                        Message::SettingsFormUpdate(SettingsFormEvent::StringFieldUpdated(SettingsFormEventValue {
                            settings_window_id: self.settings_window_id,
                            form_value_key,
                            value: new_value,
                        }))
                    });

                let validator = settings_form.validator.unwrap_or(|_| Ok(()));

                column = column.add(settings::item(
                    settings_form.label.clone(),
                    match validator(&settings_form.value) {
                        Ok(_) => text_input.width(150),
                        Err(error_text) => text_input
                            .width(150)
                            .error(error_text.clone())
                            .helper_text(error_text.clone()),
                    },
                ));
            }
        }

        container(column)
    }
}
