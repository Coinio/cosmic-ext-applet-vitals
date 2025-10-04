use crate::app::{AppState, Message};
use crate::configuration::app_configuration::AppConfiguration;
use crate::monitors::network_monitor::NetworkStats;
use crate::core::app_colours::{ACCENT_GREEN, ACCENT_RED};
use crate::core::app_icons::{DOWN_ARROW_ICON, UP_ARROW_ICON};
use crate::ui::components::indicator::{indicator, IndicatorProps, IndicatorValueItem};
use crate::ui::components::svg_icon::SvgIconProps;
use cosmic::iced::Color;
use cosmic::Element;

impl NetworkStats {
    pub fn draw<'app>(&self, app_state: &'app AppState, horizontal: bool) -> Option<Element<'app, Message>> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        if self.is_hidden(configuration) {
            return None;
        }

        let text = if configuration.network.hide_label { None } else { configuration.network.label_text.clone() };

        let display_item_color = self.label_colour(app_state);
        let font_size = app_state.font_size(horizontal);
        let icon_size = app_state.icon_size();

        let max_text_width = if configuration.general.fix_indicator_size {
            app_state
                .app_text_measurements()
                .measure(self.max_label_text(), font_size)
        } else {
            None
        };

        let mut values: Vec<IndicatorValueItem> = Vec::new();
        values.push(IndicatorValueItem {
            text: self.read_value(app_state.configuration()),
            icon: if horizontal {
                Some(SvgIconProps {
                    icon: app_state.app_icons().get(DOWN_ARROW_ICON),
                    size: icon_size,
                    colour: app_state
                        .app_colours()
                        .get(ACCENT_GREEN)
                        .map(|c| Color::new(c.red, c.green, c.blue, c.alpha)),
                })
            } else {
                None
            },
        });

        values.push(IndicatorValueItem {
            text: self.write_value(app_state.configuration()),
            icon: if horizontal {
                Some(SvgIconProps {
                    icon: app_state.app_icons().get(UP_ARROW_ICON),
                    size: icon_size,
                    colour: app_state
                        .app_colours()
                        .get(ACCENT_RED)
                        .map(|c| Color::new(c.red, c.green, c.blue, c.alpha)),
                })
            } else {
                None
            },
        });

        indicator(
            core,
            IndicatorProps {
                label_text: text,
                label_colour: display_item_color,
                font_size,
                value_width: max_text_width,
                horizontal,
                spacing: app_state.core().applet.suggested_padding(false),
                icon_spacing: 2,
                values,
            },
        )
    }
    
    fn label_colour(&self, app_state: &AppState) -> Color {
        app_state
            .configuration()
            .network
            .label_colour
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

    fn max_label_text(&self) -> &'static str {
        "99.9MB/s"
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.network.hide_indicator
    }
}
