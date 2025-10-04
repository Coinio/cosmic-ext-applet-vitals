use crate::app::Message;
use crate::core::app_colours::AppColours;
use cosmic::iced::{window, Background, Color, Radius};
use cosmic::widget;
use cosmic::Element;
use crate::core::settings::{SettingsFormEvent, SettingsFormEventValue};

pub struct FormThemeColourPickerInputProps {
    pub settings_window_id: window::Id,
    pub label: String,
    pub selected_key: String,
    pub form_value_key: &'static str,
    pub app_colours: AppColours,
    pub helper_text: Option<String>,
    pub helper_text_color: Color,
}

pub fn form_theme_colour_picker_input<'a>(
    props: FormThemeColourPickerInputProps,
) -> Element<'a, Message> {
    let FormThemeColourPickerInputProps {
        settings_window_id,
        label,
        selected_key,
        form_value_key,
        app_colours,
        helper_text,
        helper_text_color,
    } = props;

    let mut row = widget::row().spacing(8);

    for (key, colour) in app_colours.colours.into_iter() {
        let is_selected = selected_key == key;

        let mut button = widget::button::custom("")
            .height(25)
            .width(25)
            .class(cosmic::theme::style::Button::Custom {
                active: Box::new(move |_, _| widget::button::Style {
                    background: Some(Background::Color(colour.into())),
                    text_color: None,
                    border_radius: Radius::new(3.0),
                    border_color: if is_selected { Color::WHITE } else { Color::TRANSPARENT },
                    border_width: if is_selected { 3.0 } else { 0.0 },
                    ..Default::default()
                }),
                disabled: Box::new(|_| Default::default()),
                hovered: Box::new(move |_, _| widget::button::Style {
                    background: Some(Background::Color(colour.into())),
                    text_color: None,
                    border_radius: Radius::new(3.0),
                    border_color: if is_selected { Color::WHITE } else { Color::TRANSPARENT },
                    border_width: if is_selected { 3.0 } else { 0.0 },
                    ..Default::default()
                }),
                pressed: Box::new(|_, _| Default::default()),
            });

        button = button.on_press(Message::SettingsFormUpdate(
            SettingsFormEvent::StringFieldUpdated(SettingsFormEventValue {
                settings_window_id,
                form_value_key,
                value: key.to_string(),
            }),
        ));

        row = row.push(button);
    }

    let mut colour_picker_column = widget::column().spacing(8);
    colour_picker_column = colour_picker_column.push(widget::text(label));
    if let Some(helper) = helper_text {
        colour_picker_column = colour_picker_column.push(
            widget::text(helper)
                .size(12)
                .class(cosmic::theme::Text::from(helper_text_color)),
        );
    }
    colour_picker_column = colour_picker_column.push(row.wrap());

    colour_picker_column.into()
}
