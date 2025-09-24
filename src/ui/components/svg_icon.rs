use crate::app::Message;
use cosmic::iced_core::Color;
use cosmic::style::Svg;
use cosmic::widget::svg;
use cosmic::{widget, Element};
use std::rc::Rc;

pub struct SvgIconProps<'a> {
    pub icon: Option<&'a cosmic::widget::icon::Handle>,
    pub size: u16,
    pub colour: Option<Color>,
}

pub fn svg_icon(props: SvgIconProps) -> Option<Element<'_, Message>> {
    match props.icon {
        None => None,
        Some(handle) => Some(Element::from(
            widget::icon::icon(handle.clone()).size(props.size).class(Svg::Custom {
                0: Rc::new(move |t| {
                    let mut style = svg::Catalog::style(t, &Svg::default(), svg::Status::Idle);

                    style.color = props.colour;
                    style
                }),
            }),
        )),
    }
}
