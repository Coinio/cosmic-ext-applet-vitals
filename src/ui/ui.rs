use crate::app::{AppState, Message};
use crate::core::app_configuration::ConfigValue::MemoryUpdateInterval;
use crate::core::app_configuration::SENSOR_INTERVAL_MINIMUM_IN_MS;
use crate::ui::display_item::DisplayItem;
use cosmic::iced::alignment::Vertical;
use cosmic::iced_widget::{container, Container};
use cosmic::widget::{button, row, settings, Button};
use cosmic::{widget, Element, Theme};
use std::cmp;
use std::time::Duration;

pub struct Ui;

impl Ui {
    pub fn build_indicator<'a>(app_state: &'a AppState, display_item: &'a impl DisplayItem)
        -> Button<'a, Message> {

        let core = app_state.core();
        let configuration = app_state.configuration();

        let padding = core.applet.suggested_padding(false);

        let label_container = container(
            core.applet.text(display_item.label(&configuration))
                .class(cosmic::theme::Text::Custom(|theme| {
                    let mut c: cosmic::iced_core::Color = theme.current_container().on.into();
                    c.a *= 0.5;
                    cosmic::iced_widget::text::Style { color: Some(c) }
                }))
                .font(cosmic::iced::Font {
                    weight: cosmic::iced::font::Weight::Medium,
                    ..Default::default()
                })
        ).padding([0, padding]);

        let text_container = container(
            core.applet.text(display_item.text(&configuration))
                .font(cosmic::iced::Font {
                    weight: cosmic::iced::font::Weight::Bold,
                    ..Default::default()
                })
        ).padding([0, padding]);

        let content = vec![
            Element::new(label_container),
            Element::new(text_container),
        ];

        button::custom(Element::from(
            row::with_children(content).align_y(Vertical::Center),
        )).on_press(Message::TogglePopup(display_item.settings_window_id()))
            .class(cosmic::theme::Button::AppletMenu)
    }

    pub fn build_memory_settings_view(app_state: &AppState) -> Container<Message, Theme> {

        let configuration = app_state.configuration();
        let title = "Memory Settings";

        let current_interval = configuration.memory.update_interval.as_millis().to_string();

        let container = container(widget::list_column()
            .padding(5)
            .spacing(0)
            .add(widget::text(title))
            .add(settings::item("Update Interval (ms)",
                widget::text_input("Enter a value in ms...", current_interval)
                    .on_input(|new_interval| Self::handle_interval_change(new_interval,
                                                                          configuration.memory
                                                                              .update_interval))))
        );

        container
    }
    
    pub fn build_cpu_settings_view(app_state: &AppState) -> Container<Message,Theme> {

        let title = "CPU Settings";

        let content_list = container(widget::list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                title,
                widget::toggler(false).on_toggle(Message::ToggleExampleRow),
            )));

        content_list
    }

    fn handle_interval_change(new_interval: String, previous_interval: Duration) -> Message {
        let interval = new_interval.trim();

        let parsed_interval = match interval.parse() {
            Ok(value) => cmp::max(value, SENSOR_INTERVAL_MINIMUM_IN_MS),
            Err(_) => return Message::ConfigValueUpdated(MemoryUpdateInterval(previous_interval))
        };

        Message::ConfigValueUpdated(MemoryUpdateInterval(Duration::from_millis(parsed_interval)))
    }

}