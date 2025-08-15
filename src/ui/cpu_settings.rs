use cosmic::iced_widget::{container, Container};
use cosmic::{widget, Theme};
use cosmic::widget::settings;
use crate::app::{AppState, Message};

pub struct CpuSettingsUi;

impl CpuSettingsUi {
    pub fn content(app_state: &AppState) -> Container<Message, Theme> {
        let title = "CPU Settings";

        let content_list = container(widget::list_column().padding(5).spacing(0).add(
            settings::item(
                title,
                widget::toggler(false).on_toggle(Message::ToggleExampleRow),
            ),
        ));

        content_list
    }
}