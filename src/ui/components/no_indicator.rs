use crate::app::Message;
use cosmic::{widget, Element};

pub struct NoIndicatorProps<'a> {
    pub icon: Option<&'a widget::icon::Handle>,
    pub size: u16,
}

pub fn no_indicators_content<'a>(props: NoIndicatorProps) -> Element<'a, Message> {
    match props.icon {
        None => Element::new(widget::Row::new()),
        Some(handle) => Element::from(widget::icon::icon(handle.clone()).size(props.size)),
    }
}
