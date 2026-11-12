use crate::core::settings::{SettingsFormEvent, SettingsFormEventValue};
use crate::settings_app::Message;
use cosmic::iced::{window, Color, Length};
use cosmic::widget::{self, settings};
use cosmic::Element;
use std::sync::Arc;

pub struct FormDropdownInputProps<'a> {
    pub settings_window_id: window::Id,
    pub label: String,
    pub value: &'a str,
    pub form_value_key: &'static str,
    /// The option labels shown in the dropdown.
    pub labels: &'a [String],
    /// The form value of each option, parallel to `labels`.
    pub values: Arc<[String]>,
    pub helper_text: Option<String>,
    pub helper_text_color: Color,
}

/// A dropdown that sets the form value to the chosen option's value.
pub fn form_dropdown_input<'a>(props: FormDropdownInputProps<'a>) -> Element<'a, Message> {
    let FormDropdownInputProps {
        settings_window_id,
        label,
        value,
        form_value_key,
        labels,
        values,
        helper_text,
        helper_text_color,
    } = props;

    let selected = values.iter().position(|option| option == value);

    // Shared rather than cloned on every render, as there can be hundreds of options, e.g. fonts.
    let dropdown = widget::dropdown(labels, selected, move |index| {
        Message::SettingsFormUpdate(SettingsFormEvent::StringFieldUpdated(SettingsFormEventValue {
            settings_window_id,
            form_value_key,
            value: values[index].clone(),
        }))
    });

    let mut column = widget::Column::new().spacing(6);
    column = column.push(settings::item(label, dropdown));

    if let Some(helper) = helper_text {
        let helper_text_widget = widget::text(helper)
            .size(12)
            .class(cosmic::theme::Text::from(helper_text_color));
        column = column.push(widget::container(helper_text_widget).width(Length::Fill));
    }

    column.into()
}
