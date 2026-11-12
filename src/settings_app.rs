// SPDX-License-Identifier: GPL-3.0-only

//! The settings window, which runs as its own process.
//!
//! When the applet is in a panel or dock, cosmic-panel hosts it on a nested Wayland compositor, so any toplevel
//! window the applet opens is embedded in the panel instead of appearing on the desktop. The applet launches this
//! binary again with `--settings` instead, which connects to the desktop compositor and runs as a normal app.
//! Changes are saved as they are made, and the applet picks them up from its configuration watcher.
//!
//! The applet opens the window on the page for the indicator that was clicked. When the window is already open, the
//! applet asks it to switch to that page through a [`SettingsPageRequest`] in the app's state instead.

use crate::app::GLOBAL_APP_ID;
use crate::configuration::app_configuration::{
    AppConfiguration, CPU_SETTINGS_WINDOW_ID, DISK_SETTINGS_WINDOW_ID, GENERAL_SETTINGS_WINDOW_ID,
    GPU_SETTINGS_WINDOW_ID, MEMORY_SETTINGS_WINDOW_ID, NETWORK_SETTINGS_WINDOW_ID, STORAGE_SETTINGS_WINDOW_ID,
};
use crate::configuration::general::FontSizeSlot;
use crate::core::app_colours::AppColour;
use crate::core::app_fonts::FontOptions;
use crate::core::gpu_options::GpuOptions;
use crate::core::mount_options::MountOptions;
use crate::core::sensor_options::SensorOptions;
use crate::core::settings::{SettingsForm, SettingsFormEvent};
use crate::fl;
use crate::sensors::hwmon_reader::HwmonSensorKind;
use crate::ui::components::colour_picker::{ColourPickerEvent, ColourPickerState};
use cosmic::app::{Core, Task};
use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, Config, CosmicConfigEntry};
use cosmic::iced::{window, Length, Limits, Size, Subscription};
use cosmic::iced::widget::row;
use cosmic::widget::{container, nav_bar, scrollable};
use cosmic::{Application, Element};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// The command line flag that runs the binary as the settings window instead of the applet.
pub const SETTINGS_FLAG: &str = "--settings";

/// The command line flag that chooses the page the settings window opens on, e.g. `--page=cpu`.
const PAGE_FLAG: &str = "--page=";

/// A page of the settings window.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsPage {
    #[default]
    General,
    Cpu,
    Memory,
    Gpu,
    Network,
    Disk,
    Storage,
}

impl SettingsPage {
    const ALL: [SettingsPage; 7] = [
        SettingsPage::General,
        SettingsPage::Cpu,
        SettingsPage::Memory,
        SettingsPage::Gpu,
        SettingsPage::Network,
        SettingsPage::Disk,
        SettingsPage::Storage,
    ];

    fn name(self) -> &'static str {
        match self {
            SettingsPage::General => "general",
            SettingsPage::Cpu => "cpu",
            SettingsPage::Memory => "memory",
            SettingsPage::Gpu => "gpu",
            SettingsPage::Network => "network",
            SettingsPage::Disk => "disk",
            SettingsPage::Storage => "storage",
        }
    }

    /// The command line argument that opens the settings window on this page.
    pub fn to_arg(self) -> String {
        format!("{PAGE_FLAG}{}", self.name())
    }

    pub fn from_arg(arg: &str) -> Option<Self> {
        let name = arg.strip_prefix(PAGE_FLAG)?;
        Self::ALL.into_iter().find(|page| page.name() == name)
    }

    /// The id of the settings form shown on this page.
    fn form_id(self) -> window::Id {
        match self {
            SettingsPage::General => *GENERAL_SETTINGS_WINDOW_ID,
            SettingsPage::Cpu => *CPU_SETTINGS_WINDOW_ID,
            SettingsPage::Memory => *MEMORY_SETTINGS_WINDOW_ID,
            SettingsPage::Gpu => *GPU_SETTINGS_WINDOW_ID,
            SettingsPage::Network => *NETWORK_SETTINGS_WINDOW_ID,
            SettingsPage::Disk => *DISK_SETTINGS_WINDOW_ID,
            SettingsPage::Storage => *STORAGE_SETTINGS_WINDOW_ID,
        }
    }
}

/// Asks the open settings window to show a page. It lives in the app's state, which the window watches.
#[derive(Debug, Clone, Default, PartialEq, CosmicConfigEntry)]
#[version = 1]
pub struct SettingsPageRequest {
    /// The page, and when it was asked for, so asking for the page the window was last asked for still changes the
    /// request, after the user has moved to another page.
    pub request: Option<(SettingsPage, u64)>,
}

impl SettingsPageRequest {
    /// Ask the open settings window to show `page`.
    pub fn send(page: SettingsPage) {
        let requested_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64;
        let request = SettingsPageRequest {
            request: Some((page, requested_at)),
        };
        let result = Config::new_state(GLOBAL_APP_ID, SettingsPageRequest::VERSION)
            .and_then(|state| request.write_entry(&state));
        if let Err(err) = result {
            error!("Failed to ask the settings window to show the {} page: {}", page.name(), err);
        }
    }
}

const SETTINGS_WINDOW_DEFAULT_SIZE: Size = Size::new(760.0, 600.0);
const SETTINGS_WINDOW_MIN_SIZE: Size = Size::new(560.0, 400.0);

/// Run the settings window, which opens on `page`. `font_size_slot` is the panel/dock size the applet that opened it
/// is running in.
pub fn run(font_size_slot: Option<FontSizeSlot>, page: SettingsPage) -> cosmic::iced::Result {
    let settings = cosmic::app::Settings::default().size(SETTINGS_WINDOW_DEFAULT_SIZE).size_limits(
        Limits::NONE
            .min_width(SETTINGS_WINDOW_MIN_SIZE.width)
            .min_height(SETTINGS_WINDOW_MIN_SIZE.height),
    );

    cosmic::app::run::<SettingsApp>(settings, SettingsFlags { font_size_slot, page })
}

pub struct SettingsFlags {
    pub font_size_slot: Option<FontSizeSlot>,
    pub page: SettingsPage,
}

pub struct SettingsApp {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The configuration being edited
    configuration: AppConfiguration,
    /// The font size slot of the panel/dock the applet is running in, which is the one the general form edits.
    font_size_slot: Option<FontSizeSlot>,
    /// The fonts that can be selected for the indicators
    font_options: FontOptions,
    /// The mount points that can be included in the storage totals
    mount_options: MountOptions,
    /// The sensors that can be chosen for the CPU temperature
    temperature_sensor_options: SensorOptions,
    /// The sensors that can be chosen for the CPU fan
    fan_sensor_options: SensorOptions,
    /// The GPUs that can be chosen for the panel
    gpu_options: GpuOptions,
    /// The settings forms that are available for configuration of the monitors.
    settings_forms: BTreeMap<window::Id, SettingsForm>,
    /// The navigation model for the sidebar. Each entry holds the id of its settings form.
    settings_nav: nav_bar::Model,
    /// The custom colour picker, while it is open for one of the settings form colour fields.
    custom_colour_picker: Option<CustomColourPicker>,
}

/// The settings form field that a custom colour picker edits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColourPickerTarget {
    pub settings_window_id: window::Id,
    pub form_value_key: &'static str,
}

/// Only one custom colour picker is open at a time, so it remembers which field it is for.
struct CustomColourPicker {
    target: ColourPickerTarget,
    state: ColourPickerState,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// The user has updated the settings form
    SettingsFormUpdate(SettingsFormEvent),
    /// Open the custom colour picker for a settings form field, or close it if it is already open for that field
    ToggleCustomColourPicker(ColourPickerTarget),
    /// The user interacted with the open custom colour picker
    CustomColourPicker(ColourPickerEvent),
    /// The applet asked to show a page, because its indicator was clicked while the window was open
    PageRequested(SettingsPage),
    /// The page request in the app's state was read, without it having changed
    Ignore,
}

impl Application for SettingsApp {
    type Executor = cosmic::executor::Default;
    type Flags = SettingsFlags;
    type Message = Message;

    // Shares the applet's id so the window is matched to its desktop entry and icon.
    const APP_ID: &'static str = GLOBAL_APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        info!("Opening settings window");

        let configuration = load_configuration();
        let settings_forms = configuration.settings_form_options(flags.font_size_slot);
        let settings_nav = build_settings_nav(&settings_forms, flags.page.form_id());

        core.set_header_title(fl!("settings-title"));
        let set_title = core.set_title(core.main_window_id(), fl!("settings-title"));

        let app = SettingsApp {
            core,
            font_options: FontOptions::new(&configuration.general.font),
            mount_options: MountOptions::new(&configuration.storage.mounts),
            temperature_sensor_options: SensorOptions::new(
                HwmonSensorKind::Temperature,
                configuration.cpu.temperature_sensor.as_ref(),
                configuration.cpu.use_fahrenheit,
            ),
            fan_sensor_options: SensorOptions::new(
                HwmonSensorKind::Fan,
                configuration.cpu.fan_sensor.as_ref(),
                configuration.cpu.use_fahrenheit,
            ),
            gpu_options: GpuOptions::new(configuration.gpu.device.as_deref()),
            configuration,
            font_size_slot: flags.font_size_slot,
            settings_forms,
            settings_nav,
            custom_colour_picker: None,
        };

        (app, set_title)
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.settings_nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.settings_nav.activate(id);
        self.custom_colour_picker = None;
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        cosmic_config::config_state_subscription::<_, SettingsPageRequest>(
            std::any::TypeId::of::<SettingsPageRequest>(),
            GLOBAL_APP_ID.into(),
            SettingsPageRequest::VERSION,
        )
        // The subscription first reports the request as it was when the window opened, which changes no keys. That
        // may be from an earlier time the window was open, and the applet chose the opening page on the command line.
        .map(|update| match update.config.request {
            Some((page, _)) if !update.keys.is_empty() => Message::PageRequested(page),
            _ => Message::Ignore,
        })
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::SettingsFormUpdate(settings_form_event) => {
                match settings_form_event {
                    SettingsFormEvent::StringFieldUpdated(value) | SettingsFormEvent::CheckBoxUpdated(value) => {
                        // Choosing a preset colour replaces whatever was being picked in the custom picker.
                        let target = ColourPickerTarget {
                            settings_window_id: value.settings_window_id,
                            form_value_key: value.form_value_key,
                        };
                        if self.custom_colour_picker.as_ref().is_some_and(|picker| picker.target == target) {
                            self.custom_colour_picker = None;
                        }

                        self.set_form_value(target, value.value);
                    }
                }

                self.update_configuration();
            }
            Message::ToggleCustomColourPicker(target) => {
                if self.custom_colour_picker.take().is_some_and(|picker| picker.target == target) {
                    return Task::none();
                }

                let theme = self.core.system_theme().cosmic();
                let current = self
                    .settings_form_value(target)
                    .and_then(AppColour::parse)
                    .unwrap_or(AppColour::ThemeAccent)
                    .resolve(theme);

                self.custom_colour_picker = Some(CustomColourPicker {
                    target,
                    state: ColourPickerState::new(current),
                });
            }
            Message::CustomColourPicker(event) => match event {
                ColourPickerEvent::Cancel => {
                    self.custom_colour_picker = None;
                }
                ColourPickerEvent::Apply => {
                    if let Some(picker) = self.custom_colour_picker.take() {
                        let colour = AppColour::Custom(picker.state.colour());
                        self.set_form_value(picker.target, colour.to_config_value());
                        self.update_configuration();
                    }
                }
                event => {
                    if let Some(picker) = self.custom_colour_picker.as_mut() {
                        picker.state.update(event);
                    }
                }
            },
            Message::PageRequested(page) => {
                let form_id = page.form_id();
                let entity = self
                    .settings_nav
                    .iter()
                    .find(|entity| self.settings_nav.data::<window::Id>(*entity) == Some(&form_id));
                if let Some(entity) = entity {
                    return self.on_nav_select(entity);
                }
            }
            Message::Ignore => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match self
            .settings_nav
            .active_data::<window::Id>()
            .and_then(|form_id| self.settings_forms.get(form_id))
        {
            None => container(row!["No settings window configured."]).into(),
            Some(form) => scrollable(container(form.draw(self)).width(Length::Fill).padding([0, 16, 16, 0]))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        }
    }
}

impl SettingsApp {
    pub fn core(&self) -> &Core {
        &self.core
    }

    pub fn font_options(&self) -> &FontOptions {
        &self.font_options
    }

    pub fn mount_options(&self) -> &MountOptions {
        &self.mount_options
    }

    pub fn sensor_options(&self, kind: HwmonSensorKind) -> &SensorOptions {
        match kind {
            HwmonSensorKind::Temperature => &self.temperature_sensor_options,
            HwmonSensorKind::Fan => &self.fan_sensor_options,
        }
    }

    pub fn gpu_options(&self) -> &GpuOptions {
        &self.gpu_options
    }

    /// The custom colour picker, if it is open for the given settings form field.
    pub fn custom_colour_picker(
        &self,
        settings_window_id: window::Id,
        form_value_key: &'static str,
    ) -> Option<&ColourPickerState> {
        let target = ColourPickerTarget {
            settings_window_id,
            form_value_key,
        };
        self.custom_colour_picker
            .as_ref()
            .filter(|picker| picker.target == target)
            .map(|picker| &picker.state)
    }

    fn settings_form_value(&self, target: ColourPickerTarget) -> Option<&str> {
        self.settings_forms
            .get(&target.settings_window_id)
            .and_then(|form| form.values.get(target.form_value_key))
            .map(|item| item.value.as_str())
    }

    fn set_form_value(&mut self, target: ColourPickerTarget, value: String) {
        let form = self.settings_forms.get_mut(&target.settings_window_id).expect(
            format!("No settings form configured with key: {}", target.settings_window_id).as_str(),
        );

        let settings_form_item = form
            .values
            .get_mut(target.form_value_key)
            .expect(format!("No form row with key: {}", target.form_value_key).as_str());

        settings_form_item.value = value;
    }

    /// Rebuild the configuration from the settings forms and save it, so the applet picks up the change.
    fn update_configuration(&mut self) {
        let form = |id: &window::Id, name: &str| {
            self.settings_forms
                .get(id)
                .unwrap_or_else(|| panic!("No {name} settings form configured."))
        };

        let configuration = AppConfiguration {
            general: self
                .configuration
                .general
                .update(form(&GENERAL_SETTINGS_WINDOW_ID, "general"), self.font_size_slot),
            memory: self.configuration.memory.update(form(&MEMORY_SETTINGS_WINDOW_ID, "memory")),
            cpu: self.configuration.cpu.update(form(&CPU_SETTINGS_WINDOW_ID, "cpu")),
            gpu: self.configuration.gpu.update(form(&GPU_SETTINGS_WINDOW_ID, "gpu")),
            network: self.configuration.network.update(form(&NETWORK_SETTINGS_WINDOW_ID, "network")),
            disk: self.configuration.disk.update(form(&DISK_SETTINGS_WINDOW_ID, "disk")),
            storage: self.configuration.storage.update(form(&STORAGE_SETTINGS_WINDOW_ID, "storage")),
        };

        if configuration == self.configuration {
            return;
        }

        info!("Updating configuration: {:?}", configuration);
        self.configuration = configuration;
        self.save_configuration();
    }

    fn save_configuration(&self) {
        info!("Saving configuration");

        match Config::new(GLOBAL_APP_ID, AppConfiguration::VERSION) {
            Ok(helper) => {
                if let Err(err) = self.configuration.write_entry(&helper) {
                    error!("Failed to save configuration: {}", err);
                }
            }
            Err(err) => error!("Failed to create config context for saving: {}", err),
        }
    }
}

fn load_configuration() -> AppConfiguration {
    Config::new(GLOBAL_APP_ID, AppConfiguration::VERSION)
        .map(|context| {
            AppConfiguration::get_entry(&context).unwrap_or_else(|(errors, config)| {
                error!("Errors while loading configuration: {:?}", errors);
                config
            })
        })
        .unwrap_or_default()
}

/// Build the sidebar, with one entry per settings form. The entry for `active_form_id` is selected, or the first
/// entry when there's no such form.
fn build_settings_nav(settings_forms: &BTreeMap<window::Id, SettingsForm>, active_form_id: window::Id) -> nav_bar::Model {
    let mut model = nav_bar::Model::default();

    let active_form_id = if settings_forms.contains_key(&active_form_id) {
        Some(active_form_id)
    } else {
        settings_forms.keys().next().copied()
    };
    for (form_id, form) in settings_forms {
        let entity = model.insert().text(form.title.clone()).data(*form_id);
        if Some(*form_id) == active_form_id {
            entity.activate();
        }
    }

    model
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_page_round_trips_through_arg() {
        for page in SettingsPage::ALL {
            assert_eq!(SettingsPage::from_arg(&page.to_arg()), Some(page));
        }
    }

    #[test]
    fn font_size_slot_args_are_not_pages() {
        assert_eq!(SettingsPage::from_arg("horizontal-M"), None);
        assert_eq!(SettingsPage::from_arg("cpu"), None);
    }
}
