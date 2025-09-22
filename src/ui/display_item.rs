use crate::app::AppState;
use crate::configuration::app_configuration::AppConfiguration;
use crate::monitors::cpu_monitor::CpuStats;
use crate::monitors::disk_monitor::{DiskDirection, DiskStats};
use crate::monitors::memory_monitor::MemoryStats;
use crate::monitors::network_monitor::{NetworkDirection, NetworkStats};
use crate::ui::app_icons::*;
use cosmic::iced::Color;
use cosmic::widget::icon::Handle;

/// This trait defines what will display for each resource, i.e. CPU, RAM, etc, on the panel
pub trait DisplayItem {
    fn label_icon<'app>(&self, app_state: &'app AppState) -> Option<&'app Handle>;
    fn label_icon_color(&self, app_state: &AppState) -> Color;
    fn label(&self, app_config: &AppConfiguration) -> String;
    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str;
    fn is_hidden(&self, app_config: &AppConfiguration) -> bool;
}

impl DisplayItem for MemoryStats {
    fn label_icon<'app>(&self, app_state: &'app AppState) -> Option<&'app Handle> {
        app_state.app_icons().get(MEMORY_USAGE_ICON_DARK_KEY)
    }

    fn label_icon_color(&self, app_state: &AppState) -> Color {
        app_state
            .app_configuration()
            .memory
            .label_colour
            .as_deref()
            .and_then(|key| app_state.app_colours().get(key))
            .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha))
    }

    fn label(&self, _app_config: &AppConfiguration) -> String {
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

impl DisplayItem for CpuStats {
    fn label_icon<'app>(&self, app_state: &'app AppState) -> Option<&'app Handle> {
        app_state.app_icons().get(CPU_USAGE_ICON_DARK_KEY)
    }

    fn label_icon_color(&self, app_state: &AppState) -> Color {
        app_state
            .app_configuration()
            .cpu
            .label_colour
            .as_deref()
            .and_then(|key| app_state.app_colours().get(key))
            .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha))
    }

    fn label(&self, _app_config: &AppConfiguration) -> String {
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

impl DisplayItem for NetworkStats {
    fn label_icon<'app>(&self, app_state: &'app AppState) -> Option<&'app Handle> {
        match self.direction {
            NetworkDirection::Download => app_state.app_icons().get(NETWORK_RX_USAGE_ICON_DARK_KEY),
            NetworkDirection::Upload => app_state.app_icons().get(NETWORK_TX_USAGE_ICON_DARK_KEY),
        }
    }

    fn label_icon_color(&self, app_state: &AppState) -> Color {
        match self.direction {
            NetworkDirection::Download => app_state
                .app_configuration()
                .network
                .label_colour_rx
                .as_deref()
                .and_then(|key| app_state.app_colours().get(key))
                .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha)),
            NetworkDirection::Upload => app_state
                .app_configuration()
                .network
                .label_colour_tx
                .as_deref()
                .and_then(|key| app_state.app_colours().get(key))
                .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha)),
        }
    }

    fn label(&self, _app_config: &AppConfiguration) -> String {
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

    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str {
        "99.9MB/s"
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.network.hide_indicator
    }
}

impl DisplayItem for DiskStats {
    fn label_icon<'app>(&self, app_state: &'app AppState) -> Option<&'app Handle> {
        match self.direction {
            DiskDirection::Read => app_state.app_icons().get(DISK_READ_ICON_DARK_KEY),
            DiskDirection::Write => app_state.app_icons().get(DISK_WRITE_ICON_DARK_KEY),
        }
    }

    fn label_icon_color(&self, app_state: &AppState) -> Color {
        match self.direction {
            DiskDirection::Read => app_state
                .app_configuration()
                .disk
                .label_colour_read
                .as_deref()
                .and_then(|key| app_state.app_colours().get(key))
                .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha)),
            DiskDirection::Write => app_state
                .app_configuration()
                .disk
                .label_colour_write
                .as_deref()
                .and_then(|key| app_state.app_colours().get(key))
                .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha)),
        }
    }

    fn label(&self, _app_config: &AppConfiguration) -> String {
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

    fn max_label_text(&self, app_config: &AppConfiguration) -> &'static str {
        "99.9MB/s"    
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.disk.hide_indicator
    }
}
