use crate::app::AppState;
use crate::configuration::app_configuration::AppConfiguration;
use crate::monitors::cpu_monitor::CpuStats;
use crate::monitors::memory_monitor::MemoryStats;
use crate::monitors::network_monitor::{NetworkDirection, NetworkStats};
use crate::ui::app_icons::*;
use cosmic::iced::Color;
use cosmic::widget::icon::Handle;

/// This trait defines what will display for each resource, i.e. CPU, RAM, etc, on the panel
pub trait DisplayItem {
    fn label(&self, app_state: &AppState) -> Option<String>;
    fn label_text_colour(&self, app_state: &AppState) -> Color;
    fn value(&self, app_config: &AppConfiguration) -> String;
    fn value_icon(&self, app_state: &AppState) -> Option<Handle>;
    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str;
    fn is_hidden(&self, app_config: &AppConfiguration) -> bool;
}

impl DisplayItem for MemoryStats {
    fn label(&self, app_state: &AppState) -> Option<String> {
        Some("MEM".to_string())
    }

    fn label_text_colour(&self, app_state: &AppState) -> Color {
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

    fn value_icon(&self, app_state: &AppState) -> Option<Handle> {
        None
    }

    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str {
        "99.9GB"
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.memory.hide_indicator
    }
}

impl DisplayItem for CpuStats {
    fn label(&self, app_state: &AppState) -> Option<String> {
        Some("CPU".to_string())
    }

    fn label_text_colour(&self, app_state: &AppState) -> Color {
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

    fn value_icon(&self, app_state: &AppState) -> Option<Handle> {
        None
    }

    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str {
        "99.9%"
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.cpu.hide_indicator
    }
}

impl DisplayItem for NetworkStats {
    fn label(&self, app_state: &AppState) -> Option<String> {
        match self.direction {
            NetworkDirection::Download => Some("RX".to_string()),
            NetworkDirection::Upload => Some("TX".to_string()),
        }
    }

    fn label_text_colour(&self, app_state: &AppState) -> Color {
        match self.direction {
            NetworkDirection::Download => app_state
                .configuration()
                .network
                .label_colour_rx
                .as_deref()
                .and_then(|key| app_state.app_colours().get(key))
                .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha)),
            NetworkDirection::Upload => app_state
                .configuration()
                .network
                .label_colour_tx
                .as_deref()
                .and_then(|key| app_state.app_colours().get(key))
                .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha)),
        }
    }

    fn value(&self, _app_config: &AppConfiguration) -> String {
        let mb = self.bytes as f64 / 1_000_000.0;
        if mb > 999.9
        {
            let gb = self.bytes as f64 / 1_000_000_000.0;
            format!("{:.1}GB/s", gb)
        } else if mb > 99.9 {
            format!("{:.0}MB/s", mb.round())
        } else {
            format!("{:.1}MB/s", mb)
        }
    }

    fn value_icon(&self, app_state: &AppState) -> Option<Handle> {
        match self.direction {
            NetworkDirection::Download => app_state.app_icons().get(DOWN_ARROW_ICON).cloned(),
            NetworkDirection::Upload => app_state.app_icons().get(UP_ARROW_ICON).cloned(),
        }
    }

    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str {
        "99.9MB/s"
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.network.hide_indicator
    }
}

/*impl DisplayItem for DiskStats {
    fn label(&self, app_state: &AppState) -> Option<String> {
        match self.direction {
            DiskDirection::Read => Some("R".to_string()),
            DiskDirection::Write => Some("W".to_string())
        }
    }

    fn label_text_colour(&self, app_state: &AppState) -> Color {
        match self.direction {
            DiskDirection::Read => app_state
                .configuration()
                .disk
                .label_colour_read
                .as_deref()
                .and_then(|key| app_state.app_colours().get(key))
                .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha)),
            DiskDirection::Write => app_state
                .configuration()
                .disk
                .label_colour_write
                .as_deref()
                .and_then(|key| app_state.app_colours().get(key))
                .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha)),
        }
    }

    fn value(&self, _app_config: &AppConfiguration) -> String {
        let mb_per_second = self.bytes as f64 / 1_000_000.0;
        if mb_per_second > 999.9 {
            let gb_per_second = self.bytes as f64 / 1_000_000_000.0;
            format!("{:.1}GB/s", gb_per_second)
        } else if mb_per_second > 99.9 {
            format!("{:.0}MB/s", mb_per_second.round())
        } else {
            format!("{:.1}MB/s", mb_per_second)
        }
    }

    fn value_icon(&self, app_state: &AppState) -> Option<Handle> {
        match self.direction {
            DiskDirection::Read => app_state.app_icons().get(DOWN_ARROW_ICON).cloned(),
            DiskDirection::Write => app_state.app_icons().get(UP_ARROW_ICON).cloned(),
        }
    }

    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str {
        "99.9MB/s"    
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.disk.hide_indicator
    }
}*/
