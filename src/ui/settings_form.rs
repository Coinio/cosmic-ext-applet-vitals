use crate::app::Message;
use crate::fl;
use cosmic::iced::window;
use cosmic::iced_widget::{container, Container};
use cosmic::widget::settings;
use cosmic::{widget, Theme};
use std::collections::BTreeMap;

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
    pub order: Option<u32>,
}

pub struct SettingsForm {
    pub settings_window_id: window::Id,
    pub title: String,
    pub values: BTreeMap<&'static str, SettingsFormItem>,
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

        let mut items: Vec<(&'static str, &SettingsFormItem)> = self.values.iter().map(|(k, v)| (*k, v)).collect();

        items.sort_by(|a, b| match (a.1.order, b.1.order) {
            (Some(oa), Some(ob)) => oa.cmp(&ob).then_with(|| a.0.cmp(&b.0)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(&b.0),
        });

        for (form_value_key, settings_form) in items.into_iter() {
            let text_input = widget::text_input(fl!("settings-empty"), &settings_form.value).on_input(|new_value| {
                Message::SettingsFormUpdate(SettingsFormEvent::StringFieldUpdated(SettingsFormEventValue {
                    settings_window_id: self.settings_window_id,
                    form_value_key,
                    value: new_value,
                }))
            });

            let validator = settings_form.validator.unwrap_or(|_| Ok(()));

            let field = match validator(&settings_form.value) {
                Ok(_) => text_input.width(150),
                Err(error_text) => text_input
                    .width(150)
                    .error(error_text.clone())
                    .helper_text(error_text.clone()),
            };

            column = column.add(settings::item(settings_form.label.clone(), field));
        }

        container(column)
    }
}
