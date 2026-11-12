use crate::app::Message;
use crate::core::app_colours::AppColour;
use crate::core::app_icons::AppIcon;
use cosmic::font::Font;
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::font::Weight;
use cosmic::iced::widget::{container, progress_bar, text, Column, Row};
use cosmic::iced::{Alignment, Border, Color, Length};
use cosmic::style::Svg;
use cosmic::theme::{Container, ProgressBar};
use cosmic::widget::svg;
use cosmic::{widget, Element, Renderer, Theme};
use std::borrow::Cow;
use std::rc::Rc;

/// The padding around the content of a detail popup.
const POPUP_PADDING: u16 = 12;
/// The spacing between the header and each card of a detail popup.
const CARD_SPACING: u16 = 8;
/// The padding inside each card.
const CARD_PADDING: [u16; 2] = [10, 12];
/// The spacing between the rows of a card.
pub const ROW_SPACING: u16 = 6;
/// The spacing between a heading and what it heads, e.g. a label and its bar.
const HEADING_SPACING: u16 = 4;

/// The thickness of the bars in a detail popup.
pub const BAR_GIRTH: u16 = 6;
/// The size of the icon in the header's tile.
const HEADER_ICON_SIZE: u16 = 16;
/// The padding around the header's icon, which makes its tile.
const HEADER_ICON_PADDING: u16 = 7;
/// How opaque the header's icon tile is, over the popup's background.
const HEADER_TILE_ALPHA: f32 = 0.18;
/// How opaque the track behind a bar is, in the popup's text colour.
const BAR_TRACK_ALPHA: f32 = 0.12;
/// How opaque secondary text is, such as captions and section headings.
const DIMMED_ALPHA: f32 = 0.65;

const TITLE_SIZE: f32 = 16.0;
const SUMMARY_SIZE: f32 = 15.0;
const STAT_SIZE: f32 = 17.0;
const BODY_SIZE: f32 = 13.0;
const CAPTION_SIZE: f32 = 11.0;
const SECTION_HEADING_SIZE: f32 = 10.0;

type Text<'a> = widget::Text<'a, Theme, Renderer>;

/// The typography of the detail popups, in the font configured for the indicators: bold headers and labels, with
/// regular values and dimmed captions.
#[derive(Debug, Clone, Copy)]
pub struct PopupStyle {
    font: Font,
}

impl PopupStyle {
    pub fn new(font: Font) -> Self {
        Self { font }
    }

    fn text<'a>(self, text: impl Into<Cow<'a, str>> + 'a, size: f32, weight: Weight) -> Text<'a> {
        widget::text(text).size(size).font(Font { weight, ..self.font })
    }

    /// The popup's title, in its header.
    pub fn title<'a>(self, text: impl Into<Cow<'a, str>> + 'a) -> Text<'a> {
        self.text(text, TITLE_SIZE, Weight::Bold)
    }

    /// The headline value in the popup's header, e.g. the total cpu usage, in the indicator's colour.
    pub fn summary<'a>(self, text: impl Into<Cow<'a, str>> + 'a, colour: Option<AppColour>) -> Text<'a> {
        self.text(text, SUMMARY_SIZE, Weight::Bold)
            .class(cosmic::theme::Text::Color(resolve_or_accent(colour)))
    }

    /// A small, dimmed, upper case heading over a section or a stat, e.g. "THREADS".
    pub fn section_heading<'a>(self, text: impl AsRef<str>) -> Text<'a> {
        self.text(text.as_ref().to_uppercase(), SECTION_HEADING_SIZE, Weight::Bold)
            .class(cosmic::theme::Text::Custom(dimmed))
    }

    /// The label of a row, e.g. a mount point or an interface name.
    pub fn label<'a>(self, text: impl Into<Cow<'a, str>> + 'a) -> Text<'a> {
        self.text(text, BODY_SIZE, Weight::Bold)
    }

    /// The value of a row.
    pub fn value<'a>(self, text: impl Into<Cow<'a, str>> + 'a) -> Text<'a> {
        self.text(text, BODY_SIZE, Weight::Medium)
    }

    /// A prominent value under a [`Self::section_heading`], e.g. the cpu temperature.
    pub fn stat<'a>(self, text: impl Into<Cow<'a, str>> + 'a) -> Text<'a> {
        self.text(text, STAT_SIZE, Weight::Bold)
    }

    /// Secondary, dimmed text, e.g. a disk's model.
    pub fn caption<'a>(self, text: impl Into<Cow<'a, str>> + 'a) -> Text<'a> {
        self.text(text, CAPTION_SIZE, Weight::Normal)
            .class(cosmic::theme::Text::Custom(dimmed))
    }

    /// A small number beside a bar, e.g. a thread number or a percentage.
    pub fn figure<'a>(self, text: impl Into<Cow<'a, str>> + 'a) -> Text<'a> {
        self.text(text, CAPTION_SIZE, Weight::Semibold)
    }
}

/// The text colour of the popup, dimmed.
fn dimmed(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(with_alpha(theme.cosmic().background(theme.transparent).on.into(), DIMMED_ALPHA)),
        ..text::Catalog::style(theme, &cosmic::theme::Text::Default)
    }
}

fn with_alpha(colour: Color, alpha: f32) -> Color {
    Color { a: colour.a * alpha, ..colour }
}

/// The colour resolved against the active theme, or the theme's accent colour when there isn't one.
fn resolve_or_accent(colour: Option<AppColour>) -> Color {
    colour.unwrap_or(AppColour::ThemeAccent).resolve(cosmic::theme::active().cosmic())
}

/// The content of the popup shown when hovering over an indicator: a header, followed by its cards.
pub fn detail_popup<'a>(
    header: Element<'a, Message>,
    cards: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    Column::new()
        .spacing(CARD_SPACING)
        .padding(POPUP_PADDING)
        .push(header)
        .extend(cards)
        .into()
}

/// The popup's title, after the indicator's icon in a tile of its colour when it has one, with a summary at the
/// other end, e.g. the total usage.
pub fn popup_header<'a>(
    style: PopupStyle,
    icon: Option<&'a AppIcon>,
    colour: Option<AppColour>,
    title: String,
    summary: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let tile = icon.map(|icon| {
        container(popup_icon(icon, HEADER_ICON_SIZE, Some(colour.unwrap_or(AppColour::ThemeAccent))))
            .padding(HEADER_ICON_PADDING)
            .class(Container::custom(move |theme| {
                let cosmic = theme.cosmic();
                let tint = colour.unwrap_or(AppColour::ThemeAccent).resolve(cosmic);
                container::Style {
                    background: Some(with_alpha(tint, HEADER_TILE_ALPHA).into()),
                    border: Border {
                        radius: cosmic.corner_radii.radius_s.into(),
                        ..Border::default()
                    },
                    ..container::Style::default()
                }
            }))
    });

    Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        // Keeps the header the same height with or without a tile.
        .height(Length::Fixed(f32::from(HEADER_ICON_SIZE + 2 * HEADER_ICON_PADDING)))
        .push_maybe(tile)
        .push(style.title(title).width(Length::Fill))
        .push_maybe(summary)
        .into()
}

/// A raised, rounded group of related details.
pub fn card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .width(Length::Fill)
        .padding(CARD_PADDING)
        .class(Container::custom(|theme| {
            let cosmic = theme.cosmic();
            container::Style {
                background: Some(Color::from(cosmic.background(theme.transparent).component.base).into()),
                border: Border {
                    radius: cosmic.corner_radii.radius_s.into(),
                    width: 1.0,
                    color: with_alpha(cosmic.background(theme.transparent).component.divider.into(), 0.5),
                },
                ..container::Style::default()
            }
        }))
        .into()
}

/// A [`card`] of `rows`, under a section heading when it has one.
pub fn card_with_heading<'a>(
    style: PopupStyle,
    heading: Option<String>,
    rows: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let rows = Column::with_children(rows).spacing(ROW_SPACING);
    card(
        Column::new()
            .spacing(8)
            .push_maybe(heading.map(|heading| style.section_heading(heading)))
            .push(rows),
    )
}

/// An icon in `colour`, or in the popup's text colour when there isn't one, so it shows in light and dark themes.
pub fn popup_icon<'a>(icon: &'a AppIcon, size: u16, colour: Option<AppColour>) -> Element<'a, Message> {
    widget::icon::icon(icon.handle.clone())
        .size(size)
        .width(Length::Fixed(f32::from(icon.width(size))))
        .class(Svg::Custom(Rc::new(move |theme| {
            let mut style = svg::Catalog::style(theme, &Svg::default(), svg::Status::Idle);
            let cosmic = theme.cosmic();
            style.color = Some(colour.map_or_else(|| cosmic.background(theme.transparent).on.into(), |colour| colour.resolve(cosmic)));
            style
        })))
        .into()
}

/// A bar in the indicator's colour, or the theme's accent colour, over a faint track.
pub fn coloured_bar_class(colour: Option<AppColour>) -> ProgressBar {
    ProgressBar::custom(move |theme| {
        let cosmic = theme.cosmic();
        progress_bar::Style {
            background: with_alpha(cosmic.background(theme.transparent).on.into(), BAR_TRACK_ALPHA).into(),
            bar: colour.unwrap_or(AppColour::ThemeAccent).resolve(cosmic).into(),
            border: Border {
                radius: cosmic.corner_radii.radius_xl.into(),
                ..Border::default()
            },
        }
    })
}

/// A bar filled to `fraction`.
pub fn bar<'a>(fraction: f32, colour: Option<AppColour>) -> Element<'a, Message> {
    progress_bar(0.0..=1.0, fraction.clamp(0.0, 1.0))
        .girth(BAR_GIRTH)
        .class(coloured_bar_class(colour))
        .into()
}

/// A heading on the left, e.g. a name, with a caption describing it on the right, e.g. its address.
pub fn heading_row<'a>(style: PopupStyle, label: impl Into<String>, caption: Option<String>) -> Element<'a, Message> {
    Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(style.label(label.into()).width(Length::Fill))
        .push_maybe(caption.map(|caption| style.caption(caption)))
        .into()
}

/// Stats side by side, each a dimmed heading over a prominent value, sharing the width equally.
pub fn stat_tiles<'a>(style: PopupStyle, stats: impl IntoIterator<Item = (String, String)>) -> Element<'a, Message> {
    let tiles = stats.into_iter().map(|(heading, value)| {
        Column::new()
            .spacing(2)
            .width(Length::Fill)
            .push(style.section_heading(heading))
            .push(style.stat(value))
            .into()
    });

    Row::with_children(tiles).spacing(12).into()
}

/// A bold label and a caption describing it, e.g. how much of something is used, above a bar filled to `fraction`.
pub fn labelled_bar<'a>(
    style: PopupStyle,
    label: impl Into<String>,
    caption: impl Into<String>,
    fraction: f32,
    colour: Option<AppColour>,
) -> Element<'a, Message> {
    let label = Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(style.label(label.into()).width(Length::Fill))
        .push(style.value(caption.into()));

    Column::new()
        .spacing(HEADING_SPACING)
        .push(label)
        .push(bar(fraction, colour))
        .into()
}

/// A bar between a leading label, e.g. a thread number, and an optional trailing percentage, in fixed widths so the
/// bars of a column line up.
pub fn inline_bar<'a>(
    style: PopupStyle,
    leading: Text<'a>,
    leading_width: f32,
    percent: Option<f64>,
    fraction: f32,
    colour: Option<AppColour>,
) -> Element<'a, Message> {
    Row::new()
        .spacing(6)
        .align_y(Alignment::Center)
        .push(leading.width(Length::Fixed(leading_width)))
        .push(bar(fraction, colour))
        .push_maybe(percent.map(|percent| {
            style
                .figure(format!("{:.0}%", percent.clamp(0.0, 100.0)))
                .width(Length::Fixed(PERCENT_WIDTH))
                .align_x(Horizontal::Right)
        }))
        .into()
}

/// The width of a percentage after an [`inline_bar`], which fits "100%".
const PERCENT_WIDTH: f32 = 32.0;

/// How much of `total` is `used`, from 0 to 1. Nothing is used of an empty total.
pub fn used_fraction(used: u64, total: u64) -> f32 {
    if total == 0 {
        return 0.0;
    }
    (used.min(total) as f64 / total as f64) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn used_fraction_is_clamped_to_the_total() {
        assert_eq!(used_fraction(750, 1000), 0.75);
        assert_eq!(used_fraction(0, 1000), 0.0);
        assert_eq!(used_fraction(2000, 1000), 1.0);
    }

    #[test]
    fn used_fraction_of_an_empty_total_is_zero() {
        assert_eq!(used_fraction(0, 0), 0.0);
    }
}
