use crate::app::{AppState, Message};
use crate::configuration::app_configuration::AppConfiguration;
use crate::monitors::cpu_monitor::CpuStats;
use crate::ui::components::indicator::{indicator, IndicatorProps, IndicatorValueItem};
use cosmic::iced::Color;
use cosmic::Element;

impl CpuStats {

    pub fn draw<'app>(&self, app_state: &'app AppState, horizontal: bool) -> Option<Element<'app, Message>> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        if self.is_hidden(configuration) {
            return None;
        }

        let text = if configuration.cpu.hide_label { None } else { configuration.cpu.label_text.clone() };

        let display_item_color = self.label_colour(app_state);
        let font_size = app_state.font_size(horizontal);

        let max_text_width = if configuration.general.fix_indicator_size {
            app_state
                .app_text_measurements()
                .measure(self.max_label_text(), font_size)
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
                label_text: text,
                label_colour: display_item_color,
                font_size,
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
            .cpu
            .label_colour
            .as_deref()
            .and_then(|key| app_state.app_colours().get(key))
            .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha))
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
