use crate::app::{AppState, Message};
use crate::configuration::app_configuration::AppConfiguration;
use crate::monitors::disk_monitor::DiskStats;
use crate::ui::app_colours::{BRIGHT_GREEN, BRIGHT_RED};
use crate::ui::app_icons::{DOWN_ARROW_ICON, UP_ARROW_ICON};
use crate::ui::components::indicator::{indicator, IndicatorProps, IndicatorValueItem};
use crate::ui::components::svg_icon::SvgIconProps;
use cosmic::iced::{Alignment, Color};
use cosmic::Element;

impl DiskStats {
    pub fn draw<'app>(&self, app_state: &'app AppState, horizontal: bool) -> Option<Element<'app, Message>> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        if self.is_hidden(configuration) {
            return None;
        }

        let text = self.label(app_state);

        let display_item_color = self.label_colour(app_state);
        let font_size = app_state.font_size(horizontal);
        let icon_size = app_state.icon_size();

        let max_text_width = app_state
            .app_text_measurements()
            .measure(self.max_label_text(configuration), font_size)
            .unwrap_or(0.0);

        let mut values: Vec<IndicatorValueItem> = Vec::new();
        values.push(IndicatorValueItem {
            text: self.read_value(app_state.configuration()),
            icon: if horizontal {
                Some(SvgIconProps {
                    icon: app_state.app_icons().get(DOWN_ARROW_ICON),
                    size: icon_size,
                    colour: app_state
                        .app_colours()
                        .get(BRIGHT_GREEN)
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
                        .get(BRIGHT_RED)
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
                values,
            },
        )
            }

    fn label(&self, app_state: &AppState) -> Option<String> {
        Some("DISK".to_string())
    }

    fn label_colour(&self, app_state: &AppState) -> Color {
        app_state
            .configuration()
            .disk
            .label_colour
            .as_deref()
            .and_then(|key| app_state.app_colours().get(key))
            .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha))
    }

    fn read_value(&self, _app_config: &AppConfiguration) -> String {
        let mb_per_second = self.avg_bytes_read as f64 / 1_000_000.0;
        if mb_per_second > 999.9 {
            let gb_per_second = self.avg_bytes_read as f64 / 1_000_000_000.0;
            format!("{:.1}GB/s", gb_per_second)
        } else if mb_per_second > 99.9 {
            format!("{:.0}MB/s", mb_per_second.round())
        } else {
            format!("{:.1}MB/s", mb_per_second)
        }
    }

    fn write_value(&self, _app_config: &AppConfiguration) -> String {
        let mb_per_second = self.avg_bytes_written as f64 / 1_000_000.0;
        if mb_per_second > 999.9 {
            let gb_per_second = self.avg_bytes_written as f64 / 1_000_000_000.0;
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
        app_config.disk.hide_indicator
    }
}
