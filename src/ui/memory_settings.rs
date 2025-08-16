use crate::app::{AppState, Message};
use crate::core::app_configuration::{CpuConfiguration, MemoryConfiguration, SettingsForm};
use crate::fl;
use cosmic::iced_widget::{container, Container};
use cosmic::widget::settings;
use cosmic::{widget, Theme};

#[derive(Debug, Clone, Default)]
pub struct MemorySettingsForm {
    pub label_text: String,
    pub label_colour: String,
    pub update_interval: String,
    pub max_samples: String,
}

impl MemorySettingsForm {
    pub fn from(configuration: &MemoryConfiguration) -> Self {
        Self {
            label_text: configuration.label_text.clone(),
            label_colour: configuration.label_colour.display_rgba().to_string(),
            update_interval: configuration.update_interval.as_millis().to_string(),
            max_samples: configuration.max_samples.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CpuSettingsForm {
    pub label_text: String,
    pub label_colour: String,
    pub update_interval: String,
    pub max_samples: String,
}

impl CpuSettingsForm {
    pub fn from(configuration: &CpuConfiguration) -> Self {
        Self {
            label_text: configuration.label_text.clone(),
            label_colour: configuration.label_colour.display_rgba().to_string(),
            update_interval: configuration.update_interval.as_millis().to_string(),
            max_samples: configuration.max_samples.to_string(),
        }
    }
}

pub struct MemorySettingsUi;

impl MemorySettingsUi {
    pub fn content(app_state: &AppState) -> Container<Message, Theme> {

;        let mut memory_settings_form = app_state.memory_settings_form()
            .expect("Memory settings form must be set before calling content");

        let title = fl!("settings-memory-title");

        let container = container(
            widget::list_column()
                .padding(5)
                .spacing(0)
                .add(widget::text(title))
                .add(settings::item(
                    fl!("settings-update-interval"),
                    widget::text_input(fl!("settings-empty"), &memory_settings_form.update_interval)
                        .on_input(
                        | new_interval| {
                            let mut form = memory_settings_form.clone();
                            form.update_interval = new_interval;
                            Message::SettingFormUpdated(SettingsForm::MemorySettings(form))
                        },
                    ),
                ))
                .add(settings::item(
                    fl!("settings-max-samples"),
                    widget::text_input(fl!("settings-empty"), &memory_settings_form.max_samples)
                        .on_input(|new_max_samples| {
                            let mut form = memory_settings_form.clone();
                            form.max_samples = new_max_samples;
                            Message::SettingFormUpdated(SettingsForm::MemorySettings(form))
                        }),
                ))
                .add(settings::item(
                    fl!("settings-label-text"),
                    widget::text_input(fl!("settings-empty"), &memory_settings_form.label_text).on_input(
                        |new_label_text| {
                            let mut form = memory_settings_form.clone();
                            form.label_text = new_label_text;
                            Message::SettingFormUpdated(SettingsForm::MemorySettings(form))
                        },
                    ),
                ))
                .add(settings::item(
                    fl!("settings-label-colour"),
                    widget::text_input(fl!("settings-empty"), &memory_settings_form.label_colour).on_input(
                        |new_label_color| {
                            let mut form = memory_settings_form.clone();
                            form.label_colour = new_label_color;
                            Message::SettingFormUpdated(SettingsForm::MemorySettings(form))
                        },
                    ),
                )),
        );

        container
    }
}
