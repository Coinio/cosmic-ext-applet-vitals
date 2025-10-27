use crate::app::Message;
use cosmic::iced::{Alignment, Color};
use cosmic::iced_core::Font;
use cosmic::Element;

pub struct IndicatorLabelProps<'a> {
    pub text: Option<String>,
    pub font_size: u16,
    pub font: &'a Font,
    pub colour: Color,
}

/// Creates a label for an indicator
pub fn indicator_label<'core>(
    core: &'core cosmic::Core,
    props: IndicatorLabelProps,
) -> Option<Element<'core, Message>> {
    if props.text.is_none() {
        return None;
    }

    Some(
        core.applet
            .text(props.text?)
            .size(props.font_size)
            .class(cosmic::theme::Text::from(props.colour))
            .align_x(Alignment::Start)
            .font(*props.font)
            .into(),
    )
}
