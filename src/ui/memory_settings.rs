use crate::app::{AppState, Message};
use crate::core::app_configuration::ConfigurationValue::{
    MemoryLabelText, MemoryMaxSamples, MemoryUpdateInterval,
};
use crate::fl;
use cosmic::iced_widget::{container, Container};
use cosmic::widget::settings;
use cosmic::{widget, Theme};
use crate::ui::settings_input_sanitisers::SettingsInputSanitisers;

pub struct MemorySettingsUi;

impl MemorySettingsUi {
    pub fn content(app_state: &AppState) -> Container<Message, Theme> {
        let configuration = app_state.configuration();
        let title = fl!("settings-memory-title");

        let current_interval = configuration.memory.update_interval.as_millis().to_string();

        let container = container(
            widget::list_column()
                .padding(5)
                .spacing(0)
                .add(widget::text(title))
                .add(settings::item(
                    fl!("settings-update-interval"),
                    widget::text_input(fl!("settings-empty"), current_interval).on_input(
                        |new_interval| {
                            let sanitised_interval = SettingsInputSanitisers::sanitise_interval_input(
                                new_interval,
                                configuration.memory.update_interval,
                            );
                            Message::ConfigValueUpdated(MemoryUpdateInterval(sanitised_interval))
                        },
                    ),
                ))
                .add(settings::item(
                    fl!("settings-max-samples"),
                    widget::text_input(
                        fl!("settings-empty"),
                        configuration.memory.max_samples.to_string(),
                    )
                    .on_input(|new_max_samples| {
                        let sanitised_samples = SettingsInputSanitisers::sanitise_max_samples(
                            new_max_samples,
                            configuration.memory.max_samples,
                        );

                        Message::ConfigValueUpdated(MemoryMaxSamples(sanitised_samples))
                    }),
                ))
                .add(settings::item(
                    fl!("settings-label-text"),
                    widget::text_input(
                        fl!("settings-empty"),
                        configuration.memory.label_text.as_str(),
                    )
                    .on_input(|new_label_text| {
                        let sanitised_label_text = SettingsInputSanitisers::sanitise_label_text(new_label_text);

                        Message::ConfigValueUpdated(MemoryLabelText(sanitised_label_text))
                    }),
                )),
        );

        container
    }
}
