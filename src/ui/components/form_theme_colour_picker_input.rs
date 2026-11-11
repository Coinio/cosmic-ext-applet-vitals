use crate::settings_app::{ColourPickerTarget, Message};
use crate::core::app_colours::{AppColour, PRESET_COLOURS};
use crate::core::settings::{SettingsFormEvent, SettingsFormEventValue};
use crate::fl;
use crate::ui::components::colour_picker::{colour_picker, hue_gradient, ColourPickerState};
use cosmic::iced::{window, Background, Color, Radius};
use cosmic::widget::tooltip::Position;
use cosmic::{widget, Element, Theme};

const SWATCH_SIZE: u16 = 25;

pub struct FormThemeColourPickerInputProps<'a> {
    pub settings_window_id: window::Id,
    pub label: String,
    /// The configuration value of the selected colour.
    pub value: &'a str,
    pub form_value_key: &'static str,
    pub helper_text: Option<String>,
    pub helper_text_color: Color,
    /// The custom colour picker, when it is open for this field.
    pub custom_picker: Option<&'a ColourPickerState>,
}

/// A row of theme colour presets, followed by a swatch that opens a picker for any other colour.
pub fn form_theme_colour_picker_input<'a>(props: FormThemeColourPickerInputProps<'a>) -> Element<'a, Message> {
    let FormThemeColourPickerInputProps {
        settings_window_id,
        label,
        value,
        form_value_key,
        helper_text,
        helper_text_color,
        custom_picker,
    } = props;

    let selected = AppColour::parse(value);
    let mut row = widget::Row::new().spacing(8);

    for preset in PRESET_COLOURS {
        let fill = move |theme: &Theme| Background::Color(preset.resolve(theme.cosmic()));
        let on_press = Message::SettingsFormUpdate(SettingsFormEvent::StringFieldUpdated(SettingsFormEventValue {
            settings_window_id,
            form_value_key,
            value: preset.to_config_value(),
        }));
        let button = swatch(fill, selected == Some(preset) && custom_picker.is_none(), on_press);

        row = row.push(match preset {
            AppColour::ThemeAccent => tooltip(button, fl!("settings-colour-theme-accent")),
            _ => button,
        });
    }

    // Colours that aren't presets are shown in the custom swatch. That includes palette colours offered by earlier
    // versions, so they still show as selected.
    let custom_colour = selected.filter(|colour| !colour.is_preset());
    let custom_fill = move |theme: &Theme| match custom_colour {
        Some(colour) => Background::Color(colour.resolve(theme.cosmic())),
        None => hue_gradient(),
    };
    let toggle_picker = Message::ToggleCustomColourPicker(ColourPickerTarget {
        settings_window_id,
        form_value_key,
    });
    row = row.push(tooltip(
        swatch(custom_fill, custom_colour.is_some() || custom_picker.is_some(), toggle_picker),
        fl!("settings-colour-custom"),
    ));

    let mut column = widget::Column::new().spacing(8);
    column = column.push(widget::text(label));
    if let Some(helper) = helper_text {
        column = column.push(
            widget::text(helper)
                .size(12)
                .class(cosmic::theme::Text::from(helper_text_color)),
        );
    }
    column = column.push(row.wrap());

    if let Some(state) = custom_picker {
        column = column.push(widget::container(colour_picker(state)).class(cosmic::theme::Container::Card));
    }

    column.into()
}

fn swatch<'a>(fill: impl Fn(&Theme) -> Background + Copy + 'static, selected: bool, on_press: Message) -> Element<'a, Message> {
    let style = move |theme: &Theme| widget::button::Style {
        background: Some(fill(theme)),
        text_color: None,
        border_radius: Radius::new(3.0),
        border_color: if selected { theme.cosmic().background(theme.transparent).on.into() } else { Color::TRANSPARENT },
        border_width: if selected { 3.0 } else { 0.0 },
        ..Default::default()
    };

    widget::button::custom("")
        .height(SWATCH_SIZE)
        .width(SWATCH_SIZE)
        .class(cosmic::theme::style::Button::Custom {
            active: Box::new(move |_, theme| style(theme)),
            disabled: Box::new(move |theme| style(theme)),
            hovered: Box::new(move |_, theme| style(theme)),
            pressed: Box::new(move |_, theme| style(theme)),
        })
        .on_press(on_press)
        .into()
}

fn tooltip<'a>(content: Element<'a, Message>, text: String) -> Element<'a, Message> {
    widget::tooltip(content, widget::text(text), Position::Bottom).into()
}
