// SPDX-License-Identifier: GPL-3.0-only

use crate::core::app_configuration::{
    AppConfiguration, CpuConfiguration, MemoryConfiguration, CPU_SETTINGS_WINDOW_ID, MEMORY_SETTINGS_WINDOW_ID,
};
use crate::monitors::cpu_monitor::{CpuMonitor, CpuStats};
use crate::monitors::memory_monitor::{MemoryMonitor, MemoryStats};
use crate::sensors::proc_meminfo_reader::ProcMemInfoSensorReader;
use crate::sensors::proc_stat_reader::ProcStatSensorReader;
use crate::ui::cpu_settings::{CpuSettingsForm};
use crate::ui::indicators::IndicatorsUI;
use crate::ui::memory_settings::{MemorySettingsForm};
use crate::ui::settings::{
    SettingsForm, SettingsFormEvent, LABEL_COLOUR_SETTING_KEY, LABEL_TEXT_SETTING_KEY, MAX_SAMPLES_SETTING_KEY,
    UPDATE_INTERVAL_SETTING_KEY,
};
use crate::ui::settings_input_sanitisers::FormInputValidation;
use cosmic::app::{Core, Task};
use cosmic::cosmic_config::{Config, CosmicConfigEntry};
use cosmic::iced::Limits;
use cosmic::iced::{window, Subscription};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::widget::{autosize, container, Id};
use cosmic::{cosmic_config, Application, Element};
use log::{error, info};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use cosmic::iced_widget::row;
use tokio_util::sync::CancellationToken;

static AUTOSIZE_MAIN_ID: Lazy<Id> = Lazy::new(|| Id::new("autosize-main"));

#[derive(Default)]
pub struct AppState {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The cancellation token to stop the status updates
    monitor_cancellation_token: Option<CancellationToken>,
    /// The application configuration
    configuration: AppConfiguration,
    /// The settings forms that are available for configuration of the monitors.
    settings_forms: HashMap<window::Id, SettingsForm>,
    /// The current memory usage stats
    memory: MemoryStats,
    /// The current cpu usage stats
    cpu: CpuStats,
    /// The popup id.
    popup: Option<window::Id>,
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    ToggleSettingsPopup(window::Id),
    SettingsPopupClosed(window::Id),
    StartMonitoring,
    ConfigFileChanged(AppConfiguration),
    MemoryUpdate(MemoryStats),
    CpuUpdate(CpuStats),
    SettingsFormUpdate(SettingsFormEvent),
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for AppState {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "dev.eidolon.cosmic-vitals-applet";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// This is the entry point of your application, it is where you initialize your application.
    ///
    /// Any work that needs to be done before the application starts should be done here.
    ///
    /// - `core` is used to passed on for you by libcosmic to use in the core of your own application.
    /// - `flags` is used to pass in any data that your application needs to use before it starts.
    /// - `Command` type is used to send messages to your application. `Command::none()` can be used to send no messages to your application.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let configuration = cosmic_config::Config::new(Self::APP_ID, AppConfiguration::VERSION)
            .map(|context| {
                AppConfiguration::get_entry(&context).unwrap_or_else(|(_errors, config)| {
                    error!("{:?}", _errors);
                    config
                })
            })
            .unwrap_or_default();

        let settings_forms = HashMap::from([
            (
                MEMORY_SETTINGS_WINDOW_ID.clone(),
                SettingsForm::new(
                    MEMORY_SETTINGS_WINDOW_ID.clone(),
                    MemorySettingsForm::from(&configuration.memory),
                ),
            ),
            (
                CPU_SETTINGS_WINDOW_ID.clone(),
                SettingsForm::new(
                    CPU_SETTINGS_WINDOW_ID.clone(),
                    CpuSettingsForm::from(&configuration.cpu),
                ),
            ),
        ]);

        let app = AppState {
            core,
            settings_forms,
            configuration,
            ..Default::default()
        };

        (app, cosmic::task::message(Message::StartMonitoring))
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::SettingsPopupClosed(id))
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.core()
            .watch_config::<AppConfiguration>(Self::APP_ID)
            .map(|update| Message::ConfigFileChanged(update.config))
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Commands may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::ToggleSettingsPopup(id) => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    self.popup.replace(id);

                    let form = self
                        .settings_forms
                        .get_mut(&id)
                        .expect(format!("No settings form configured with window Id: {}", id).as_str());

                    match id {
                        id if id == *MEMORY_SETTINGS_WINDOW_ID => {
                            form.values = MemorySettingsForm::from(&self.configuration.memory);
                        }
                        id if id == *CPU_SETTINGS_WINDOW_ID => {
                            form.values = CpuSettingsForm::from(&self.configuration.cpu);
                        }
                        _ => {
                            error!("Unknown window id: {}", id);
                            panic!("Unknown window id: {}", id)
                        }
                    };

                    let mut popup_settings =
                        self.core
                            .applet
                            .get_popup_settings(self.core.main_window_id().unwrap(), id, None, None, None);
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                };
            }
            Message::SettingsPopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                    self.save_configuration();
                    return cosmic::task::message(Message::StartMonitoring);
                }
            }
            Message::StartMonitoring => {
                if let Some(token) = &self.monitor_cancellation_token {
                    info!("Stopping previous monitors");
                    token.cancel();
                }

                info!("Starting monitoring");

                let cancellation_token = CancellationToken::new();
                self.monitor_cancellation_token = Some(cancellation_token.clone());

                let config = self.configuration.clone();

                return cosmic::Task::stream(async_stream::stream! {
                    let mut memory_update_interval = tokio::time::interval(config.memory.update_interval);
                    let mut cpu_update_interval = tokio::time::interval(config.cpu.update_interval);

                    let mut memory_monitor = MemoryMonitor::new(ProcMemInfoSensorReader, &config);
                    let mut cpuinfo_reader = CpuMonitor::new(ProcStatSensorReader, &config);

                    loop {
                        tokio::select! {
                            _ = memory_update_interval.tick() => {
                                yield Message::MemoryUpdate(memory_monitor.poll().unwrap_or_default());
                            },
                            _ = cpu_update_interval.tick() => {
                                yield Message::CpuUpdate(cpuinfo_reader.poll().unwrap_or_default())
                            },
                            _ = cancellation_token.cancelled() => {
                                break;
                            }
                        }

                    }
                })
                .map(cosmic::Action::App);
            }
            Message::MemoryUpdate(memory_usage) => {
                self.memory = memory_usage;
            }
            Message::CpuUpdate(cpu_usage) => {
                self.cpu = cpu_usage;
            }
            Message::ConfigFileChanged(configuration) => {
                self.configuration = configuration;
                return cosmic::task::message(Message::StartMonitoring);
            }
            Message::SettingsFormUpdate(settings_form_event) => {
                match settings_form_event {
                    SettingsFormEvent::StringFieldUpdated(value) => {
                        let form = self.settings_forms.get_mut(&value.settings_window_id).expect(
                            format!("No settings form configured with key: {}", value.settings_window_id).as_str(),
                        );

                        let label_text = form
                            .values
                            .get_mut(value.form_value_key)
                            .expect(format!("No form row with key: {}", value.form_value_key).as_str());

                        label_text.value = value.value;
                    }
                }

                self.update_configuration();
            }
        }
        Task::none()
    }

    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<'_, Self::Message> {
        // TODO: Handle horizontal / vertical layout
        //let horizontal = matches!(self.core.applet.anchor, PanelAnchor::Top |
        // PanelAnchor::Bottom);

        let container = container(
            cosmic::widget::row()
                .push(IndicatorsUI::content(&self, &self.cpu))
                .push(IndicatorsUI::content(&self, &self.memory)),
        );

        autosize::autosize(container, AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn view_window(&'_ self, id: window::Id) -> Element<'_, Self::Message> {

        let content = match self.settings_forms.get(&id) {
            None => { container(row!["No settings window configured."]) }
            Some(form) => { form.content(self) }
        };

        self.core.applet.popup_container(content).into()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

impl AppState {
    pub fn core(&self) -> &Core {
        &self.core
    }

    pub fn configuration(&self) -> &AppConfiguration {
        &self.configuration
    }

    fn update_configuration(&mut self) {
        let mut configuration = self.configuration.clone();

        info!("Updating configuration: {:?}", configuration);

        // TODO: Temporary unwraps mid refactor
        configuration.memory = match self.settings_forms.get(&MEMORY_SETTINGS_WINDOW_ID.clone()) {
            None => self.configuration.memory.clone(),
            Some(new_settings) => MemoryConfiguration {
                update_interval: FormInputValidation::sanitise_interval_input(
                    new_settings
                        .values
                        .get(UPDATE_INTERVAL_SETTING_KEY)
                        .unwrap()
                        .value
                        .clone(),
                    configuration.memory.update_interval,
                ),
                max_samples: FormInputValidation::sanitise_max_samples(
                    new_settings.values.get(MAX_SAMPLES_SETTING_KEY).unwrap().value.clone(),
                    configuration.memory.max_samples,
                ),
                label_text: FormInputValidation::sanitise_label_text(
                    new_settings.values.get(LABEL_TEXT_SETTING_KEY).unwrap().value.clone(),
                ),
                label_colour: FormInputValidation::sanitise_label_colour(
                    new_settings.values.get(LABEL_COLOUR_SETTING_KEY).unwrap().value.clone(),
                    configuration.memory.label_colour,
                ),
            },
        };

        // TODO: Temporary unwraps mid refactor
        configuration.cpu = match self.settings_forms.get(&CPU_SETTINGS_WINDOW_ID.clone()) {
            None => self.configuration.cpu.clone(),
            Some(new_settings) => CpuConfiguration {
                update_interval: FormInputValidation::sanitise_interval_input(
                    new_settings
                        .values
                        .get(UPDATE_INTERVAL_SETTING_KEY)
                        .unwrap()
                        .value
                        .clone(),
                    configuration.cpu.update_interval,
                ),
                max_samples: FormInputValidation::sanitise_max_samples(
                    new_settings.values.get(MAX_SAMPLES_SETTING_KEY).unwrap().value.clone(),
                    configuration.cpu.max_samples,
                ),
                label_text: FormInputValidation::sanitise_label_text(
                    new_settings.values.get(LABEL_TEXT_SETTING_KEY).unwrap().value.clone(),
                ),
                label_colour: FormInputValidation::sanitise_label_colour(
                    new_settings.values.get(LABEL_COLOUR_SETTING_KEY).unwrap().value.clone(),
                    configuration.cpu.label_colour,
                ),
            },
        };

        self.configuration = AppConfiguration {
            memory: configuration.memory,
            cpu: configuration.cpu,
        }
    }

    fn save_configuration(&self) {
        info!("Saving configuration");

        if let Ok(helper) = Config::new(Self::APP_ID, AppConfiguration::VERSION) {
            if let Err(err) = self.configuration.write_entry(&helper) {
                error!("Failed to save configuration: {}", err);
            }
        }
    }
}
