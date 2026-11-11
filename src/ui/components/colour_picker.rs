use crate::settings_app::Message;
use crate::core::app_colours::{parse_hex, to_hex};
use crate::fl;
use crate::ui::components::gradient_pad::GradientPad;
use cosmic::cosmic_theme::palette::{FromColor, Hsv, Srgb};
use cosmic::iced::gradient::Linear;
use cosmic::iced::{Alignment, Background, Color, Gradient, Length, Radians};
use cosmic::{widget, Element};
use std::f32::consts::{FRAC_PI_2, PI};

/// The gradient stops a linear gradient supports.
const MAX_GRADIENT_STOPS: u16 = 8;

#[derive(Debug, Clone)]
pub enum ColourPickerEvent {
    /// The saturation and value, each from 0 to 1.
    SaturationValue(f32, f32),
    /// The hue in degrees, from 0 to 360.
    Hue(f32),
    /// The hex colour text was edited.
    Input(String),
    Apply,
    Cancel,
}

/// The state of a custom colour picker while it is open.
pub struct ColourPickerState {
    /// Kept separately from the colour so the hue isn't lost when the saturation or value is dragged to zero.
    hue: f32,
    saturation: f32,
    value: f32,
    /// The hex colour text, which may be part way through being typed.
    input: String,
}

impl ColourPickerState {
    pub fn new(colour: Color) -> Self {
        let mut state = ColourPickerState {
            hue: 0.0,
            saturation: 0.0,
            value: 0.0,
            input: to_hex(colour),
        };
        state.set_colour(colour);
        state
    }

    pub fn colour(&self) -> Color {
        hsv_colour(self.hue, self.saturation, self.value)
    }

    /// Applies an edit. `Apply` and `Cancel` close the picker, so they are handled by its owner.
    pub fn update(&mut self, event: ColourPickerEvent) {
        match event {
            ColourPickerEvent::SaturationValue(saturation, value) => {
                self.saturation = saturation;
                self.value = value;
                self.input = to_hex(self.colour());
            }
            ColourPickerEvent::Hue(hue) => {
                self.hue = hue;
                self.input = to_hex(self.colour());
            }
            ColourPickerEvent::Input(input) => {
                if let Some(colour) = parse_hex(&input) {
                    self.set_colour(colour);
                }
                self.input = input;
            }
            ColourPickerEvent::Apply | ColourPickerEvent::Cancel => {}
        }
    }

    fn set_colour(&mut self, colour: Color) {
        let hsv = Hsv::from_color(Srgb::new(colour.r, colour.g, colour.b));
        // Greys have no hue, so keep the one already chosen.
        if hsv.saturation > 0.0 && hsv.value > 0.0 {
            self.hue = hsv.hue.into_positive_degrees();
        }
        self.saturation = hsv.saturation;
        self.value = hsv.value;
    }
}

/// A saturation/value square and a hue strip, with a hex input to type a colour.
pub fn colour_picker<'a>(state: &'a ColourPickerState) -> Element<'a, Message> {
    let event = |event: ColourPickerEvent| Message::CustomColourPicker(event);
    let colour = state.colour();
    let pure_hue = hsv_colour(state.hue, 1.0, 1.0);

    // Saturation increases from left to right and value from bottom to top: the pure hue, faded to white on the
    // left and to black at the bottom.
    let saturation_value = GradientPad::new(
        vec![
            Background::Color(pure_hue),
            linear(FRAC_PI_2, [Color::WHITE, Color { a: 0.0, ..Color::WHITE }]),
            linear(PI, [Color { a: 0.0, ..Color::BLACK }, Color::BLACK]),
        ],
        (state.saturation, 1.0 - state.value),
        colour,
        move |x, y| event(ColourPickerEvent::SaturationValue(x, 1.0 - y)),
    );

    let hue = GradientPad::new(vec![hue_gradient()], (state.hue / 360.0, 0.5), pure_hue, move |x, _| {
        event(ColourPickerEvent::Hue(x * 360.0))
    })
    .height(16);

    let preview = widget::container(widget::Space::new(32, 32)).class(cosmic::theme::Container::custom(move |theme| {
        widget::container::Style {
            background: Some(Background::Color(colour)),
            border: cosmic::iced::Border {
                radius: theme.cosmic().corner_radii.radius_s.into(),
                width: 1.0,
                color: theme.cosmic().palette.neutral_5.into(),
            },
            ..Default::default()
        }
    }));

    let input = widget::text_input("#RRGGBB", &state.input)
        .on_input(move |text| event(ColourPickerEvent::Input(text)))
        .on_submit(move |_| event(ColourPickerEvent::Apply))
        .width(Length::Fixed(120.0));

    let buttons = widget::row()
        .spacing(8)
        .push(widget::button::standard(fl!("settings-colour-picker-cancel")).on_press(event(ColourPickerEvent::Cancel)))
        .push(widget::button::suggested(fl!("settings-colour-picker-save")).on_press(event(ColourPickerEvent::Apply)));

    let controls = widget::row()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(preview)
        .push(input)
        .push(widget::horizontal_space())
        .push(buttons);

    widget::column()
        .spacing(12)
        .padding(12)
        .max_width(400)
        .push(saturation_value)
        .push(hue)
        .push(controls)
        .into()
}

/// The full range of hues from left to right, as used for the hue strip and the custom colour swatch.
pub fn hue_gradient() -> Background {
    let last = f32::from(MAX_GRADIENT_STOPS - 1);
    let gradient = (0..MAX_GRADIENT_STOPS).fold(Linear::new(Radians(FRAC_PI_2)), |gradient, stop| {
        let offset = f32::from(stop) / last;
        gradient.add_stop(offset, hsv_colour(offset * 360.0, 1.0, 1.0))
    });
    Background::Gradient(Gradient::Linear(gradient))
}

/// A gradient between two colours. An angle of zero runs bottom to top, and it turns clockwise.
fn linear(angle: f32, [from, to]: [Color; 2]) -> Background {
    Background::Gradient(Gradient::Linear(
        Linear::new(Radians(angle)).add_stop(0.0, from).add_stop(1.0, to),
    ))
}

fn hsv_colour(hue: f32, saturation: f32, value: f32) -> Color {
    let rgb = Srgb::from_color(Hsv::new(hue, saturation, value));
    Color::from_rgb(rgb.red, rgb.green, rgb.blue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_hue_when_dragged_to_grey() {
        let mut state = ColourPickerState::new(Color::from_rgb8(0, 0, 255));
        state.update(ColourPickerEvent::SaturationValue(0.0, 0.5));
        state.update(ColourPickerEvent::SaturationValue(1.0, 1.0));
        assert_eq!(to_hex(state.colour()), "#0000FF");
    }

    #[test]
    fn typing_a_hex_colour_moves_the_picker() {
        let mut state = ColourPickerState::new(Color::BLACK);
        state.update(ColourPickerEvent::Input("#FF8000".to_string()));
        assert_eq!(to_hex(state.colour()), "#FF8000");
    }

    #[test]
    fn partial_hex_input_is_kept_without_changing_the_colour() {
        let mut state = ColourPickerState::new(Color::from_rgb8(255, 0, 0));
        state.update(ColourPickerEvent::Input("#00".to_string()));
        assert_eq!(state.input, "#00");
        assert_eq!(to_hex(state.colour()), "#FF0000");
    }

    #[test]
    fn dragging_updates_the_hex_input() {
        let mut state = ColourPickerState::new(Color::from_rgb8(255, 0, 0));
        state.update(ColourPickerEvent::SaturationValue(1.0, 0.0));
        assert_eq!(state.input, "#000000");
    }
}
