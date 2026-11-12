use crate::app::{AppState, Message};
use crate::core::app_colours::AppColour;
use crate::core::app_text_measurements::indicator_line_height;
use crate::ui::components::svg_icon::{svg_icon, SvgIconProps};
use cosmic::iced::core::text::LineHeight;
use cosmic::iced::widget::{Column, Row};
use cosmic::iced::Alignment;
use cosmic::{widget, Element};

/// The gap between an icon and the text it labels.
const ICON_SPACING: u16 = 2;

/// One of the app's icons, tinted with `colour`. Without a colour the icon keeps its own (white) fill.
pub struct IndicatorIcon {
    pub key: &'static str,
    pub colour: Option<AppColour>,
}

impl IndicatorIcon {
    /// An icon tinted with a configured colour. Unrecognised colours leave the icon untinted.
    pub fn new(key: &'static str, colour: Option<&str>) -> Self {
        Self {
            key,
            colour: colour.and_then(AppColour::parse),
        }
    }
}

pub struct IndicatorLine {
    /// Drawn before the text, e.g. the direction of a data rate. Only drawn in horizontal panels, where there's room.
    pub icon: Option<IndicatorIcon>,
    pub text: String,
}

impl IndicatorLine {
    pub fn text(text: String) -> Self {
        Self { icon: None, text }
    }
}

/// What an indicator shows: an optional icon identifying it, followed by one or more lines of text.
pub struct IndicatorContent {
    pub icon: Option<IndicatorIcon>,
    pub lines: Vec<IndicatorLine>,
    /// The widest text any line can show. Lines are this wide when the indicator size is fixed, so the indicator
    /// doesn't jitter as values change.
    pub max_label_text: &'static str,
}

pub fn indicator<'a>(app_state: &'a AppState, horizontal: bool, content: IndicatorContent) -> Element<'a, Message> {
    let core = app_state.core();
    let font = *app_state.indicator_font();
    let font_size = app_state.font_size(horizontal);
    let line_height = indicator_line_height(font_size);
    let measurements = app_state.app_text_measurements();
    let digit_centre = measurements.digit_centre(font_size, line_height, &font);
    let line_width = if app_state.configuration().general.fix_indicator_size {
        measurements.measure(content.max_label_text, font_size, &font)
    } else {
        None
    };

    let icon = |icon: IndicatorIcon| {
        let props = SvgIconProps {
            icon: app_state.app_icons().get(icon.key),
            size: font_size,
            colour: icon.colour,
        };
        aligned_icon(props, line_height, digit_centre)
    };

    let lines = content.lines.into_iter().map(|line| {
        let text = core
            .applet
            .text(line.text)
            .size(font_size)
            .line_height(LineHeight::Absolute(f32::from(line_height).into()))
            .font(font);
        let mut text = widget::container(text)
            .align_x(if horizontal { Alignment::End } else { Alignment::Center })
            .align_y(Alignment::Center);
        if let Some(width) = line_width {
            text = text.width(width);
        }

        let line_icon = line.icon.filter(|_| horizontal).and_then(icon);
        Row::new()
            .spacing(ICON_SPACING)
            .align_y(Alignment::Center)
            .push_maybe(line_icon)
            .push(text)
            .into()
    });

    let icon = content.icon.and_then(icon);

    if horizontal {
        let lines = Row::from_iter(lines)
            .spacing(core.applet.suggested_padding(true).1)
            .align_y(Alignment::Center);
        Row::new()
            .spacing(ICON_SPACING)
            .align_y(Alignment::Center)
            .push_maybe(icon)
            .push(lines)
            .into()
    } else {
        Column::new()
            .align_x(Alignment::Center)
            .push_maybe(icon)
            .extend(lines)
            .into()
    }
}

/// An icon in a box the height of the text line, so it shares the text's position within the row, offset so its
/// centre lines up with the centre of the rendered digits. When the digit centre is unknown, the icon is centred in
/// the line box instead.
fn aligned_icon<'a>(mut icon: SvgIconProps<'a>, line_height: u16, digit_centre: Option<f32>) -> Option<Element<'a, Message>> {
    let Some(digit_centre) = digit_centre else {
        return svg_icon(icon).map(|el| widget::container(el).height(line_height).align_y(Alignment::Center).into());
    };

    // The gap above the icon, centre - size / 2, is only a whole number of pixels when twice the centre and the size
    // are both even or both odd. Shrink the icon by a pixel when they aren't, so it isn't drawn between pixels.
    let twice_centre = (digit_centre * 2.0).round() as i32;
    let mut size = icon.size as i32;
    if (twice_centre - size) % 2 != 0 {
        size -= 1;
    }
    let top = ((twice_centre - size) / 2).clamp(0, (line_height as i32 - size).max(0));
    icon.size = size.max(1) as u16;

    svg_icon(icon).map(|el| {
        widget::container(el)
            .height(line_height)
            .padding([top as u16, 0, 0, 0])
            .into()
    })
}
