use crate::settings_app::Message;
use crate::core::app_fonts::FontOptions;
use crate::core::settings::{SettingsFormEvent, SettingsFormEventValue};
use cosmic::iced::{window, Color, Length};
use cosmic::widget::{self, settings};
use cosmic::Element;

pub struct FormFontDropdownInputProps<'a> {
    pub settings_window_id: window::Id,
    pub label: String,
    pub value: &'a str,
    pub form_value_key: &'static str,
    pub options: &'a FontOptions,
    pub helper_text: Option<String>,
    pub helper_text_color: Color,
}

pub fn form_font_dropdown_input<'a>(props: FormFontDropdownInputProps<'a>) -> Element<'a, Message> {
    let FormFontDropdownInputProps {
        settings_window_id,
        label,
        value,
        form_value_key,
        options,
        helper_text,
        helper_text_color,
    } = props;

    // Shared rather than cloned on every render, as there can be hundreds of fonts.
    let values = options.values.clone();

    let dropdown = widget::dropdown(options.labels.as_slice(), options.position(value), move |index| {
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
