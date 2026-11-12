use crate::app::Message;
use crate::core::app_colours::AppColour;
use cosmic::iced::widget::progress_bar::Catalog;
use cosmic::iced::widget::Column;
use cosmic::theme::ProgressBar;
use cosmic::{widget, Element};

/// The spacing between the title and each section of a detail popup.
const SECTION_SPACING: u16 = 12;

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
