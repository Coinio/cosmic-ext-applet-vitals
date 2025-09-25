// SPDX-License-Identifier: GPL-3.0-only

use crate::configuration::app_configuration::{
    AppConfiguration, CPU_SETTINGS_WINDOW_ID, DISK_SETTINGS_WINDOW_ID, MAIN_SETTINGS_WINDOW_ID,
    MEMORY_SETTINGS_WINDOW_ID, NETWORK_SETTINGS_WINDOW_ID,
};
use crate::monitors::cpu_monitor::{CpuMonitor, CpuStats};
use crate::monitors::disk_monitor::{DiskMonitor, DiskStats};
use crate::monitors::memory_monitor::{MemoryMonitor, MemoryStats};
use crate::monitors::network_monitor::{NetworkMonitor, NetworkStats};
use crate::sensors::proc_disk_stats_reader::ProcDiskStatsReader;
use crate::sensors::proc_meminfo_reader::ProcMemInfoSensorReader;
use crate::sensors::proc_net_dev_reader::ProcNetDevReader;
use crate::sensors::proc_stat_reader::ProcStatSensorReader;
use crate::ui::app_colours::AppColours;
use crate::ui::app_icons::{AppIcons, APP_LOGO_ICON};
use crate::ui::app_text_measurements::AppTextMeasurements;
use crate::ui::components::no_indicator::{no_indicators_content, NoIndicatorProps};
use crate::ui::main_settings_form::MainSettingsForm;
use crate::ui::settings_form::{SettingsForm, SettingsFormEvent};
use cosmic::app::{Core, Task};
use cosmic::applet::cosmic_panel_config::{PanelAnchor, PanelSize};
use cosmic::cosmic_config::{Config, CosmicConfigEntry};
use cosmic::iced::{window, Subscription};
use cosmic::iced::{Alignment, Limits};
use cosmic::iced_widget::{row, Column, Row};
use cosmic::iced_winit::commands::popup::get_popup;
use cosmic::widget;
use cosmic::widget::{autosize, container, divider, Id};
use cosmic::{cosmic_config, Application, Element};
use log::{error, info};
use once_cell::sync::Lazy;
use std::collections::BTreeMap;
use tokio_util::sync::CancellationToken;

pub const GLOBAL_APP_ID: &'static str = "dev.eidolon.cosmic-vitals-applet";

static AUTOSIZE_MAIN_ID: Lazy<Id> = Lazy::new(|| Id::new("autosize-main"));

const DEFAULT_INDICATOR_FONT_SIZE: u16 = 14;
const DEFAULT_INDICATOR_ICON_SIZE: u16 = 12;
const DEFAULT_INDICATOR_SPACING: u16 = 8;

#[derive(Default)]
pub struct AppState {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The cancellation token to stop the status updates
    monitor_cancellation_token: Option<CancellationToken>,
    /// The colours available to the application
    app_colours: AppColours,
    /// The icons available to the application
    app_icons: AppIcons,
    /// The text measurements for indicator labels to prevent jittering as labels change size
    app_text_measurements: AppTextMeasurements,
    /// The application configuration
    configuration: AppConfiguration,
    /// The settings forms that are available for configuration of the monitors.
    settings_forms: BTreeMap<window::Id, SettingsForm>,
    /// The current memory usage stats
    memory: MemoryStats,
    /// The current cpu usage stats
    cpu: CpuStats,
    /// The current network usage stats
    network: NetworkStats,
    /// The current disk usage stats   
    disk: DiskStats,
    /// The popup id.
    popup: Option<window::Id>,
}

/// The messages processed by the application update
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle the main settings popup
    SettingsPopupOpened(window::Id),
    /// A settings popup was closed
    SettingsPopupClosed(window::Id),
    /// Start monitoring the system resources
    StartMonitoring,
    /// The memory usage stats were updated
    MemoryUpdate(MemoryStats),
    /// The cpu usage stats were updated
    CpuUpdate(CpuStats),
    /// The network usage stats were updated
    NetworkUpdate(NetworkStats),
    /// The disk usage stats were updated
    DiskUpdate(DiskStats),
    /// The user has updated the settings form
    SettingsFormUpdate(SettingsFormEvent),
    /// The configuration file was changed externally
    ConfigFileChanged(AppConfiguration),
}

impl Application for AppState {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = GLOBAL_APP_ID;

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
        let app_colours = AppColours::from(&core.system_theme().cosmic().palette);
        let app_icons = AppIcons::new();
        let app_text_measurements = AppTextMeasurements::new();

        let app = AppState {
            core,
            settings_forms,
            app_colours,
            app_icons,
            app_text_measurements,
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
            Message::SettingsPopupOpened(target_id) => {

                info!("Opening settings popup with id: {}", target_id);

                match self.popup {
                    None => {
                        self.popup = Some(MAIN_SETTINGS_WINDOW_ID.clone());
                        // Ensure the configuration is up to date when we open the settings form.
                        self.refresh_configuration_from_disk();
                    },
                    Some(_) => self.popup = Some(target_id),
                };

                if target_id != MAIN_SETTINGS_WINDOW_ID.clone() {
                    return Task::none();
                }

                let mut popup_settings = self.core.applet.get_popup_settings(
                    self.core.main_window_id().unwrap(),
                    MAIN_SETTINGS_WINDOW_ID.clone(),
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
            Message::SettingsPopupClosed(id) => {
                if id == MAIN_SETTINGS_WINDOW_ID.clone() {
                    info!("Closing main settings window");
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

                    let mut memory_monitor = (!config.memory.hide_indicator)
                        .then(|| MemoryMonitor::new(ProcMemInfoSensorReader, &config));
                    let mut cpuinfo_reader = (!config.cpu.hide_indicator)
                        .then(|| CpuMonitor::new(ProcStatSensorReader, &config));
                    let mut network_monitor = (!config.network.hide_indicator)
                        .then(|| NetworkMonitor::new(ProcNetDevReader, &config));
                    let mut disk_monitor = (!config.disk.hide_indicator)
                        .then(|| DiskMonitor::new(ProcDiskStatsReader, &config));

                    loop {
                        tokio::select! {
                            _ = memory_update_interval.tick(), if !config.memory.hide_indicator => {
                                if let Some(memory_monitor) = memory_monitor.as_mut() {
                                    yield Message::MemoryUpdate(memory_monitor.poll().unwrap_or_default());
                                }
                            },
                            _ = cpu_update_interval.tick(), if !config.cpu.hide_indicator => {
                                if let Some(cpu_monitor) = cpuinfo_reader.as_mut() {
                                    yield Message::CpuUpdate(cpu_monitor.poll().unwrap_or_default());
                                }
                            },
                            _ = network_update_interval.tick(), if !config.network.hide_indicator => {
                                if let Some(network_monitor) = network_monitor.as_mut() {
                                    yield Message::NetworkUpdate(network_monitor.poll().unwrap_or_default());
                                }
                            },
                            _ = disk_update_interval.tick(), if !config.disk.hide_indicator => {
                                if let Some(disk_monitor) = disk_monitor.as_mut() {
                                    yield Message::DiskUpdate(disk_monitor.poll().unwrap_or_default());
                                }
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
                    SettingsFormEvent::StringFieldUpdated(value) | SettingsFormEvent::CheckBoxUpdated(value) => {
                        let form = self.settings_forms.get_mut(&value.settings_window_id).expect(
                            format!("No settings form configured with key: {}", value.settings_window_id).as_str(),
                        );

                        let settings_form_item = form
                            .values
                            .get_mut(value.form_value_key)
                            .expect(format!("No form row with key: {}", value.form_value_key).as_str());

                        settings_form_item.value = value.value;
                    }
                }

                self.update_configuration();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let is_horizontal = true;
        //matches!(self.core.applet.anchor, PanelAnchor::Top |            PanelAnchor::Bottom);

        let mut elements: Vec<Element<Message>> = Vec::new();

        elements.push(divider::vertical::default().into());

        if let Some(element) = self.cpu.draw(&self, is_horizontal) {
            elements.push(element);
            elements.push(divider::vertical::default().into())
        }
        if let Some(element) = self.memory.draw(&self, is_horizontal) {
            elements.push(element);
            elements.push(divider::vertical::default().into())
        }
        if let Some(element) = self.network.draw(&self, is_horizontal) {
            elements.push(element);
            elements.push(divider::vertical::default().into())
        }
        if let Some(element) = self.disk.draw(&self, is_horizontal) {
            elements.push(element);
            elements.push(divider::vertical::default().into());
        }
        if elements.len() <= 1 {
            elements.push(no_indicators_content(NoIndicatorProps {
                icon: self.app_icons.get(APP_LOGO_ICON),
                size: self.icon_size(),
            }));
            elements.push(divider::vertical::default().into());
        }

        let wrapper: Element<Message> = if is_horizontal {
            Row::from_vec(elements)
                .align_y(Alignment::Center)
                .spacing(DEFAULT_INDICATOR_SPACING)
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
            .on_press(Message::SettingsPopupOpened(MAIN_SETTINGS_WINDOW_ID.clone()));

        autosize::autosize(container(button), AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn view_window(&'_ self, _id: window::Id) -> Element<'_, Self::Message> {
        let content_id = self.popup.unwrap_or_else(|| MAIN_SETTINGS_WINDOW_ID.clone());

        let content = if content_id == MAIN_SETTINGS_WINDOW_ID.clone() {
            MainSettingsForm::content(self.configuration())
        } else {
            match self.settings_forms.get(&content_id) {
                None => container(row!["No settings window configured."]),
                Some(form) => form.content(&self),
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

    pub fn app_colours(&self) -> &AppColours {
        &self.app_colours
    }

    pub fn app_icons(&self) -> &AppIcons {
        &self.app_icons
    }

    pub fn configuration(&self) -> &AppConfiguration {
        &self.configuration
    }
    pub fn app_text_measurements(&self) -> &AppTextMeasurements {
        &self.app_text_measurements
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
            general: self.configuration.general.clone(),
            memory: self.configuration.memory.update(memory_settings_form),
            cpu: self.configuration.cpu.update(cpu_settings_form),
            network: self.configuration.network.update(network_settings_form),
            disk: self.configuration.disk.update(disk_settings_form),
            ..Default::default()
        }
    }

    fn refresh_configuration_from_disk(&mut self) {
        if let Ok(helper) = Config::new(Self::APP_ID, AppConfiguration::VERSION) {
            let loaded = AppConfiguration::get_entry(&helper)
                .unwrap_or_else(|(errs, cfg)| {
                    error!("Errors while loading configuration: {:?}", errs);
                    cfg
                });

            self.configuration = loaded;
            self.settings_forms = self.configuration.settings_form_options();
        } else {
            error!("Failed to create config context for reload");
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

    pub fn font_size(&self, horizontal: bool) -> u16 {
        let configuration = self.configuration();

        match self.core.applet.size {
            cosmic::applet::Size::PanelSize(PanelSize::XS) if horizontal => {
                configuration.general.horizontal_font_size_xs
            }
            cosmic::applet::Size::PanelSize(PanelSize::XS) => configuration.general.vertical_font_size_xs,
            cosmic::applet::Size::PanelSize(PanelSize::S) if horizontal => {
                configuration.general.horizontal_font_size_sm
            }
            cosmic::applet::Size::PanelSize(PanelSize::S) => configuration.general.vertical_font_size_sm,
            cosmic::applet::Size::PanelSize(PanelSize::M) if horizontal => {
                configuration.general.horizontal_font_size_md
            }
            cosmic::applet::Size::PanelSize(PanelSize::M) => configuration.general.vertical_font_size_md,
            cosmic::applet::Size::PanelSize(PanelSize::L) if horizontal => {
                configuration.general.horizontal_font_size_lg
            }
            cosmic::applet::Size::PanelSize(PanelSize::L) => configuration.general.vertical_font_size_lg,
            cosmic::applet::Size::PanelSize(PanelSize::XL) if horizontal => {
                configuration.general.horizontal_font_size_xl
            }
            cosmic::applet::Size::PanelSize(PanelSize::XL) => configuration.general.vertical_font_size_xl,
            _ => DEFAULT_INDICATOR_FONT_SIZE,
        }
    }

    pub fn icon_size(&self) -> u16 {
        match self.core.applet.size {
            cosmic::applet::Size::PanelSize(PanelSize::XS) => DEFAULT_INDICATOR_ICON_SIZE,
            cosmic::applet::Size::PanelSize(PanelSize::S) => DEFAULT_INDICATOR_ICON_SIZE + 2,
            cosmic::applet::Size::PanelSize(PanelSize::M) => DEFAULT_INDICATOR_ICON_SIZE + 4,
            cosmic::applet::Size::PanelSize(PanelSize::L) => DEFAULT_INDICATOR_ICON_SIZE + 6,
            cosmic::applet::Size::PanelSize(PanelSize::XL) => DEFAULT_INDICATOR_ICON_SIZE + 8,
            _ => DEFAULT_INDICATOR_ICON_SIZE,
        }
    }
}
