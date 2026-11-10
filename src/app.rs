// SPDX-License-Identifier: GPL-3.0-only

use crate::configuration::app_configuration::{
    AppConfiguration, CPU_SETTINGS_WINDOW_ID, DISK_SETTINGS_WINDOW_ID, GENERAL_SETTINGS_WINDOW_ID,
    MEMORY_SETTINGS_WINDOW_ID, NETWORK_SETTINGS_WINDOW_ID,
};
use crate::core::app_colours::AppColours;
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
    /// The active system font
    active_interface_font: Font,
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

        let settings_forms = configuration.settings_form_options();
        let app_colours = AppColours::from(&core.system_theme().cosmic().palette);
        let app_icons = AppIcons::new();
        let app_text_measurements = AppTextMeasurements::new();
        let active_interface_font: Font = COSMIC_TK
            .read()
            .expect("No COSMIC_TK configuration found")
            .interface_font
            .clone()
            .into();

        let settings_nav = build_settings_nav(&settings_forms);

        let app = AppState {
            core,
            settings_forms,
            settings_nav,
            active_interface_font,
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
                .watch_config("com.system76.CosmicTk")
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
            Message::ThemeChanged(cosmic_tk) => {
                self.active_interface_font = cosmic_tk.interface_font.clone().into();
                self.app_text_measurements.reset();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let is_horizontal = matches!(self.core.applet.anchor, PanelAnchor::Top | PanelAnchor::Bottom);

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
    pub fn active_interface_font(&self) -> &Font {
        &self.active_interface_font
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
            general: self.configuration.general.update(general_settings_form),
            memory: self.configuration.memory.update(memory_settings_form),
            cpu: self.configuration.cpu.update(cpu_settings_form),
            network: self.configuration.network.update(network_settings_form),
            disk: self.configuration.disk.update(disk_settings_form),
            ..Default::default()
        }
    }

    fn refresh_configuration_from_disk(&mut self) {
        if let Ok(helper) = Config::new(Self::APP_ID, AppConfiguration::VERSION) {
            let loaded = AppConfiguration::get_entry(&helper).unwrap_or_else(|(errs, cfg)| {
                error!("Errors while loading configuration: {:?}", errs);
                cfg
            });

            self.configuration = loaded;
            self.settings_forms = self.configuration.settings_form_options();
            self.settings_nav = build_settings_nav(&self.settings_forms);
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
