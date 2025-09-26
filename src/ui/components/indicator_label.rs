use crate::app::Message;
use cosmic::iced::{Alignment, Color};
use cosmic::Element;

pub struct IndicatorLabelProps {
    pub text: Option<String>,
    pub font_size: u16,
    pub colour: Color,
}

/// Creates a label for an indicator
pub fn indicator_label(core: &'_ cosmic::Core, props: IndicatorLabelProps) -> Option<Element<'_, Message>> {
    if props.text.is_none() {
        return None;
    }

    Some(
        core.applet
            .text(props.text?)
            .size(props.font_size)
            .class(cosmic::theme::Text::from(props.colour))
            .align_x(Alignment::Start)
            .font(cosmic::iced::Font {
                weight: cosmic::iced::font::Weight::Bold,
                ..Default::default()
            })
            .into(),
    )
}
