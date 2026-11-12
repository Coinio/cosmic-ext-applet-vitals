use crate::app::Message;
use crate::core::app_colours::AppColour;
use cosmic::iced::widget::progress_bar::Catalog;
use cosmic::iced::widget::{progress_bar, Column, Row};
use cosmic::iced::{Alignment, Length};
use cosmic::theme::ProgressBar;
use cosmic::{widget, Element};

/// The spacing between the title and each section of a detail popup.
const SECTION_SPACING: u16 = 12;

/// The spacing between the rows of a section.
pub const ROW_SPACING: u16 = 4;

/// The thickness of the bar under a [`labelled_bar`].
const LABELLED_BAR_GIRTH: u16 = 6;

/// The content of the popup shown when hovering over an indicator: a title, followed by its sections.
pub fn detail_popup<'a>(
    title: String,
    sections: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    Column::new()
        .spacing(SECTION_SPACING)
        .padding([12, 16])
        .push(widget::text::heading(title))
        .extend(sections)
        .into()
}

/// The theme's progress bar, with the bar drawn in the indicator's icon colour when one is set.
pub fn coloured_bar_class(colour: Option<AppColour>) -> ProgressBar {
    match colour {
        None => ProgressBar::Primary,
        Some(colour) => ProgressBar::custom(move |theme| {
            let mut style = theme.style(&ProgressBar::Primary);
            style.bar = colour.resolve(theme.cosmic()).into();
            style
        }),
    }
}

/// A label on the left, with its value on the right.
pub fn detail_row<'a>(label: impl Into<String>, value: impl Into<String>) -> Element<'a, Message> {
    Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(widget::text::body(label.into()).width(Length::Fill))
        .push(widget::text::body(value.into()))
        .into()
}

/// A section of [`detail_row`]s.
pub fn detail_rows<'a>(rows: impl IntoIterator<Item = (String, String)>) -> Element<'a, Message> {
    Column::with_children(rows.into_iter().map(|(label, value)| detail_row(label, value)))
        .spacing(ROW_SPACING)
        .into()
}

/// A label and a caption describing it, e.g. how much of something is used, above a bar filled to `fraction`.
pub fn labelled_bar<'a>(
    label: impl Into<String>,
    caption: impl Into<String>,
    fraction: f32,
    colour: Option<AppColour>,
) -> Element<'a, Message> {
    let label = Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(widget::text::body(label.into()).width(Length::Fill))
        .push(widget::text::caption(caption.into()));

    let bar = progress_bar(0.0..=1.0, fraction.clamp(0.0, 1.0))
        .girth(LABELLED_BAR_GIRTH)
        .class(coloured_bar_class(colour));

    Column::new().spacing(ROW_SPACING).push(label).push(bar).into()
}

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
