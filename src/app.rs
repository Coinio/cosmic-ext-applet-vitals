// SPDX-License-Identifier: GPL-3.0-only

use crate::configuration::app_configuration::AppConfiguration;
use crate::configuration::general::FontSizeSlot;
use crate::core::app_fonts;
use crate::core::app_icons::{AppIcons, APP_LOGO_ICON};
use crate::core::app_text_measurements::AppTextMeasurements;
use crate::monitors::cpu_monitor::{CpuMonitor, CpuStats};
use crate::monitors::cpu_sensors_monitor::{CpuSensorStats, CpuSensorsMonitor};
use crate::monitors::disk_monitor::{DiskMonitor, DiskStats};
use crate::monitors::memory_monitor::{MemoryMonitor, MemoryStats};
use crate::monitors::network_monitor::{NetworkMonitor, NetworkStats};
use crate::monitors::storage_monitor::{StorageMonitor, StorageStats};
use crate::sensors::cpu_topology_reader::{CpuTopology, SYSFS_CPU_ROOT};
use crate::sensors::hwmon_reader::HWMON_ROOT;
use crate::sensors::proc_disk_stats_reader::ProcDiskStatsReader;
use crate::sensors::proc_meminfo_reader::ProcMemInfoSensorReader;
use crate::sensors::proc_net_dev_reader::ProcNetDevReader;
use crate::sensors::proc_stat_reader::ProcStatSensorReader;
use crate::sensors::storage_reader::StorageReader;
use crate::settings_app::{SettingsPage, SettingsPageRequest, SETTINGS_FLAG};
use crate::ui::components::indicator_area::indicator_area;
use crate::ui::components::indicator::indicator;
use crate::ui::components::indicator_strip::indicator_strip;
use crate::ui::components::no_indicator::{no_indicators_content, NoIndicatorProps};
use cosmic::app::{Core, Task};
use cosmic::applet::cosmic_panel_config::{PanelAnchor, PanelSize};
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::font::Font;
use cosmic::iced::platform_specific::runtime::wayland::popup::SctkPopupSettings;
use cosmic::iced::platform_specific::shell::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{window, Alignment, Padding, Point, Rectangle, Size, Subscription};
use cosmic::widget;
use cosmic::widget::{autosize, container, divider, Id};
use cosmic::{cosmic_config, Application, Element};
use log::{error, info, warn};
use once_cell::sync::Lazy;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

pub const GLOBAL_APP_ID: &'static str = "dev.eidolon.cosmic-ext-applet-vitals";

static AUTOSIZE_MAIN_ID: Lazy<Id> = Lazy::new(|| Id::new("autosize-main"));

const DEFAULT_INDICATOR_FONT_SIZE: u16 = 11;
const DEFAULT_INDICATOR_ICON_SIZE: u16 = 11;
const DEFAULT_INDICATOR_SPACING: u16 = 8;

/// How long the cursor has to rest on an indicator before its detail popup opens, so sweeping the cursor across the
/// panel doesn't flash popups open.
const DETAIL_POPUP_DELAY: Duration = Duration::from_millis(400);

/// Identifies an indicator, e.g. to know which one's detail popup to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorKind {
    Cpu,
    Memory,
    Network,
    Disk,
    Storage,
}

impl IndicatorKind {
    /// Whether hovering over the indicator shows a popup with more detail.
    fn has_detail(self) -> bool {
        matches!(self, IndicatorKind::Cpu | IndicatorKind::Storage)
    }

    /// The settings page that clicking the indicator opens.
    fn settings_page(self) -> SettingsPage {
        match self {
            IndicatorKind::Cpu => SettingsPage::Cpu,
            IndicatorKind::Memory => SettingsPage::Memory,
            IndicatorKind::Network => SettingsPage::Network,
            IndicatorKind::Disk => SettingsPage::Disk,
            IndicatorKind::Storage => SettingsPage::Storage,
        }
    }
}

/// The detail popup for the indicator under the cursor. It opens after [`DETAIL_POPUP_DELAY`].
#[derive(Debug)]
struct DetailPopup {
    kind: IndicatorKind,
    /// The indicator's bounds in the applet, which the popup is anchored to.
    bounds: Rectangle,
    /// Identifies this hover, so the delayed open of an earlier hover is ignored.
    hover: u64,
    /// The popup's window, once it has opened.
    window_id: Option<window::Id>,
}

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
    /// The latest cpu temperature and fan speed, which are only read while the cpu detail popup is wanted.
    cpu_sensors: CpuSensorStats,
    /// Tells the monitors whether the cpu detail popup is wanted, so they only read its sensors while it is.
    cpu_detail_wanted: watch::Sender<bool>,
    /// The current network usage stats
    network: NetworkStats,
    /// The current disk usage stats   
    disk: DiskStats,
    /// The current available and total storage
    storage: StorageStats,
    /// The settings window process, if one has been launched. See [`crate::settings_app`] for why it is separate.
    settings_process: Option<Child>,
    /// The detail popup for the hovered indicator, if one is hovered.
    detail_popup: Option<DetailPopup>,
    /// Counts the hovers over indicators, to identify each one.
    hover_count: u64,
}

/// Monitors that fall due within this window of each other are polled together, so they share one redraw.
const MONITOR_COALESCE_WINDOW: Duration = Duration::from_millis(50);

/// The results of a single monitor tick. Only the monitors that were due are populated.
#[derive(Debug, Clone, Default)]
pub struct StatsUpdate {
    memory: Option<MemoryStats>,
    cpu: Option<CpuStats>,
    cpu_sensors: Option<CpuSensorStats>,
    network: Option<NetworkStats>,
    disk: Option<DiskStats>,
    storage: Option<StorageStats>,
}

/// The messages processed by the application update
#[derive(Debug, Clone)]
pub enum Message {
    /// Open the settings window on a page, or show the page if the window is already open
    OpenSettingsWindow(SettingsPage),
    /// Start monitoring the system resources
    StartMonitoring,
    /// One or more monitors were polled. Updates are batched so each tick causes a single redraw.
    StatsUpdate(StatsUpdate),
    /// The configuration file was changed externally
    ConfigFileChanged(AppConfiguration),
    /// The cursor entered an indicator, which has these bounds
    IndicatorHovered(IndicatorKind, Rectangle),
    /// The cursor left an indicator
    IndicatorUnhovered(IndicatorKind),
    /// The cursor has rested on an indicator long enough to open its detail popup. Identifies the hover.
    OpenDetailPopup(u64),
    /// A popup was closed by the compositor
    PopupClosed(window::Id),
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
        let task = self.handle_message(message);
        self.sync_cpu_detail_wanted();
        task
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let is_horizontal = is_horizontal(&self.core);

        let indicators = [
            (IndicatorKind::Cpu, self.cpu.content(&self.configuration)),
            (IndicatorKind::Memory, self.memory.content(&self.configuration)),
            (IndicatorKind::Network, self.network.content(&self.configuration)),
            (IndicatorKind::Disk, self.disk.content(&self.configuration)),
            (IndicatorKind::Storage, self.storage.content(&self.configuration, is_horizontal)),
        ];
        let mut contents: Vec<(Element<Message>, Option<IndicatorKind>)> = indicators
            .into_iter()
            .filter_map(|(kind, content)| Some((indicator(self, is_horizontal, content?), Some(kind))))
            .collect();
        if contents.is_empty() {
            let content = no_indicators_content(NoIndicatorProps {
                icon: self.app_icons.get(APP_LOGO_ICON),
                size: self.icon_size(),
            });
            contents.push((content, None));
        }

        // The minor-axis padding, which is what `suggested_padding` returned before it also reported the major axis.
        let (_, applet_padding) = self.core.applet.suggested_padding(false);
        let applet_padding = f32::from(applet_padding);
        let show_separators = self.configuration.general.show_indicator_separators;

        // The applet's padding, and the gaps between indicators, belong to the indicators rather than the button and
        // strip, so each indicator's hover area and highlight reach the applet's edges and the next indicator.
        // Separators keep the applet's padding along the panel, so they aren't drawn on its edge.
        let across = if is_horizontal { applet_padding } else { 1.0 };
        let gap = if is_horizontal { f32::from(DEFAULT_INDICATOR_SPACING) } else { 0.0 };
        let half_gap = gap / 2.0;
        let spacing = if show_separators { half_gap } else { 0.0 };
        let edge = if show_separators { half_gap } else { applet_padding };
        let count = contents.len();

        let separator = || -> Element<Message> {
            // Inset across the panel, so it's the length of the indicators' text rather than the applet's thickness.
            let inset = if is_horizontal { [applet_padding, 0.0] } else { [0.0, 0.0] };
            container(divider::vertical::default()).padding(inset).into()
        };

        let mut elements: Vec<Element<Message>> = Vec::new();
        if show_separators {
            elements.push(separator());
        }
        for (index, (content, kind)) in contents.into_iter().enumerate() {
            let before = if index == 0 { edge } else { half_gap };
            let after = if index + 1 == count { edge } else { half_gap };
            let padding = if is_horizontal {
                Padding { top: across, right: after, bottom: across, left: before }
            } else {
                Padding { top: before, right: across, bottom: after, left: across }
            };
            let content: Element<Message> = container(content)
                .padding(padding)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into();
            let page = kind.map_or(SettingsPage::General, IndicatorKind::settings_page);
            let mut area = indicator_area(content, Message::OpenSettingsWindow(page));
            if let Some(kind) = kind.filter(|kind| kind.has_detail()) {
                area = area.on_hover(
                    move |bounds| Message::IndicatorHovered(kind, bounds),
                    Message::IndicatorUnhovered(kind),
                );
            }
            elements.push(area.into());
            if show_separators {
                elements.push(separator());
            }
        }

        let wrapper = indicator_strip(is_horizontal, spacing, elements);

        let padding = match (show_separators, is_horizontal) {
            (false, _) => Padding::ZERO,
            (true, true) => Padding::from([0.0, applet_padding]),
            (true, false) => Padding::from([applet_padding, 0.0]),
        };

        // The indicators highlight themselves, so the button keeps its resting look while hovered or pressed rather
        // than highlighting the whole applet. Clicking it away from an indicator, e.g. on a separator, opens the
        // general settings.
        let resting = |focused: bool, theme: &cosmic::Theme| {
            widget::button::Catalog::active(theme, focused, false, &cosmic::theme::Button::AppletIcon)
        };
        let button = widget::button::custom(wrapper)
            .class(cosmic::theme::Button::Custom {
                active: Box::new(resting),
                disabled: Box::new(move |theme| resting(false, theme)),
                hovered: Box::new(resting),
                pressed: Box::new(resting),
            })
            .padding(padding)
            .on_press(Message::OpenSettingsWindow(SettingsPage::General));

        autosize::autosize(container(button), AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Self::Message> {
        let content = match &self.detail_popup {
            Some(popup) if popup.window_id == Some(id) => self.detail_popup_content(popup.kind),
            _ => None,
        };
        match content {
            Some(content) => self.core.applet.popup_container(content).into(),
            None => widget::Space::new().into(),
        }
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        Some(Message::PopupClosed(id))
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
    fn handle_message(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenSettingsWindow(page) => {
                self.open_settings_window(page);
                return self.close_detail_popup();
            }
            Message::StartMonitoring => {
                self.stop_monitoring();

                info!("Starting monitoring");

                let cancellation_token = CancellationToken::new();
                self.monitor_cancellation_token = Some(cancellation_token.clone());

                let config = self.configuration.clone();
                let mut cpu_detail_wanted = self.cpu_detail_wanted.subscribe();

                return cosmic::Task::stream(async_stream::stream! {
                    let mut memory_monitor = (!config.memory.hide_indicator)
                        .then(|| MemoryMonitor::new(ProcMemInfoSensorReader, &config));
                    let mut cpu_monitor = (!config.cpu.hide_indicator).then(|| {
                        let topology = config.cpu.combine_cores.then(|| CpuTopology::read(Path::new(SYSFS_CPU_ROOT)));
                        CpuMonitor::new(ProcStatSensorReader, &config, topology)
                    });
                    // The temperature and fan are only shown in the cpu detail popup, so they're only read while
                    // it's wanted. Some sensor drivers are slow to read, or wake their device to do it.
                    let cpu_sensors_monitor = (!config.cpu.hide_indicator)
                        .then(|| CpuSensorsMonitor::new(Path::new(HWMON_ROOT), &config.cpu));
                    let mut network_monitor = (!config.network.hide_indicator)
                        .then(|| NetworkMonitor::new(ProcNetDevReader, &config));
                    let mut disk_monitor = (!config.disk.hide_indicator)
                        .then(|| DiskMonitor::new(ProcDiskStatsReader, &config));
                    let mut storage_monitor = (!config.storage.hide_indicator)
                        .then(|| StorageMonitor::new(StorageReader, config.storage.mounts.clone()));

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

                        // Set when the cpu detail popup has just become wanted, to read its sensors straight away
                        // instead of at the next cpu tick.
                        let mut cpu_sensors_requested = false;

                        tokio::select! {
                            _ = tokio::time::sleep_until(next_due) => {},
                            changed = cpu_detail_wanted.changed(), if cpu_sensors_monitor.is_some() => {
                                // The sender lives as long as the applet, so this only fails as it exits.
                                if changed.is_err() {
                                    break;
                                }
                                cpu_sensors_requested = *cpu_detail_wanted.borrow();
                            },
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

                        let read_cpu_sensors = cpu_sensors_requested || (due[1] && *cpu_detail_wanted.borrow());

                        // Waking for the cpu detail popup can find nothing else due, and nothing to read when it
                        // stopped being wanted.
                        if !read_cpu_sensors && !due.contains(&true) {
                            continue;
                        }

                        let update = StatsUpdate {
                            memory: memory_monitor.as_mut().filter(|_| due[0]).map(|m| m.poll().unwrap_or_default()),
                            cpu: cpu_monitor.as_mut().filter(|_| due[1]).map(|m| m.poll().unwrap_or_default()),
                            cpu_sensors: cpu_sensors_monitor.as_ref().filter(|_| read_cpu_sensors).map(|m| m.poll()),
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
                if let Some(cpu_sensors) = update.cpu_sensors {
                    self.cpu_sensors = cpu_sensors;
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
                // The hovered indicator may have been hidden, which leaves nothing to report that it's no longer hovered.
                return self
                    .close_detail_popup()
                    .chain(cosmic::task::message(Message::StartMonitoring));
            }
            Message::IndicatorHovered(kind, bounds) => {
                let close = self.close_detail_popup();

                self.hover_count += 1;
                let hover = self.hover_count;
                self.detail_popup = Some(DetailPopup {
                    kind,
                    bounds,
                    hover,
                    window_id: None,
                });

                let open = cosmic::task::future(async move {
                    tokio::time::sleep(DETAIL_POPUP_DELAY).await;
                    Message::OpenDetailPopup(hover)
                });
                return Task::batch([close, open]);
            }
            Message::IndicatorUnhovered(kind) => {
                // Moving between indicators can report entering the next one before leaving this one.
                if self.detail_popup.as_ref().is_some_and(|popup| popup.kind == kind) {
                    return self.close_detail_popup();
                }
            }
            Message::OpenDetailPopup(hover) => return self.open_detail_popup(hover),
            Message::PopupClosed(id) => {
                if self.detail_popup.as_ref().is_some_and(|popup| popup.window_id == Some(id)) {
                    self.detail_popup = None;
                }
            }
        }
        Task::none()
    }

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

    /// Launch the settings window as a separate process on `page`. If it's already running, ask it to show `page`.
    fn open_settings_window(&mut self, page: SettingsPage) {
        if let Some(child) = self.settings_process.as_mut() {
            match child.try_wait() {
                Ok(None) => {
                    info!("Settings window is already open, showing the {:?} page", page);
                    SettingsPageRequest::send(page);
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
        command.arg(page.to_arg());

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

    /// Open the detail popup for the hovered indicator, if it is still the hover that asked for it.
    fn open_detail_popup(&mut self, hover: u64) -> Task<Message> {
        let Some(parent) = self.core.main_window_id() else {
            return Task::none();
        };
        let Some(popup) = self.detail_popup.as_mut().filter(|popup| popup.hover == hover) else {
            return Task::none();
        };
        if popup.window_id.is_some() {
            return Task::none();
        }

        let id = window::Id::unique();
        popup.window_id = Some(id);
        get_popup(detail_popup_settings(&self.core, parent, id, popup.bounds))
    }

    /// The cpu detail popup is wanted from when the cpu indicator is hovered, so its sensors have been read by the
    /// time it opens, until it closes.
    fn sync_cpu_detail_wanted(&self) {
        let wanted = self.detail_popup.as_ref().is_some_and(|popup| popup.kind == IndicatorKind::Cpu);
        self.cpu_detail_wanted.send_if_modified(|current| {
            let changed = *current != wanted;
            *current = wanted;
            changed
        });
    }

    fn close_detail_popup(&mut self) -> Task<Message> {
        match self.detail_popup.take().and_then(|popup| popup.window_id) {
            Some(id) => destroy_popup(id),
            None => Task::none(),
        }
    }

    fn detail_popup_content(&self, kind: IndicatorKind) -> Option<Element<'_, Message>> {
        match kind {
            IndicatorKind::Cpu => Some(self.cpu.detail(&self.cpu_sensors, &self.configuration)),
            IndicatorKind::Storage => Some(self.storage.detail(&self.configuration)),
            IndicatorKind::Memory | IndicatorKind::Network | IndicatorKind::Disk => None,
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

/// A popup anchored to the indicator with `bounds`, beyond the panel's edge, which doesn't take the pointer or
/// keyboard, so the indicator stays hovered and the panel keeps working while it's open.
fn detail_popup_settings(core: &Core, parent: window::Id, id: window::Id, bounds: Rectangle) -> SctkPopupSettings {
    let mut settings = core.applet.get_popup_settings(parent, id, None, None, None);

    // The default anchor spans the whole applet. Narrow it to the indicator along the panel, and keep the applet's
    // thickness across it so the popup clears the panel.
    let anchor = &mut settings.positioner.anchor_rect;
    if is_horizontal(core) {
        anchor.x = bounds.x.round() as i32;
        anchor.width = bounds.width.round() as i32;
    } else {
        anchor.y = bounds.y.round() as i32;
        anchor.height = bounds.height.round() as i32;
    }

    settings.grab = false;
    // An empty input region, placed off the surface as the COSMIC applet tooltips do, so the pointer passes through.
    settings.input_zone = Some(vec![Rectangle::new(Point::new(-1000.0, -1000.0), Size::default())]);
    settings
}

/// The font size slot for the panel/dock the applet is running in, which is the one the settings form edits.
fn font_size_slot(core: &Core) -> Option<FontSizeSlot> {
    FontSizeSlot::from_applet_size(&core.applet.size, is_horizontal(core))
}
