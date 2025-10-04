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
    pub helper_text: Option<String>,
    pub helper_text_color: cosmic::iced::Color,
}

pub fn form_checkbox_input<'a>(props: FormCheckboxInputProps) -> Element<'a, Message> {
    let FormCheckboxInputProps {
        settings_window_id,
        label,
        value,
        form_value_key,
        helper_text,
        helper_text_color,
    } = props;

    let checkbox_input = widget::checkbox("", value).on_toggle(move |new_value| {
        Message::SettingsFormUpdate(SettingsFormEvent::CheckBoxUpdated(SettingsFormEventValue {
            settings_window_id,
            form_value_key,
            value: new_value.to_string(),
        }))
    });

    let mut column = widget::column().spacing(6);
    column = column.push(settings::item(label, checkbox_input));

    if let Some(helper) = helper_text {
        let helper_text_widget = widget::text(helper)
            .size(12)
            .class(cosmic::theme::Text::from(helper_text_color));
        column = column.push(widget::container(helper_text_widget).width(cosmic::iced::Length::Fill));
    }

    column.into()
}
