use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_icons::MEMORY_ICON;
use crate::monitors::memory_monitor::MemoryStats;
use crate::ui::components::indicator::{IndicatorContent, IndicatorIcon, IndicatorLine};

impl MemoryStats {
    pub fn content(&self, configuration: &AppConfiguration) -> Option<IndicatorContent> {
        let memory = &configuration.memory;
        if memory.hide_indicator {
            return None;
        }

        let use_iec_units = configuration.general.use_iec_units;

        Some(IndicatorContent {
            icon: (!memory.hide_icon).then(|| IndicatorIcon::new(MEMORY_ICON, memory.icon_colour.as_deref())),
            lines: vec![IndicatorLine::text(self.value(use_iec_units))],
            max_label_text: if use_iec_units { "99.9GiB" } else { "99.9GB" },
        })
    }

    fn value(&self, use_iec_units: bool) -> String {
        if use_iec_units {
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
}
