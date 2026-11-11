use crate::app::Message;
use crate::core::app_colours::AppColour;
use cosmic::style::Svg;
use cosmic::widget::svg;
use cosmic::{widget, Element};
use std::rc::Rc;

pub struct SvgIconProps<'a> {
    pub icon: Option<&'a cosmic::widget::icon::Handle>,
    pub size: u16,
    /// Resolved against the theme the icon is drawn with. Without a colour the icon keeps its own fill.
    pub colour: Option<AppColour>,
}

pub fn svg_icon<'a>(props: SvgIconProps<'a>) -> Option<Element<'a, Message>> {
    match props.icon {
        None => None,
        Some(handle) => Some(Element::from(
            widget::icon::icon(handle.clone()).size(props.size).class(Svg::Custom {
                0: Rc::new(move |t| {
                    let mut style = svg::Catalog::style(t, &Svg::default(), svg::Status::Idle);

                    style.color = props.colour.map(|colour| colour.resolve(t.cosmic()));
                    style
                }),
            }),
        )),
    }
}
