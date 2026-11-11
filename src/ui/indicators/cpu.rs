use crate::app::{AppState, Message};
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_icons::CPU_ICON;
use crate::monitors::cpu_monitor::CpuStats;
use crate::ui::components::indicator::{indicator, IndicatorProps, IndicatorValueItem};
use cosmic::Element;
use crate::ui::indicators::{configured_colour, text_line_metrics, themed_icon};

impl CpuStats {

    pub fn draw<'app>(&self, app_state: &'app AppState, horizontal: bool) -> Option<Element<'app, Message>> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        if self.is_hidden(configuration) {
            return None;
        }

        let font_size = app_state.font_size(horizontal);
        let (line_height, digit_centre) = text_line_metrics(app_state, font_size);
        let icon = if configuration.cpu.hide_icon {
            None
        } else {
            Some(themed_icon(app_state, CPU_ICON, configured_colour(configuration.cpu.icon_colour.as_deref()), font_size))
        };

        let max_text_width = if configuration.general.fix_indicator_size {
            app_state
                .app_text_measurements()
                .measure(self.max_label_text(), font_size, app_state.indicator_font())
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
                spacing: app_state.core().applet.suggested_padding(true).1,
                icon_spacing: 2,
                values,
            },
        )
    }

    fn value(&self, _app_config: &AppConfiguration) -> String {
        if self.cpu_usage_percent >= 100.0 {
            "100%".to_string()
        } else {
            format!("{:.1}%", self.cpu_usage_percent)
        }
    }

    fn max_label_text(&self) -> &'static str {
        "99.9%"
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.cpu.hide_indicator
    }
}
