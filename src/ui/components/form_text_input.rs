use crate::app::Message;
use crate::fl;
use cosmic::iced::{window, Color};
use cosmic::widget::{self, settings};
use cosmic::Element;
use crate::core::settings::{SettingsFormEvent, SettingsFormEventValue};

pub struct FormTextInputProps<'a> {
    pub settings_window_id: window::Id,
    pub label: String,
    pub value: &'a str,
    pub form_value_key: &'static str,
    pub validator: Option<fn(&str) -> Result<(), String>>,
    pub error_color: Color,
    pub helper_text: Option<String>,
    pub helper_text_color: Color,
}

pub fn form_text_input<'a>(props: FormTextInputProps<'a>) -> Element<'a, Message> {
    let FormTextInputProps {
        settings_window_id,
        label,
        value,
        form_value_key,
        validator,
        error_color,
        helper_text,
        helper_text_color,
    } = props;

    let input_id = cosmic::iced::widget::text_input::Id::new(format!(
        "settings_input::{:?}::{}",
        settings_window_id, form_value_key
    ));

    let text_input = widget::text_input(fl!("settings-empty"), value)
        .id(input_id.into())
        .on_input(move |new_value| {
            Message::SettingsFormUpdate(SettingsFormEvent::StringFieldUpdated(
                SettingsFormEventValue {
                    settings_window_id,
                    form_value_key,
                    value: new_value,
                },
            ))
        });

    let validator = validator.unwrap_or(|_| Ok(()));
    let validation_result = validator(value);

    let field = match &validation_result {
        Ok(_) => text_input.width(150),
        Err(error_text) => text_input.width(150).error(error_text.clone()),
    };

    let mut column = widget::column().spacing(6);
    column = column.push(settings::item(label, field));

    if let Some(helper) = helper_text {
        let helper_text_widget = widget::text(helper)
            .size(12)
            .class(cosmic::theme::Text::from(helper_text_color));
        column = column.push(widget::container(helper_text_widget).width(cosmic::iced::Length::Fill));
    }

    if let Err(error_text) = validation_result {
        let error_text_widget = widget::text(error_text)
            .size(12)
            .class(cosmic::theme::Text::from(error_color));

        column = column.push(widget::container(error_text_widget).width(cosmic::iced::Length::Fill));
    }

    column.into()
}
