use crate::app::{AppState, Message};
use crate::configuration::app_configuration::AppConfiguration;
use crate::monitors::network_monitor::NetworkStats;
use crate::ui::app_colours::{BRIGHT_GREEN, BRIGHT_RED};
use crate::ui::app_icons::{DOWN_ARROW_ICON, UP_ARROW_ICON};
use crate::ui::components::indicator_label::{indicator_label, IndicatorLabelProps};
use crate::ui::components::indicator_value::{indicator_value, IndicatorValueProps};
use crate::ui::components::svg_icon::{svg_icon, SvgIconProps};
use cosmic::iced::{Alignment, Color};
use cosmic::iced_widget::Row;
use cosmic::widget::Column;
use cosmic::Element;

impl NetworkStats {
    pub fn draw<'app>(&self, app_state: &'app AppState, horizontal: bool) -> Option<Element<'app, Message>> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        if self.is_hidden(configuration) {
            return None;
        }

        let text = self.label(app_state);

        let display_item_color = self.label_colour(app_state);
        let font_size = app_state.font_size();

        let label = indicator_label(
            core,
            IndicatorLabelProps {
                text,
                font_size,
                colour: display_item_color,
            },
        );

        let max_text_width = app_state
            .app_text_measurements()
            .measure(self.max_label_text(configuration), font_size)
            .unwrap_or(0.0);

        let rx_value = indicator_value(
            core,
            IndicatorValueProps {
                text: self.read_value(app_state.configuration()),
                font_size,
                width: max_text_width,
                horizontal,
            },
        );

        let rx_icon = if horizontal {
            svg_icon(SvgIconProps {
                icon: app_state.app_icons().get(DOWN_ARROW_ICON),
                size: font_size,
                colour: app_state
                    .app_colours()
                    .get(BRIGHT_GREEN)
                    .map(|c| Color::new(c.red, c.green, c.blue, c.alpha)),
            })
        } else {
            None
        };

        let tx_value = indicator_value(
            core,
            IndicatorValueProps {
                text: self.write_value(app_state.configuration()),
                font_size,
                width: max_text_width,
                horizontal,
            },
        );

        let tx_icon = if horizontal {
            svg_icon(SvgIconProps {
                icon: app_state.app_icons().get(UP_ARROW_ICON),
                size: font_size,
                colour: app_state
                    .app_colours()
                    .get(BRIGHT_RED)
                    .map(|c| Color::new(c.red, c.green, c.blue, c.alpha)),
            })
        } else {
            None
        };

        let mut content: Vec<Element<Message>> = Vec::new();

        if let Some(label) = label {
            content.push(label);
        }
        let mut read_container = Row::new().spacing(3);

        if let Some(value) = rx_value {
            read_container = read_container.push(value);
        }
        if let Some(icon) = rx_icon {
            read_container = read_container.push(icon);
        }

        let suggested_padding = app_state.core().applet.suggested_padding(false);

        if (horizontal) {
            content.push(read_container.align_y(Alignment::Center).padding(suggested_padding)
                .into());
        } else {
            content.push(read_container.into());
        }

        let mut write_container = Row::new().spacing(3);

        if let Some(value) = tx_value {
            write_container = write_container.push(value);
        }
        if let Some(icon) = tx_icon {
            write_container = write_container.push(icon);
        }

        if (horizontal) {
            content.push(write_container.align_y(Alignment::Center).padding(suggested_padding)
                .into());
        } else {
            content.push(write_container.into());
        }

        let row: Element<Message> = if horizontal {
            Row::from_vec(content)
                .spacing(app_state.core().applet.suggested_padding(false))
                .align_y(Alignment::Center)
                .into()
        } else {
            Column::from_vec(content).align_x(Alignment::Center).into()
        };

        Some(row)
    }

    fn label(&self, app_state: &AppState) -> Option<String> {
        Some("NET".to_string())
    }

    fn label_colour(&self, app_state: &AppState) -> Color {
        app_state
            .configuration()
            .network
            .label_colour_rx
            .as_deref()
            .and_then(|key| app_state.app_colours().get(key))
            .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha))
    }

    fn read_value(&self, _app_config: &AppConfiguration) -> String {
        let mb_per_second = self.rx_bytes as f64 / 1_000_000.0;
        if mb_per_second > 999.9 {
            let gb_per_second = self.rx_bytes as f64 / 1_000_000_000.0;
            format!("{:.1}GB/s", gb_per_second)
        } else if mb_per_second > 99.9 {
            format!("{:.0}MB/s", mb_per_second.round())
        } else {
            format!("{:.1}MB/s", mb_per_second)
        }
    }

    fn write_value(&self, _app_config: &AppConfiguration) -> String {
        let mb_per_second = self.tx_bytes as f64 / 1_000_000.0;
        if mb_per_second > 999.9 {
            let gb_per_second = self.tx_bytes as f64 / 1_000_000_000.0;
            format!("{:.1}GB/s", gb_per_second)
        } else if mb_per_second > 99.9 {
            format!("{:.0}MB/s", mb_per_second.round())
        } else {
            format!("{:.1}MB/s", mb_per_second)
        }
    }

    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str {
        "99.9MB/s"
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.network.hide_indicator
    }
}
