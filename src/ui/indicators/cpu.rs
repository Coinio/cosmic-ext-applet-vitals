use crate::app::{AppState, Message};
use crate::configuration::app_configuration::AppConfiguration;
use crate::monitors::cpu_monitor::CpuStats;
use crate::ui::components::indicator_label::{indicator_label, IndicatorLabelProps};
use crate::ui::components::indicator_value::{indicator_value, IndicatorValueProps};
use cosmic::iced::{Alignment, Color};
use cosmic::iced_widget::Row;
use cosmic::widget::Column;
use cosmic::Element;

impl CpuStats {

    pub fn draw<'app>(&self, app_state: &'app AppState, horizontal: bool) -> Option<Element<'app, Message>> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        if self.is_hidden(configuration) {
            return None;
        }

        let text = self.label(app_state);

        let display_item_color = self.label_colour(app_state);
        let font_size = app_state.font_size(horizontal);

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

        let value = indicator_value(
            core,
            IndicatorValueProps {
                text: self.value(app_state.configuration()),
                font_size,
                width: max_text_width,
                horizontal,
            },
        );
        let mut content: Vec<Element<Message>> = Vec::new();

        if let Some(label) = label {
            content.push(label);
        }
        if let Some(value) = value {
            content.push(value);
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
        Some("CPU".to_string())
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

    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str {
        "99.9%"
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.cpu.hide_indicator
    }
}
