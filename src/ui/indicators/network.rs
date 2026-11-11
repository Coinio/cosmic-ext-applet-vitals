use crate::app::{AppState, Message};
use crate::configuration::app_configuration::AppConfiguration;
use crate::monitors::network_monitor::NetworkStats;
use crate::core::app_icons::{DOWN_ARROW_ICON, UP_ARROW_ICON};
use crate::ui::components::indicator::{indicator, IndicatorProps, IndicatorValueItem};
use cosmic::Element;
use crate::ui::indicators::{configured_colour, format_bytes_per_second, text_line_metrics, themed_icon};

impl NetworkStats {
    pub fn draw<'app>(&self, app_state: &'app AppState, horizontal: bool) -> Option<Element<'app, Message>> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        if self.is_hidden(configuration) {
            return None;
        }

        let font_size = app_state.font_size(horizontal);
        let (line_height, digit_centre) = text_line_metrics(app_state, font_size);

        let max_text_width = if configuration.general.fix_indicator_size {
            app_state
                .app_text_measurements()
                .measure(self.max_label_text(configuration), font_size, app_state.indicator_font())
        } else {
            None
        };

        let mut values: Vec<IndicatorValueItem> = Vec::new();
        values.push(IndicatorValueItem {
            text: self.read_value(app_state.configuration()),
            icon: if horizontal {
                Some(themed_icon(app_state, DOWN_ARROW_ICON, configured_colour(configuration.network.download_colour.as_deref()), font_size))
            } else {
                None
            },
        });

        values.push(IndicatorValueItem {
            text: self.write_value(app_state.configuration()),
            icon: if horizontal {
                Some(themed_icon(app_state, UP_ARROW_ICON, configured_colour(configuration.network.upload_colour.as_deref()), font_size))
            } else {
                None
            },
        });

        indicator(
            core,
            IndicatorProps {
                icon: None,
                font_size,
                font: app_state.indicator_font(),
                line_height,
                digit_centre,
                value_width: max_text_width,
                horizontal,
                spacing: app_state.core().applet.suggested_padding(true).1,
                icon_spacing: 2,
                values,
            },
        )
    }
    
    fn read_value(&self, app_config: &AppConfiguration) -> String {
        format_bytes_per_second(self.rx_bytes, app_config)
    }

    fn write_value(&self, app_config: &AppConfiguration) -> String {
        format_bytes_per_second(self.tx_bytes, app_config)
    }

    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str {
        if app_config.general.use_iec_units {
            "99.9MiB/s"
        } else {
            "99.9MB/s"
        }
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.network.hide_indicator
    }
}
