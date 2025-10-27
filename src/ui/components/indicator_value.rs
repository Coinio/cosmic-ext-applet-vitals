use crate::app::Message;
use cosmic::iced::Alignment;
use cosmic::{widget, Element};
use cosmic::font::Font;

pub struct IndicatorValueProps<'a> {
    pub text: String,
    pub font_size: u16,
    pub font: &'a Font,
    pub width: Option<f32>,
    pub horizontal: bool,
}

pub fn indicator_value<'core>(
    core: &'core cosmic::Core,
    props: IndicatorValueProps,
) -> Option<Element<'core, Message>> {
    let value_text = core.applet.text(props.text)
        .size(props.font_size)
        .font(*props.font);
    if props.horizontal {
        let mut container = widget::container(value_text)
            .align_x(Alignment::End)
            .align_y(Alignment::Center);
        if let Some(w) = props.width { container = container.width(w); }
        Some(Element::from(container))
    } else {
        let mut container = widget::container(value_text)
            .align_x(Alignment::Center);
        if let Some(w) = props.width { container = container.width(w); }
        Some(Element::from(container))
    }
}
