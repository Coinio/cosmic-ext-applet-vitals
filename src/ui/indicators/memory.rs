use crate::app::{AppState, Message};
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_icons::MEMORY_ICON;
use crate::monitors::memory_monitor::MemoryStats;
use crate::ui::components::indicator::{indicator, IndicatorProps, IndicatorValueItem};
use cosmic::Element;
use crate::ui::indicators::{text_line_metrics, themed_icon};

impl MemoryStats {
    pub fn draw<'app>(&self, app_state: &'app AppState, horizontal: bool) -> Option<Element<'app, Message>> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        if self.is_hidden(configuration) {
            return None;
        }

        let font_size = app_state.font_size(horizontal);
        let (line_height, digit_centre) = text_line_metrics(app_state, font_size);
        let icon = if configuration.memory.hide_icon {
            None
        } else {
            Some(themed_icon(app_state, MEMORY_ICON, configuration.memory.icon_colour.as_deref(), font_size))
        };

        let max_text_width = if configuration.general.fix_indicator_size {
            app_state
                .app_text_measurements()
                .measure(self.max_label_text(configuration), font_size, app_state.indicator_font())
        } else {
            None
        };

        let values = vec![IndicatorValueItem {
            text: self.value(app_state.configuration()),
            icon: None,
        }];

        indicator(
            core,
            IndicatorProps {
                icon,
                font_size,
                font: app_state.indicator_font(),
                line_height,
                digit_centre,
                value_width: max_text_width,
                horizontal,
                spacing: app_state.core().applet.suggested_padding(true),
                icon_spacing: 2,
                values,
            },
        )
    }

    fn value(&self, app_config: &AppConfiguration) -> String {
        if app_config.general.use_iec_units {
            let used_gib = self.used_kib as f64 / 1024.0 / 1024.0;
            if used_gib > 99.9 {
                format!("{:.0}GiB", used_gib.round())
            } else {
                format!("{:.1}GiB", used_gib)
            }
        } else {
            let used_gb = self.used_kib as f64 * 1024.0 / 1_000_000_000.0;
            if used_gb > 99.9 {
                format!("{:.0}GB", used_gb.round())
            } else {
                format!("{:.1}GB", used_gb)
            }
        }
    }

    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str {
        if app_config.general.use_iec_units {
            "99.9MiB"
        } else {
            "99.9MB"
        }
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.memory.hide_indicator
    }
}
