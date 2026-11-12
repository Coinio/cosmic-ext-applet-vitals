use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_icons::CPU_ICON;
use crate::monitors::cpu_monitor::CpuStats;
use crate::ui::components::indicator::{IndicatorContent, IndicatorIcon, IndicatorLine};

impl CpuStats {
    pub fn content(&self, configuration: &AppConfiguration) -> Option<IndicatorContent> {
        let cpu = &configuration.cpu;
        if cpu.hide_indicator {
            return None;
        }

        Some(IndicatorContent {
            icon: (!cpu.hide_icon).then(|| IndicatorIcon::new(CPU_ICON, cpu.icon_colour.as_deref())),
            lines: vec![IndicatorLine::text(self.value())],
            max_label_text: "99.9%",
        })
    }

    fn value(&self) -> String {
        if self.cpu_usage_percent >= 100.0 {
            "100%".to_string()
        } else {
            format!("{:.1}%", self.cpu_usage_percent)
        }
    }
}
