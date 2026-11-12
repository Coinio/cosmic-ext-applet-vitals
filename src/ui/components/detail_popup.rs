use crate::app::Message;
use cosmic::iced::widget::Column;
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
