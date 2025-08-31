// SPDX-License-Identifier: GPL-3.0-only

use crate::configuration::app_configuration::{
    AppConfiguration, CPU_SETTINGS_WINDOW_ID, DISK_SETTINGS_WINDOW_ID, MAIN_SETTINGS_WINDOW_ID,
    MEMORY_SETTINGS_WINDOW_ID, NETWORK_SETTINGS_WINDOW_ID,
};
use crate::monitors::cpu_monitor::{CpuMonitor, CpuStats};
use crate::monitors::disk_monitor::{DiskMonitor, DiskStats};
use crate::monitors::memory_monitor::{MemoryMonitor, MemoryStats};
use crate::monitors::network_monitor::{NetworkMonitor, NetworkStats, NETWORK_STAT_RX_INDEX, NETWORK_STAT_TX_INDEX};
use crate::sensors::proc_disk_stats_reader::ProcDiskStatsReader;
use crate::sensors::proc_meminfo_reader::ProcMemInfoSensorReader;
use crate::sensors::proc_net_dev_reader::ProcNetDevReader;
use crate::sensors::proc_stat_reader::ProcStatSensorReader;
use crate::ui::indicators::IndicatorsUI;
use crate::ui::main_settings_form::MainSettingsForm;
use crate::ui::settings_form::{SettingsForm, SettingsFormEvent};
use cosmic::app::{Core, Task};
use cosmic::applet::cosmic_panel_config::PanelAnchor;
use cosmic::cosmic_config::{Config, CosmicConfigEntry};
use cosmic::iced::{window, Subscription};
use cosmic::iced::{Alignment, Limits};
use cosmic::iced_widget::{row, Column, Row};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::widget;
use cosmic::widget::{autosize, container, Id};
use cosmic::{cosmic_config, Application, Element};
use log::{error, info};
use once_cell::sync::Lazy;
use std::collections::HashMap;
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
    /// The current network usage stats
    network: [NetworkStats; 2],
    /// The current disk usage stats   
    disk: [DiskStats; 2],
    /// The popup id.
    popup: Option<window::Id>,
}

/// The messages processed by the application update
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle the main settings popup
    ToggleMainSettingsPopup(window::Id),
    /// A settings popup was closed
    SettingsPopupClosed(window::Id),
    /// Start monitoring the system resources
    StartMonitoring,
    /// The memory usage stats were updated
    MemoryUpdate(MemoryStats),
    /// The cpu usage stats were updated
    CpuUpdate(CpuStats),
    /// The network usage stats were updated
    NetworkUpdate([NetworkStats; 2]),
    /// The disk usage stats were updated
    DiskUpdate([DiskStats; 2]),
    /// The user has updated the settings form
    SettingsFormUpdate(SettingsFormEvent),
    /// The configuration file was changed externally
    ConfigFileChanged(AppConfiguration),
}

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

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let configuration = cosmic_config::Config::new(Self::APP_ID, AppConfiguration::VERSION)
            .map(|context| {
                AppConfiguration::get_entry(&context).unwrap_or_else(|(_errors, config)| {
                    error!("{:?}", _errors);
                    config
                })
            })
            .unwrap_or_default();

        let settings_forms = configuration.settings_form_options();

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

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::ToggleMainSettingsPopup(target_id) => {
                info!("Opening settings popup with id: {}", target_id);
                if let Some(current_id) = self.popup.take() {
                    if current_id == target_id {
                        return destroy_popup(current_id);
                    } else {
                        self.popup = Some(target_id);

                        let mut popup_settings = self.core.applet.get_popup_settings(
                            self.core.main_window_id().unwrap(),
                            target_id,
                            None,
                            None,
                            None,
                        );
                        popup_settings.positioner.size_limits = Limits::NONE
                            .max_width(372.0)
                            .min_width(300.0)
                            .min_height(200.0)
                            .max_height(1080.0);

                        return Task::batch(vec![destroy_popup(current_id), get_popup(popup_settings)]);
                    }
                } else {
                    self.popup = Some(target_id);

                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        target_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    return get_popup(popup_settings);
                }
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
                    let mut network_update_interval = tokio::time::interval(config.network.update_interval);
                    let mut disk_update_interval = tokio::time::interval(config.disk.update_interval);

                    let mut memory_monitor = MemoryMonitor::new(ProcMemInfoSensorReader, &config);
                    let mut cpuinfo_reader = CpuMonitor::new(ProcStatSensorReader, &config);
                    let mut network_monitor = NetworkMonitor::new(ProcNetDevReader, &config);
                    let mut disk_monitor = DiskMonitor::new(ProcDiskStatsReader, &config);

                    loop {
                        tokio::select! {
                            _ = memory_update_interval.tick() => {
                                yield Message::MemoryUpdate(memory_monitor.poll().unwrap_or_default());
                            },
                            _ = cpu_update_interval.tick() => {
                                yield Message::CpuUpdate(cpuinfo_reader.poll().unwrap_or_default())
                            },
                            _ = network_update_interval.tick() => {
                                yield Message::NetworkUpdate(network_monitor.poll().unwrap_or_default());
                            },
                            _ = disk_update_interval.tick() => {
                                yield Message::DiskUpdate(disk_monitor.poll().unwrap_or_default());
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
            Message::NetworkUpdate(network_usage) => {
                self.network = network_usage;
            }
            Message::DiskUpdate(disk_usage) => {
                self.disk = disk_usage;
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

    fn view(&self) -> Element<'_, Self::Message> {
        let is_horizontal = matches!(self.core.applet.anchor, PanelAnchor::Top | PanelAnchor::Bottom);

        let mut elements = Vec::new();

        elements.extend(IndicatorsUI::content(&self, &self.cpu, is_horizontal));
        elements.extend(IndicatorsUI::content(&self, &self.memory, is_horizontal));
        elements.extend(IndicatorsUI::content(
            &self,
            &self.network[NETWORK_STAT_RX_INDEX],
            is_horizontal,
        ));
        elements.extend(IndicatorsUI::content(
            &self,
            &self.network[NETWORK_STAT_TX_INDEX],
            is_horizontal,
        ));
        elements.extend(IndicatorsUI::content(&self, &self.disk[0], is_horizontal));
        elements.extend(IndicatorsUI::content(&self, &self.disk[1], is_horizontal));

        let wrapper: Element<Message> = if is_horizontal {
            Row::from_vec(elements)
                .align_y(Alignment::Center)
                .spacing(self.core.applet.suggested_padding(true))
                .into()
        } else {
            Column::from_vec(elements).align_x(Alignment::Center).into()
        };

        let padding = if is_horizontal {
            [0, self.core.applet.suggested_padding(true)]
        } else {
            [self.core.applet.suggested_padding(true), 1]
        };

        let button = widget::button::custom(wrapper)
            .class(cosmic::theme::Button::AppletIcon)
            .padding(padding)
            .on_press(Message::ToggleMainSettingsPopup(MAIN_SETTINGS_WINDOW_ID.clone()));

        autosize::autosize(container(button), AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn view_window(&'_ self, id: window::Id) -> Element<'_, Self::Message> {
        let content = if id == MAIN_SETTINGS_WINDOW_ID.clone() {
            MainSettingsForm::content(self.app_configuration())
        } else {
            match self.settings_forms.get(&id) {
                None => container(row!["No settings window configured."]),
                Some(form) => form.content(),
            }
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

    pub fn app_configuration(&self) -> &AppConfiguration {
        &self.configuration
    }

    fn update_configuration(&mut self) {
        info!("Updating configuration: {:?}", self.configuration);

        let memory_settings_form = self
            .settings_forms
            .get(&MEMORY_SETTINGS_WINDOW_ID.clone())
            .expect("No memory settings form configured.");

        let cpu_settings_form = self
            .settings_forms
            .get(&CPU_SETTINGS_WINDOW_ID.clone())
            .expect("No cpu settings form configured.");

        let network_settings_form = self
            .settings_forms
            .get(&NETWORK_SETTINGS_WINDOW_ID.clone())
            .expect("No network settings form configured.");

        let disk_settings_form = self
            .settings_forms
            .get(&DISK_SETTINGS_WINDOW_ID.clone())
            .expect("No disk settings form configured.");

        self.configuration = AppConfiguration {
            memory: self.configuration.memory.from(memory_settings_form),
            cpu: self.configuration.cpu.from(cpu_settings_form),
            network: self.configuration.network.from(network_settings_form),
            disk: self.configuration.disk.from(disk_settings_form),
            ..Default::default()
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
