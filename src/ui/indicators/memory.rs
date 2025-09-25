use crate::app::{AppState, Message};
use crate::configuration::app_configuration::AppConfiguration;
use crate::monitors::memory_monitor::MemoryStats;
use crate::ui::components::indicator::{indicator, IndicatorProps, IndicatorValueItem};
use cosmic::iced::Color;
use cosmic::Element;

impl MemoryStats {
    pub fn draw<'app>(&self, app_state: &'app AppState, horizontal: bool) -> Option<Element<'app, Message>> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        if self.is_hidden(configuration) {
            return None;
        }

        let text = configuration.memory.label_text.clone();

        let display_item_color = self.label_colour(app_state);
        let font_size = app_state.font_size(horizontal);

        let max_text_width = app_state
            .app_text_measurements()
            .measure(self.max_label_text(configuration), font_size)
            .unwrap_or(0.0);

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
                spacing: app_state.core().applet.suggested_padding(false),
                values,
            },
        )
    }

    fn label(&self, app_state: &AppState) -> Option<String> {
        Some("MEM".to_string())
    }

    fn label_colour(&self, app_state: &AppState) -> Color {
        app_state
            .configuration()
            .memory
            .label_colour
            .as_deref()
            .and_then(|key| app_state.app_colours().get(key))
            .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha))
    }

    fn value(&self, _app_config: &AppConfiguration) -> String {
        let used_gb = self.used_kib as f64 * 1024.0 / 1_000_000_000.0;
        if used_gb > 99.9 {
            format!("{:.0}GB", used_gb.round())
        } else {
            format!("{:.1}GB", used_gb)
        }
    }

    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str {
        "99.9GB"
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.memory.hide_indicator
    }
}
