use crate::app::Message;
use crate::core::app_icons::AppIcon;
use cosmic::{widget, Element};

pub struct NoIndicatorProps<'a> {
    pub icon: Option<&'a AppIcon>,
    pub size: u16,
}

pub fn no_indicators_content<'a>(props: NoIndicatorProps) -> Element<'a, Message> {
    match props.icon {
        None => Element::new(widget::Row::new()),
        Some(icon) => Element::from(widget::icon::icon(icon.handle.clone()).size(props.size)),
    }
}
