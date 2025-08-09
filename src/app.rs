// SPDX-License-Identifier: GPL-3.0-only

use crate::fl;
use crate::ui::display_item::DisplayItem;
use cosmic::app::{Core, Task};
use cosmic::iced::alignment::Vertical;
use cosmic::iced::window;
use cosmic::iced::Limits;
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::widget::{self, autosize, button, container, row, settings, Button, Id};
use cosmic::{Application, Element};
use once_cell::sync::Lazy;
use tokio_util::sync::CancellationToken;
use crate::monitors::cpu_monitor::{CpuMonitor, CpuStats};
use crate::monitors::memory_monitor::{MemoryMonitor, MemoryStats};
use crate::sensors::proc_meminfo_reader::ProcMemInfoSensorReader;
use crate::sensors::proc_stat_reader::ProcStatSensorReader;

static AUTOSIZE_MAIN_ID: Lazy<Id> = Lazy::new(|| Id::new("autosize-main"));

#[derive(Default)]
pub struct AppState {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The cancellation token to stop the status updates
    monitor_cancellation_token: Option<CancellationToken>,
    /// The current memory usage stats
    memory: MemoryStats,
    /// The current cpu usage stats
    cpu: CpuStats,
    /// The popup id.
    popup: Option<window::Id>,
    /// Example row toggler.
    example_row: bool,
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(window::Id),
    ToggleExampleRow(bool),
    StartMonitoring,
    MemoryUpdate(MemoryStats),
    CpuUpdate(CpuStats),
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
        let app = AppState {
            core,
            ..Default::default()
        };

        (app, cosmic::task::message(Message::StartMonitoring))
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Commands may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = window::Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::ToggleExampleRow(toggled) => self.example_row = toggled,
            Message::StartMonitoring => {
                if let Some(token) = &self.monitor_cancellation_token {
                    token.cancel();
                }

                let cancellation_token = CancellationToken::new();
                self.monitor_cancellation_token = Some(cancellation_token.clone());

                return cosmic::Task::stream(async_stream::stream! {
                    // TODO: Duration from configuration.

                    let mut memory_update_interval = tokio::time::interval
                    (std::time::Duration::from_secs(1));
                    let mut cpu_update_interval = tokio::time::interval(std::time::Duration::from_secs(1));
                                        
                    let mut memory_monitor = MemoryMonitor::new(ProcMemInfoSensorReader,3);
                    let mut cpuinfo_reader = CpuMonitor::new(ProcStatSensorReader, 4);

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
        }
        Task::none()
    }

    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<Self::Message> {
        // TODO: Handle horizontal / vertical layout
        //let horizontal = matches!(self.core.applet.anchor, PanelAnchor::Top |
        // PanelAnchor::Bottom);

        let container = container(cosmic::widget::row()
            .push(self.build_indicator(&self.memory))
            .push(self.build_indicator(&self.cpu)));

        autosize::autosize(container, AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn view_window(&self, _id: window::Id) -> Element<Self::Message> {
        let content_list = widget::list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                fl!("example-row"),
                widget::toggler(self.example_row).on_toggle(Message::ToggleExampleRow),
            ));

        self.core.applet.popup_container(content_list).into()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

impl AppState {
    fn build_indicator(&self, display_item: &impl DisplayItem) -> Button<Message> {
        let padding = self.core.applet.suggested_padding(false);
        let icon_container = container(display_item.icon()).padding(padding);

        let content = vec![
            Element::new(icon_container),
            Element::new(self.core.applet.text(display_item.text())),
        ];

        button::custom(Element::from(
            row::with_children(content).align_y(Vertical::Center),
        ))
            .on_press(Message::TogglePopup)
            .class(cosmic::theme::Button::Standard)
    }
}
