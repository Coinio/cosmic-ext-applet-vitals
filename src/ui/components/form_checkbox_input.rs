use crate::app::Message;
use cosmic::iced::window;
use cosmic::widget::{self, settings};
use cosmic::Element;
use crate::core::settings::{SettingsFormEvent, SettingsFormEventValue};

pub struct FormCheckboxInputProps {
    pub settings_window_id: window::Id,
    pub label: String,
    pub value: bool,
    pub form_value_key: &'static str,
}

pub fn form_checkbox_input<'a>(props: FormCheckboxInputProps) -> Element<'a, Message> {
    let FormCheckboxInputProps {
        settings_window_id,
        label,
        value,
        form_value_key,
    } = props;

    let checkbox_input = widget::checkbox("", value).on_toggle(move |new_value| {
        Message::SettingsFormUpdate(SettingsFormEvent::CheckBoxUpdated(SettingsFormEventValue {
            settings_window_id,
            form_value_key,
            value: new_value.to_string(),
        }))
    });

    settings::item(label, checkbox_input).into()
}
