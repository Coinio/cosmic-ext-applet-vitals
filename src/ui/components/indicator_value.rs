use crate::app::Message;
use cosmic::iced::Alignment;
use cosmic::{widget, Element};

pub struct IndicatorValueProps {
    pub text: String,
    pub font_size: u16,
    pub width: f32,
    pub horizontal: bool,
}

pub fn indicator_value(core: &cosmic::Core, props: IndicatorValueProps) -> Option<Element<Message>> {
    let value_text = core.applet.text(props.text).size(props.font_size);

    if props.horizontal {
        Some(Element::from(
            widget::container(value_text)
                .width(props.width)
                .align_x(Alignment::End)
                .align_y(Alignment::Center),
        ))
    } else {
        Some(Element::from(
            widget::container(value_text)
                .width(props.width)
                .align_x(Alignment::Center),
        ))
    }
}
