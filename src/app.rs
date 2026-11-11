// SPDX-License-Identifier: GPL-3.0-only

use crate::configuration::app_configuration::{
    AppConfiguration, CPU_SETTINGS_WINDOW_ID, DISK_SETTINGS_WINDOW_ID, GENERAL_SETTINGS_WINDOW_ID,
    MEMORY_SETTINGS_WINDOW_ID, NETWORK_SETTINGS_WINDOW_ID,
};
use crate::configuration::general::FontSizeSlot;
use crate::core::app_colours::AppColours;
use crate::core::app_fonts::{self, FontOptions};
use crate::core::app_icons::{AppIcons, APP_LOGO_ICON};
use crate::core::app_text_measurements::AppTextMeasurements;
use crate::core::settings::{SettingsForm, SettingsFormEvent};
use crate::monitors::cpu_monitor::{CpuMonitor, CpuStats};
use crate::monitors::disk_monitor::{DiskMonitor, DiskStats};
use crate::monitors::memory_monitor::{MemoryMonitor, MemoryStats};
use crate::monitors::network_monitor::{NetworkMonitor, NetworkStats};
use crate::sensors::proc_disk_stats_reader::ProcDiskStatsReader;
use crate::sensors::proc_meminfo_reader::ProcMemInfoSensorReader;
use crate::sensors::proc_net_dev_reader::ProcNetDevReader;
use crate::sensors::proc_stat_reader::ProcStatSensorReader;
use crate::ui::components::no_indicator::{no_indicators_content, NoIndicatorProps};
use crate::fl;
use cosmic::app::{Core, Task};
use cosmic::applet::cosmic_panel_config::{PanelAnchor, PanelSize};
use cosmic::config::{CosmicTk, COSMIC_TK};
use cosmic::cosmic_config::{Config, CosmicConfigEntry};
use cosmic::font::Font;
use cosmic::iced::{window, Subscription};
use cosmic::iced::{Alignment, Length, Size};
use cosmic::iced_widget::{column, row, Column, Row};
use cosmic::widget;
use cosmic::widget::{autosize, container, divider, nav_bar, scrollable, segmented_button, Id};
use cosmic::{cosmic_config, Application, Element};
use log::{error, info};
use once_cell::sync::Lazy;
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

pub const GLOBAL_APP_ID: &'static str = "dev.eidolon.cosmic-ext-applet-vitals";

static AUTOSIZE_MAIN_ID: Lazy<Id> = Lazy::new(|| Id::new("autosize-main"));

const DEFAULT_INDICATOR_FONT_SIZE: u16 = 11;
const DEFAULT_INDICATOR_ICON_SIZE: u16 = 11;
const DEFAULT_INDICATOR_SPACING: u16 = 8;

const SETTINGS_WINDOW_DEFAULT_SIZE: Size = Size::new(760.0, 600.0);
const SETTINGS_WINDOW_MIN_SIZE: Size = Size::new(560.0, 400.0);

#[derive(Default)]
pub struct AppState {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The cancellation token to stop the status updates
    monitor_cancellation_token: Option<CancellationToken>,
    interface_font: Font,
    indicator_font: Font,
    label_font: Font,
    /// Only populated while the settings window is open.
    font_options: FontOptions,
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
    /// The id of the settings window, if it is currently open.
    settings_window: Option<window::Id>,
    /// The navigation model for the settings window sidebar. Each entry holds the id of its settings form.
    settings_nav: segmented_button::SingleSelectModel,
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
}

/// The messages processed by the application update
#[derive(Debug, Clone)]
pub enum Message {
    /// Open the settings window, or focus it if it is already open
    OpenSettingsWindow,
    /// Close the settings window
    CloseSettingsWindow,
    /// Start an interactive move of the settings window from its header bar
    DragSettingsWindow,
    /// A window was closed
    WindowClosed(window::Id),
    /// A section was selected in the settings window sidebar
    SettingsNavSelected(segmented_button::Entity),
    /// Start monitoring the system resources
    StartMonitoring,
    /// One or more monitors were polled. Updates are batched so each tick causes a single redraw.
    StatsUpdate(StatsUpdate),
    /// The user has updated the settings form
    SettingsFormUpdate(SettingsFormEvent),
    /// The configuration file was changed externally
    ConfigFileChanged(AppConfiguration),
    /// The COSMIC DE theme was changed
    ThemeChanged(CosmicTk),
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

        let settings_forms = configuration.settings_form_options(font_size_slot(&core));
        let app_colours = AppColours::from(&core.system_theme().cosmic().palette);
        let app_icons = AppIcons::new();
        let app_text_measurements = AppTextMeasurements::new();

        let interface_font: Font = COSMIC_TK
            .read()
            .map(|cosmic_tk| cosmic_tk.interface_font.clone().into())
            .unwrap_or_default();
        let indicator_font = app_fonts::resolve_font(&configuration.general.font, &interface_font);
        let label_font = app_fonts::label_font(&indicator_font, configuration.general.bold_labels);

        let settings_nav = build_settings_nav(&settings_forms);

        let app = AppState {
            core,
            settings_forms,
            settings_nav,
            interface_font,
            indicator_font,
            label_font,
            app_colours,
            app_icons,
            app_text_measurements,
            configuration,
            ..Default::default()
        };

        (app, cosmic::task::message(Message::StartMonitoring))
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::WindowClosed(id))
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subscriptions = Vec::new();

        subscriptions.push(
            self.core()
                .watch_config::<AppConfiguration>(Self::APP_ID)
                .map(|update| Message::ConfigFileChanged(update.config)),
        );

        subscriptions.push(
            self.core()
                .watch_config::<CosmicTk>("com.system76.CosmicTk")
                .map(|tk| Message::ThemeChanged(tk.config)),
        );

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::OpenSettingsWindow => {
                if let Some(id) = self.settings_window {
                    return window::gain_focus(id);
                }

                info!("Opening settings window");

                // Ensure the configuration is up to date when we open the settings form.
                self.refresh_configuration_from_disk();
                self.font_options = FontOptions::new(&self.configuration.general.font);

                // Every message rebuilds and redraws all open windows, so pause the monitors while the
                // settings window is open. They are restarted with the new configuration when it closes.
                self.stop_monitoring();

                let (id, open) = window::open(window::Settings {
                    size: SETTINGS_WINDOW_DEFAULT_SIZE,
                    min_size: Some(SETTINGS_WINDOW_MIN_SIZE),
                    decorations: false,
                    transparent: true,
                    resizable: true,
                    ..Default::default()
                });

                self.settings_window = Some(id);

                return Task::batch([
                    open.discard(),
                    self.core.set_title(Some(id), fl!("settings-title")),
                ]);
            }
            Message::CloseSettingsWindow => {
                if let Some(id) = self.settings_window {
                    return window::close(id);
                }
            }
            Message::DragSettingsWindow => {
                if let Some(id) = self.settings_window {
                    return self.core.drag(Some(id));
                }
            }
            Message::WindowClosed(id) => {
                if self.settings_window == Some(id) {
                    info!("Closing settings window");
                    self.settings_window = None;
                    self.font_options = FontOptions::default();
                    self.save_configuration();
                    return cosmic::task::message(Message::StartMonitoring);
                }
            }
            Message::SettingsNavSelected(entity) => {
                self.settings_nav.activate(entity);
            }
            Message::StartMonitoring => {
                self.stop_monitoring();

                if self.settings_window.is_some() {
                    info!("Settings window is open, monitoring will start when it closes");
                    return Task::none();
                }

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

                    // A single scheduler drives every monitor. Monitors that fall due at (nearly) the same
                    // time are polled together and sent as one message, because every message rebuilds and
                    // redraws the applet. Separate timers per monitor would wake the UI once per monitor.
                    let start = Instant::now();
                    let mut schedules = [
                        memory_monitor.is_some().then_some((start, config.memory.update_interval)),
                        cpu_monitor.is_some().then_some((start, config.cpu.update_interval)),
                        network_monitor.is_some().then_some((start, config.network.update_interval)),
                        disk_monitor.is_some().then_some((start, config.disk.update_interval)),
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
                        let mut due = [false; 4];

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
            }
            Message::ConfigFileChanged(configuration) => {
                self.configuration = configuration;
                self.refresh_indicator_font();
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
            Message::ThemeChanged(cosmic_tk) => {
                self.interface_font = cosmic_tk.interface_font.into();
                self.refresh_indicator_font();
                self.app_text_measurements.reset();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let is_horizontal = is_horizontal(&self.core);

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
            [
                self.core.applet.suggested_padding(false),
                self.core.applet.suggested_padding(false),
            ]
        } else {
            [self.core.applet.suggested_padding(false), 1]
        };

        let button = widget::button::custom(wrapper)
            .class(cosmic::theme::Button::AppletIcon)
            .padding(padding)
            .on_press(Message::OpenSettingsWindow);

        autosize::autosize(container(button), AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn view_window(&'_ self, id: window::Id) -> Element<'_, Self::Message> {
        if self.settings_window != Some(id) {
            return container(row![]).into();
        }

        let focused = self.core.focused_window() == Some(id);

        let header = widget::header_bar()
            .title(fl!("settings-title"))
            .focused(focused)
            .on_drag(Message::DragSettingsWindow)
            .on_close(Message::CloseSettingsWindow);

        let nav = nav_bar(&self.settings_nav, Message::SettingsNavSelected)
            .into_container()
            .width(Length::Fixed(220.0))
            .height(Length::Fill);

        let form: Element<Message> = match self
            .settings_nav
            .active_data::<window::Id>()
            .and_then(|form_id| self.settings_forms.get(form_id))
        {
            None => container(row!["No settings window configured."]).into(),
            Some(form) => scrollable(container(form.draw(&self)).width(Length::Fill).padding([0, 16, 16, 0]))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        };

        let body = row![nav, form].spacing(16).padding([8, 8, 0, 8]);

        container(column![header, body])
            .width(Length::Fill)
            .height(Length::Fill)
            .class(cosmic::theme::Container::custom(|theme| {
                let cosmic = theme.cosmic();
                container::Style {
                    text_color: Some(cosmic.background.on.into()),
                    icon_color: Some(cosmic.background.on.into()),
                    background: Some(cosmic::iced::Color::from(cosmic.background.base).into()),
                    border: cosmic::iced::Border {
                        radius: cosmic.corner_radii.radius_s.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }))
            .into()
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
    pub fn indicator_font(&self) -> &Font {
        &self.indicator_font
    }
    pub fn label_font(&self) -> &Font {
        &self.label_font
    }
    pub fn font_options(&self) -> &FontOptions {
        &self.font_options
    }

    fn stop_monitoring(&mut self) {
        if let Some(token) = self.monitor_cancellation_token.take() {
            info!("Stopping monitors");
            token.cancel();
        }
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

        let general_settings_form = self
            .settings_forms
            .get(&GENERAL_SETTINGS_WINDOW_ID.clone())
            .expect("No general settings form configured.");

        self.configuration = AppConfiguration {
            general: self.configuration.general.update(general_settings_form, font_size_slot(&self.core)),
            memory: self.configuration.memory.update(memory_settings_form),
            cpu: self.configuration.cpu.update(cpu_settings_form),
            network: self.configuration.network.update(network_settings_form),
            disk: self.configuration.disk.update(disk_settings_form),
            ..Default::default()
        };

        self.refresh_indicator_font();
    }

    fn refresh_indicator_font(&mut self) {
        self.indicator_font = app_fonts::resolve_font(&self.configuration.general.font, &self.interface_font);
        self.label_font = app_fonts::label_font(&self.indicator_font, self.configuration.general.bold_labels);
    }

    fn refresh_configuration_from_disk(&mut self) {
        if let Ok(helper) = Config::new(Self::APP_ID, AppConfiguration::VERSION) {
            let loaded = AppConfiguration::get_entry(&helper).unwrap_or_else(|(errs, cfg)| {
                error!("Errors while loading configuration: {:?}", errs);
                cfg
            });

            self.configuration = loaded;
            self.settings_forms = self.configuration.settings_form_options(font_size_slot(&self.core));
            self.settings_nav = build_settings_nav(&self.settings_forms);
            self.refresh_indicator_font();
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

/// Build the settings window sidebar, with one entry per settings form. The first entry is selected.
fn build_settings_nav(settings_forms: &BTreeMap<window::Id, SettingsForm>) -> segmented_button::SingleSelectModel {
    let mut model = segmented_button::SingleSelectModel::default();

    for (index, (form_id, form)) in settings_forms.iter().enumerate() {
        let entity = model.insert().text(form.title.clone()).data(*form_id);
        if index == 0 {
            entity.activate();
        }
    }

    model
}
