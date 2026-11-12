// SPDX-License-Identifier: GPL-3.0-only

use crate::configuration::app_configuration::AppConfiguration;
use crate::configuration::general::FontSizeSlot;
use crate::core::app_fonts;
use crate::core::app_icons::{AppIcons, APP_LOGO_ICON};
use crate::core::app_text_measurements::AppTextMeasurements;
use crate::monitors::cpu_monitor::{CpuMonitor, CpuStats};
use crate::monitors::disk_monitor::{DiskMonitor, DiskStats};
use crate::monitors::memory_monitor::{MemoryMonitor, MemoryStats};
use crate::monitors::network_monitor::{NetworkMonitor, NetworkStats};
use crate::monitors::storage_monitor::{StorageMonitor, StorageStats};
use crate::sensors::proc_disk_stats_reader::ProcDiskStatsReader;
use crate::sensors::proc_meminfo_reader::ProcMemInfoSensorReader;
use crate::sensors::proc_net_dev_reader::ProcNetDevReader;
use crate::sensors::proc_stat_reader::ProcStatSensorReader;
use crate::sensors::storage_reader::StorageReader;
use crate::settings_app::SETTINGS_FLAG;
use crate::ui::components::indicator::indicator;
use crate::ui::components::no_indicator::{no_indicators_content, NoIndicatorProps};
use cosmic::app::{Core, Task};
use cosmic::applet::cosmic_panel_config::{PanelAnchor, PanelSize};
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::font::Font;
use cosmic::iced::Alignment;
use cosmic::iced::Subscription;
use cosmic::iced::widget::{Column, Row};
use cosmic::widget;
use cosmic::widget::{autosize, container, divider, Id};
use cosmic::{cosmic_config, Application, Element};
use log::{error, info, warn};
use once_cell::sync::Lazy;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

pub const GLOBAL_APP_ID: &'static str = "dev.eidolon.cosmic-ext-applet-vitals";

static AUTOSIZE_MAIN_ID: Lazy<Id> = Lazy::new(|| Id::new("autosize-main"));

const DEFAULT_INDICATOR_FONT_SIZE: u16 = 11;
const DEFAULT_INDICATOR_ICON_SIZE: u16 = 11;
const DEFAULT_INDICATOR_SPACING: u16 = 8;

#[derive(Default)]
pub struct AppState {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The cancellation token to stop the status updates
    monitor_cancellation_token: Option<CancellationToken>,
    interface_font: Font,
    indicator_font: Font,
    /// The icons available to the application
    app_icons: AppIcons,
    /// The text measurements for indicator labels to prevent jittering as labels change size
    app_text_measurements: AppTextMeasurements,
    /// The application configuration
    configuration: AppConfiguration,
    /// The current memory usage stats
    memory: MemoryStats,
    /// The current cpu usage stats
    cpu: CpuStats,
    /// The current network usage stats
    network: NetworkStats,
    /// The current disk usage stats   
    disk: DiskStats,
    /// The current available and total storage
    storage: StorageStats,
    /// The settings window process, if one has been launched. See [`crate::settings_app`] for why it is separate.
    settings_process: Option<Child>,
}

/// Monitors that fall due within this window of each other are polled together, so they share one redraw.
const MONITOR_COALESCE_WINDOW: Duration = Duration::from_millis(50);

/// The results of a single monitor tick. Only the monitors that were due are populated.
#[derive(Debug, Clone, Default)]
pub struct StatsUpdate {
    memory: Option<MemoryStats>,
    cpu: Option<CpuStats>,
    network: Option<NetworkStats>,
    disk: Option<DiskStats>,
    storage: Option<StorageStats>,
}

/// The messages processed by the application update
#[derive(Debug, Clone)]
pub enum Message {
    /// Open the settings window, unless it is already open
    OpenSettingsWindow,
    /// Start monitoring the system resources
    StartMonitoring,
    /// One or more monitors were polled. Updates are batched so each tick causes a single redraw.
    StatsUpdate(StatsUpdate),
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

        let app_icons = AppIcons::new();
        let app_text_measurements = AppTextMeasurements::new();

        let interface_font = cosmic::font::default();
        let indicator_font = app_fonts::resolve_font(&configuration.general.font, &interface_font);

        let app = AppState {
            core,
            interface_font,
            indicator_font,
            app_icons,
            app_text_measurements,
            configuration,
            ..Default::default()
        };

        (app, cosmic::task::message(Message::StartMonitoring))
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subscriptions = Vec::new();

        subscriptions.push(
            self.core()
                .watch_config::<AppConfiguration>(Self::APP_ID)
                .map(|update| Message::ConfigFileChanged(update.config)),
        );

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::OpenSettingsWindow => self.open_settings_window(),
            Message::StartMonitoring => {
                self.stop_monitoring();

                info!("Starting monitoring");

                let cancellation_token = CancellationToken::new();
                self.monitor_cancellation_token = Some(cancellation_token.clone());

                let config = self.configuration.clone();

                return cosmic::Task::stream(async_stream::stream! {
                    let mut memory_monitor = (!config.memory.hide_indicator)
                        .then(|| MemoryMonitor::new(ProcMemInfoSensorReader, &config));
                    let mut cpu_monitor = (!config.cpu.hide_indicator)
                        .then(|| CpuMonitor::new(ProcStatSensorReader, &config));
                    let mut network_monitor = (!config.network.hide_indicator)
                        .then(|| NetworkMonitor::new(ProcNetDevReader, &config));
                    let mut disk_monitor = (!config.disk.hide_indicator)
                        .then(|| DiskMonitor::new(ProcDiskStatsReader, &config));
                    let mut storage_monitor = (!config.storage.hide_indicator)
                        .then(|| StorageMonitor::new(StorageReader::new(config.storage.mounts.clone())));

                    // A single scheduler drives every monitor. Monitors that fall due at (nearly) the same
                    // time are polled together and sent as one message, because every message rebuilds and
                    // redraws the applet. Separate timers per monitor would wake the UI once per monitor.
                    let start = Instant::now();
                    let mut schedules = [
                        memory_monitor.is_some().then_some((start, config.memory.update_interval)),
                        cpu_monitor.is_some().then_some((start, config.cpu.update_interval)),
                        network_monitor.is_some().then_some((start, config.network.update_interval)),
                        disk_monitor.is_some().then_some((start, config.disk.update_interval)),
                        storage_monitor.is_some().then_some((start, config.storage.update_interval)),
                    ];

                    loop {
                        let Some(next_due) = schedules.iter().flatten().map(|(due, _)| *due).min() else {
                            // Every indicator is hidden, nothing to poll.
                            cancellation_token.cancelled().await;
                            break;
                        };

                        tokio::select! {
                            _ = tokio::time::sleep_until(next_due) => {},
                            _ = cancellation_token.cancelled() => break,
                        }

                        let now = Instant::now();
                        let mut due = [false; 5];

                        for (index, schedule) in schedules.iter_mut().enumerate() {
                            if let Some((next, interval)) = schedule {
                                if *next <= now + MONITOR_COALESCE_WINDOW {
                                    due[index] = true;
                                    // Scheduling from `now` keeps coalesced monitors in step and never
                                    // leaves missed ticks to catch up on.
                                    *next = now + *interval;
                                }
                            }
                        }

                        let update = StatsUpdate {
                            memory: memory_monitor.as_mut().filter(|_| due[0]).map(|m| m.poll().unwrap_or_default()),
                            cpu: cpu_monitor.as_mut().filter(|_| due[1]).map(|m| m.poll().unwrap_or_default()),
                            network: network_monitor.as_mut().filter(|_| due[2]).map(|m| m.poll().unwrap_or_default()),
                            disk: disk_monitor.as_mut().filter(|_| due[3]).map(|m| m.poll().unwrap_or_default()),
                            storage: storage_monitor.as_mut().filter(|_| due[4]).map(|m| m.poll().unwrap_or_default()),
                        };

                        yield Message::StatsUpdate(update);
                    }
                })
                .map(cosmic::Action::App);
            }
            Message::StatsUpdate(update) => {
                if let Some(memory) = update.memory {
                    self.memory = memory;
                }
                if let Some(cpu) = update.cpu {
                    self.cpu = cpu;
                }
                if let Some(network) = update.network {
                    self.network = network;
                }
                if let Some(disk) = update.disk {
                    self.disk = disk;
                }
                if let Some(storage) = update.storage {
                    self.storage = storage;
                }
            }
            Message::ConfigFileChanged(configuration) => {
                self.configuration = configuration;
                self.refresh_indicator_font();
                return cosmic::task::message(Message::StartMonitoring);
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let is_horizontal = is_horizontal(&self.core);

        let indicators = [
            self.cpu.content(&self.configuration),
            self.memory.content(&self.configuration),
            self.network.content(&self.configuration),
            self.disk.content(&self.configuration),
            self.storage.content(&self.configuration, is_horizontal),
        ];
        let mut contents: Vec<Element<Message>> = indicators
            .into_iter()
            .flatten()
            .map(|content| indicator(self, is_horizontal, content))
            .collect();
        if contents.is_empty() {
            contents.push(no_indicators_content(NoIndicatorProps {
                icon: self.app_icons.get(APP_LOGO_ICON),
                size: self.icon_size(),
            }));
        }

        let show_separators = self.configuration.general.show_indicator_separators;
        let mut elements: Vec<Element<Message>> = Vec::new();
        if show_separators {
            elements.push(divider::vertical::default().into());
        }
        for content in contents {
            elements.push(content);
            if show_separators {
                elements.push(divider::vertical::default().into());
            }
        }

        let wrapper: Element<Message> = if is_horizontal {
            Row::from_vec(elements)
                .align_y(Alignment::Center)
                .spacing(DEFAULT_INDICATOR_SPACING)
                .into()
        } else {
            Column::from_vec(elements).align_x(Alignment::Center).into()
        };

        // The minor-axis padding, which is what `suggested_padding` returned before it also reported the major axis.
        let (_, applet_padding) = self.core.applet.suggested_padding(false);
        let padding = if is_horizontal {
            [applet_padding, applet_padding]
        } else {
            [applet_padding, 1]
        };

        let button = widget::button::custom(wrapper)
            .class(cosmic::theme::Button::AppletIcon)
            .padding(padding)
            .on_press(Message::OpenSettingsWindow);

        autosize::autosize(container(button), AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    /// The interface font comes from the theme, so it's refreshed here when the theme or theme mode changes.
    fn system_theme_update(
        &mut self,
        _keys: &[&'static str],
        new_theme: &cosmic::cosmic_theme::Theme,
    ) -> Task<Self::Message> {
        let interface_font = Font::from(&new_theme.font);
        if interface_font != self.interface_font {
            self.interface_font = interface_font;
            self.refresh_indicator_font();
            self.app_text_measurements.reset();
        }
        Task::none()
    }
}

impl AppState {
    pub fn core(&self) -> &Core {
        &self.core
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
    pub fn indicator_font(&self) -> &Font {
        &self.indicator_font
    }

    fn stop_monitoring(&mut self) {
        if let Some(token) = self.monitor_cancellation_token.take() {
            info!("Stopping monitors");
            token.cancel();
        }
    }

    /// Launch the settings window as a separate process, unless it is already running.
    fn open_settings_window(&mut self) {
        if let Some(child) = self.settings_process.as_mut() {
            match child.try_wait() {
                Ok(None) => {
                    info!("Settings window is already open");
                    return;
                }
                // It has exited, and is now reaped. Launch a new one.
                Ok(Some(_)) => {}
                Err(err) => error!("Failed to check whether the settings window is open: {}", err),
            }
        }

        let executable = match std::env::current_exe() {
            Ok(executable) => executable,
            Err(err) => {
                error!("Failed to find the applet executable to open the settings window: {}", err);
                return;
            }
        };

        // The panel only passes WAYLAND_DISPLAY to applets whose desktop entry sets X-HostWaylandDisplay. Without it,
        // the settings window can't reach the desktop compositor.
        if std::env::var_os("WAYLAND_DISPLAY").is_none() {
            warn!("WAYLAND_DISPLAY is not set, the settings window may fail to open");
        }

        info!("Opening settings window");

        let mut command = Command::new(executable);
        command.arg(SETTINGS_FLAG);
        if let Some(slot) = font_size_slot(&self.core) {
            command.arg(slot.to_arg());
        }

        // The privileged socket belongs to the applet, the settings window has no use for it.
        command
            .env_remove("X_PRIVILEGED_WAYLAND_SOCKET")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        match command.spawn() {
            Ok(child) => self.settings_process = Some(child),
            Err(err) => error!("Failed to open the settings window: {}", err),
        }
    }

    fn refresh_indicator_font(&mut self) {
        self.indicator_font = app_fonts::resolve_font(&self.configuration.general.font, &self.interface_font);
    }

    pub fn font_size(&self, horizontal: bool) -> u16 {
        FontSizeSlot::from_applet_size(&self.core.applet.size, horizontal)
            .map_or(DEFAULT_INDICATOR_FONT_SIZE, |slot| self.configuration.general.font_size(slot))
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

fn is_horizontal(core: &Core) -> bool {
    matches!(core.applet.anchor, PanelAnchor::Top | PanelAnchor::Bottom)
}

/// The font size slot for the panel/dock the applet is running in, which is the one the settings form edits.
fn font_size_slot(core: &Core) -> Option<FontSizeSlot> {
    FontSizeSlot::from_applet_size(&core.applet.size, is_horizontal(core))
}
