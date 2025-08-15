use crate::app::{AppState, Message};
use crate::core::app_configuration::ConfigurationValue::{
    MemoryLabelText, MemoryMaxSamples, MemoryUpdateInterval,
};
use crate::core::app_configuration::{
    SENSOR_INTERVAL_MINIMUM_IN_MS, SENSOR_MAX_LABEL_LENGTH, SENSOR_MAX_SAMPLES_MINIMUM,
};
use crate::fl;
use crate::ui::display_item::DisplayItem;
use cosmic::iced::alignment::Vertical;
use cosmic::iced_widget::{container, Container};
use cosmic::widget::{button, row, settings, Button};
use cosmic::{widget, Element, Theme};
use std::cmp;
use std::time::Duration;

pub struct Ui;

impl Ui {
    pub fn build_indicator<'a>(
        app_state: &'a AppState,
        display_item: &'a impl DisplayItem,
    ) -> Button<'a, Message> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        let padding = core.applet.suggested_padding(false);

        let label_container = container(
            core.applet
                .text(display_item.label(&configuration))
                .class(cosmic::theme::Text::Custom(|theme| {
                    let mut c: cosmic::iced_core::Color = theme.current_container().on.into();
                    c.a *= 0.5;
                    cosmic::iced_widget::text::Style { color: Some(c) }
                }))
                .font(cosmic::iced::Font {
                    weight: cosmic::iced::font::Weight::Medium,
                    ..Default::default()
                }),
        )
        .padding([0, padding]);

        let text_container = container(core.applet.text(display_item.text(&configuration)).font(
            cosmic::iced::Font {
                weight: cosmic::iced::font::Weight::Bold,
                ..Default::default()
            },
        ))
        .padding([0, padding]);

        let content = vec![Element::new(label_container), Element::new(text_container)];

        button::custom(Element::from(
            row::with_children(content).align_y(Vertical::Center),
        ))
        .on_press(Message::TogglePopup(display_item.settings_window_id()))
        .class(cosmic::theme::Button::AppletMenu)
    }

    pub fn build_memory_settings_view(app_state: &AppState) -> Container<Message, Theme> {
        let configuration = app_state.configuration();
        let title = fl!("settings-memory-title");

        let current_interval = configuration.memory.update_interval.as_millis().to_string();

        let container = container(
            widget::list_column()
                .padding(5)
                .spacing(0)
                .add(widget::text(title))
                .add(settings::item(
                    fl!("settings-update-interval"),
                    widget::text_input(fl!("settings-empty"), current_interval).on_input(
                        |new_interval| {
                            let sanitised_interval = Self::sanitise_interval_input(
                                new_interval,
                                configuration.memory.update_interval,
                            );
                            Message::ConfigValueUpdated(MemoryUpdateInterval(sanitised_interval))
                        },
                    ),
                ))
                .add(settings::item(
                    fl!("settings-max-samples"),
                    widget::text_input(
                        fl!("settings-empty"),
                        configuration.memory.max_samples.to_string(),
                    )
                    .on_input(|new_max_samples| {
                        let sanitised_samples = Self::sanitise_max_samples(
                            new_max_samples,
                            configuration.memory.max_samples,
                        );

                        Message::ConfigValueUpdated(MemoryMaxSamples(sanitised_samples))
                    }),
                ))
                .add(settings::item(
                    fl!("settings-label-text"),
                    widget::text_input(
                        fl!("settings-empty"),
                        configuration.memory.label_text.as_str(),
                    )
                    .on_input(|new_label_text| {
                        let sanitised_label_text = Self::sanitise_label_text(new_label_text);

                        Message::ConfigValueUpdated(MemoryLabelText(sanitised_label_text))
                    }),
                )),
        );

        container
    }

    pub fn build_cpu_settings_view(app_state: &AppState) -> Container<Message, Theme> {
        let title = "CPU Settings";

        let content_list = container(widget::list_column().padding(5).spacing(0).add(
            settings::item(
                title,
                widget::toggler(false).on_toggle(Message::ToggleExampleRow),
            ),
        ));

        content_list
    }

    fn sanitise_interval_input(new_input: String, previous_interval: Duration) -> Duration {
        match new_input.trim().parse() {
            Ok(value) => Duration::from_millis(value),
            Err(_) => previous_interval,
        }
    }

    fn sanitise_max_samples(new_input: String, old_value: usize) -> usize {
        match new_input.trim().parse() {
            Ok(value) => cmp::max(value, SENSOR_MAX_SAMPLES_MINIMUM),
            Err(_) => old_value,
        }
    }

    fn sanitise_label_text(new_input: String) -> String {
        if new_input.len() > SENSOR_MAX_LABEL_LENGTH {
            new_input[..SENSOR_MAX_LABEL_LENGTH].to_string()
        } else {
            new_input.to_string()
        }
    }
}
