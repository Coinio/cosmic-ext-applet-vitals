use crate::settings_app::Message;
use crate::core::settings::{SettingsFormEvent, SettingsFormEventValue};
use cosmic::iced::{window, Color, Length};
use cosmic::widget::{self, settings};
use cosmic::Element;

pub struct FormSpinButtonInputProps {
    pub settings_window_id: window::Id,
    pub label: String,
    pub value: u16,
    pub min: u16,
    pub max: u16,
    pub form_value_key: &'static str,
    pub helper_text: Option<String>,
    pub helper_text_color: Color,
}

pub fn form_spin_button_input<'a>(props: FormSpinButtonInputProps) -> Element<'a, Message> {
    let FormSpinButtonInputProps {
        settings_window_id,
        label,
        value,
        min,
        max,
        form_value_key,
        helper_text,
        helper_text_color,
    } = props;

    let value = value.clamp(min, max);

    let spin_button = widget::spin_button(value.to_string(), value, 1, min, max, move |new_value| {
        Message::SettingsFormUpdate(SettingsFormEvent::StringFieldUpdated(SettingsFormEventValue {
            settings_window_id,
            form_value_key,
            value: new_value.to_string(),
        }))
    });

    let mut column = widget::Column::new().spacing(6);
    column = column.push(settings::item(label, spin_button));

    if let Some(helper) = helper_text {
        let helper_text_widget = widget::text(helper)
            .size(12)
            .class(cosmic::theme::Text::from(helper_text_color));
        column = column.push(widget::container(helper_text_widget).width(Length::Fill));
    }

    column.into()
}
