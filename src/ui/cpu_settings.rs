use crate::app::{AppState, Message};
use crate::fl;
use cosmic::iced_widget::{container, Container};
use cosmic::{widget, Theme};
use cosmic::widget::settings;
use crate::core::app_configuration::{SettingsFormEvent, SettingsFormValue};

pub const CPU_SETTINGS_FORM_KEY: &str = "cpu_settings_form";

pub struct CpuSettingsUi;

impl CpuSettingsUi {
    pub fn content(app_state: &'_ AppState) -> Container<'_, Message, Theme> {
        let title = fl!("settings-cpu-title");

        let cpu_settings_form = app_state.cpu_settings_form()
            .expect("CPU settings form must be set before calling content");

        let container = container(
            widget::list_column()
                .padding(5)
                .spacing(0)
                .add(widget::text(title).font(cosmic::iced::Font {
                    weight: cosmic::iced::font::Weight::ExtraBold,
                    ..Default::default()
                }))
                .add(settings::item(
                    fl!("settings-update-interval"),
                    widget::text_input(
                        fl!("settings-empty"),
                        &cpu_settings_form.update_interval,
                    )
                        .on_input(|new_interval| {
                            Message::SettingsFormUpdate(SettingsFormEvent::UpdateIntervalChanged(
                                SettingsFormValue {
                                    monitor_id: CPU_SETTINGS_FORM_KEY,
                                    value: new_interval,
                                },
                            ))
                        }),
                ))
                .add(settings::item(
                    fl!("settings-max-samples"),
                    widget::text_input(fl!("settings-empty"), &cpu_settings_form.max_samples)
                        .on_input(|new_max_samples| {
                            Message::SettingsFormUpdate(SettingsFormEvent::MaxSamplesChanged(
                                SettingsFormValue {
                                    monitor_id: CPU_SETTINGS_FORM_KEY,
                                    value: new_max_samples,
                                },
                            ))
                        }),
                ))
                .add(settings::item(
                    fl!("settings-label-text"),
                    widget::text_input(fl!("settings-empty"), &cpu_settings_form.label_text)
                        .on_input(|new_label_text| {
                            Message::SettingsFormUpdate(SettingsFormEvent::LabelTextChanged(
                                SettingsFormValue {
                                    monitor_id: CPU_SETTINGS_FORM_KEY,
                                    value: new_label_text,
                                },
                            ))
                        }),
                ))
                .add(settings::item(
                    fl!("settings-label-colour"),
                    widget::text_input(fl!("settings-empty"), &cpu_settings_form.label_colour)
                        .on_input(|new_label_color| {
                            Message::SettingsFormUpdate(SettingsFormEvent::LabelColourChanged(
                                SettingsFormValue {
                                    monitor_id: CPU_SETTINGS_FORM_KEY,
                                    value: new_label_color,
                                },
                            ))
                        }),
                )),
        );

        container
    }
}
