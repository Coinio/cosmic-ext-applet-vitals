use crate::app::{AppState, Message};
use crate::configuration::app_configuration::AppConfiguration;
use crate::monitors::disk_monitor::DiskStats;
use crate::core::app_colours::{ACCENT_GREEN, ACCENT_RED};
use crate::core::app_icons::{READ_ICON, WRITE_ICON};
use crate::ui::components::indicator::{indicator, IndicatorProps, IndicatorValueItem};
use crate::ui::components::svg_icon::SvgIconProps;
use cosmic::iced::Color;
use cosmic::Element;
use crate::ui::indicators::format_bytes_per_second;

impl DiskStats {
    pub fn draw<'app>(&self, app_state: &'app AppState, horizontal: bool) -> Option<Element<'app, Message>> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        if self.is_hidden(configuration) {
            return None;
        }

        let text = if configuration.disk.hide_label { None } else { configuration.disk.label_text.clone() };

        let display_item_color = self.label_colour(app_state);
        let font_size = app_state.font_size(horizontal);
        let icon_size = app_state.icon_size();

        let max_text_width = if configuration.general.fix_indicator_size {
            app_state
                .app_text_measurements()
                .measure(self.max_label_text(configuration), font_size)
        } else {
            None
        };

        let mut values: Vec<IndicatorValueItem> = Vec::new();
        values.push(IndicatorValueItem {
            text: self.read_value(app_state.configuration()),
            icon: if horizontal {
                Some(SvgIconProps {
                    icon: app_state.app_icons().get(READ_ICON),
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
                    icon: app_state.app_icons().get(WRITE_ICON),
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
                font: app_state.active_interface_font(),
                value_width: max_text_width,
                horizontal,
                spacing: app_state.core().applet.suggested_padding(true),
                icon_spacing: 2,
                values,
            },
        )
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

    fn read_value(&self, app_config: &AppConfiguration) -> String {
        format_bytes_per_second(self.avg_bytes_read, app_config)
    }

    fn write_value(&self, app_config: &AppConfiguration) -> String {
        format_bytes_per_second(self.avg_bytes_written, app_config)
    }

    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str {
        if app_config.general.use_iec_units {
            "99.9MiB/s"
        } else {
            "99.9MB/s"
        }
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.disk.hide_indicator
    }
}
