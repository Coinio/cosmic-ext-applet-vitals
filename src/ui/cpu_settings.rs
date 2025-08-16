use crate::app::{AppState, Message};
use crate::fl;
use cosmic::iced_widget::{container, Container};
use cosmic::{widget, Theme};
use cosmic::widget::settings;
use crate::core::app_configuration::SettingsForm;

pub struct CpuSettingsUi;

impl CpuSettingsUi {
    pub fn content(app_state: &AppState) -> Container<Message, Theme> {
        let configuration = app_state.configuration();
        let title = fl!("settings-cpu-title");

        let mut cpu_settings_form = app_state.cpu_settings_form()
            .expect("CPU settings form must be set before calling content");

        let container = container(
            widget::list_column()
                .padding(5)
                .spacing(0)
                .add(widget::text(title))
                .add(settings::item(
                    fl!("settings-update-interval"),
                    widget::text_input(fl!("settings-empty"), &cpu_settings_form.update_interval)
                        .on_input(
                            | new_interval| {
                                let mut form = cpu_settings_form.clone();
                                form.update_interval = new_interval;
                                Message::SettingFormUpdated(SettingsForm::CpuSettings(form))
                            },
                        ),
                ))
                .add(settings::item(
                    fl!("settings-max-samples"),
                    widget::text_input(fl!("settings-empty"), &cpu_settings_form.max_samples)
                        .on_input(|new_max_samples| {
                            let mut form = cpu_settings_form.clone();
                            form.max_samples = new_max_samples;
                            Message::SettingFormUpdated(SettingsForm::CpuSettings(form))
                        }),
                ))
                .add(settings::item(
                    fl!("settings-label-text"),
                    widget::text_input(fl!("settings-empty"), &cpu_settings_form.label_text).on_input(
                        |new_label_text| {
                            let mut form = cpu_settings_form.clone();
                            form.label_text = new_label_text;
                            Message::SettingFormUpdated(SettingsForm::CpuSettings(form))
                        },
                    ),
                ))
                .add(settings::item(
                    fl!("settings-label-colour"),
                    widget::text_input(fl!("settings-empty"), &cpu_settings_form.label_colour).on_input(
                        |new_label_color| {
                            let mut form = cpu_settings_form.clone();
                            form.label_colour = new_label_color;
                            Message::SettingFormUpdated(SettingsForm::CpuSettings(form))
                        },
                    ),
                )),
        );

        container
    }
}
