use crate::app::{AppState, Message};
use crate::core::app_configuration::ConfigurationValue::{
    CpuLabelText, CpuMaxSamples, CpuUpdateInterval,
};
use crate::fl;
use crate::ui::settings_input_sanitisers::SettingsInputSanitisers;
use cosmic::iced_widget::{container, Container};
use cosmic::widget::settings;
use cosmic::{widget, Theme};

pub struct CpuSettingsUi;

impl CpuSettingsUi {
    pub fn content(app_state: &AppState) -> Container<Message, Theme> {
        let configuration = app_state.configuration();
        let title = fl!("settings-cpu-title");

        let current_interval = configuration.cpu.update_interval.as_millis().to_string();

        let container = container(
            widget::list_column()
                .padding(5)
                .spacing(0)
                .add(widget::text(title))
                .add(settings::item(
                    fl!("settings-update-interval"),
                    widget::text_input(fl!("settings-empty"), current_interval).on_input(
                        |new_interval| {
                            let sanitised_interval =
                                SettingsInputSanitisers::sanitise_interval_input(
                                    new_interval,
                                    configuration.cpu.update_interval,
                                );
                            Message::ConfigValueUpdated(CpuUpdateInterval(sanitised_interval))
                        },
                    ),
                ))
                .add(settings::item(
                    fl!("settings-max-samples"),
                    widget::text_input(
                        fl!("settings-empty"),
                        configuration.cpu.max_samples.to_string(),
                    )
                    .on_input(|new_max_samples| {
                        let sanitised_samples = SettingsInputSanitisers::sanitise_max_samples(
                            new_max_samples,
                            configuration.cpu.max_samples,
                        );

                        Message::ConfigValueUpdated(CpuMaxSamples(sanitised_samples))
                    }),
                ))
                .add(settings::item(
                    fl!("settings-label-text"),
                    widget::text_input(
                        fl!("settings-empty"),
                        configuration.cpu.label_text.as_str(),
                    )
                    .on_input(|new_label_text| {
                        let sanitised_label_text =
                            SettingsInputSanitisers::sanitise_label_text(new_label_text);

                        Message::ConfigValueUpdated(CpuLabelText(sanitised_label_text))
                    }),
                )),
        );

        container
    }
}
