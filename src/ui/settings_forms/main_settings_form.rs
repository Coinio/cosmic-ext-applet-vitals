use crate::app::Message;
use crate::configuration::app_configuration::{AppConfiguration};
use crate::fl;
use cosmic::iced_widget::Container;
use cosmic::widget::{container, settings};
use cosmic::{widget, Theme};
use std::clone::Clone;

pub struct MainSettingsForm;

impl MainSettingsForm {
    pub fn draw(app_config: &'_ AppConfiguration) -> Container<'_, Message, Theme> {
        let mut column = widget::list_column()
            .padding(2)
            .spacing(0)
            .divider_padding(2)
            .add(widget::text(fl!("settings-title")).font(cosmic::iced::Font {
                weight: cosmic::iced::font::Weight::ExtraBold,
                ..Default::default()
            }));

        let settings_form_options = app_config.settings_form_options();

        for settings_form in settings_form_options.values() {

            let next_button = widget::button::custom(widget::icon::from_name("go-next-symbolic").size(16).icon())
                .on_press(Message::SettingsPopupOpened(settings_form.settings_window_id));

            column = column.add(settings::item(settings_form.title.clone(), next_button));
        }

        container(column)
    }
}
