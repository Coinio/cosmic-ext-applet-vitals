use crate::app::Message;
use crate::configuration::app_configuration::{
    AppConfiguration, CPU_SETTINGS_WINDOW_ID, MEMORY_SETTINGS_WINDOW_ID, NETWORK_SETTINGS_WINDOW_ID,
};
use crate::fl;
use cosmic::iced_widget::Container;
use cosmic::widget::{container, settings};
use cosmic::{widget, Theme};
use std::clone::Clone;

pub struct MainSettingsForm;

impl MainSettingsForm {
    pub fn content(app_config: &'_ AppConfiguration) -> Container<'_, Message, Theme> {
        let mut column = widget::list_column()
            .padding(2)
            .spacing(0)
            .divider_padding(2)
            // TODO: Move to localisation file
            .add(widget::text(fl!("settings-title")).font(cosmic::iced::Font {
                weight: cosmic::iced::font::Weight::ExtraBold,
                ..Default::default()
            }));

        let settings_form_options = app_config.settings_form_options();

        // TODO: Should be moved into the settings related files.
        let ordered_window_ids = vec![
            CPU_SETTINGS_WINDOW_ID.clone(),
            MEMORY_SETTINGS_WINDOW_ID.clone(),
            NETWORK_SETTINGS_WINDOW_ID.clone(),
        ];

        for window_id in ordered_window_ids {
            let settings_form = settings_form_options.get(&window_id).unwrap();

            let next_button = widget::button::custom(widget::icon::from_name("go-next-symbolic").size(16).icon())
                .on_press(Message::ToggleMainSettingsPopup(settings_form.settings_window_id));

            column = column.add(settings::item(settings_form.title.clone(), next_button));
        }

        container(column)
    }
}
