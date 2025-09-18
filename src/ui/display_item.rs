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
    fn label_icon(&self, app_state: &AppState) -> Option<&Handle>;
    fn label_icon_color(&self, app_state: &AppState) -> Color;
    fn text(&self, app_config: &AppConfiguration) -> String;
    fn is_hidden(&self, app_config: &AppConfiguration) -> bool;
}

impl DisplayItem for MemoryStats {
    fn label_icon(&self, app_state: &AppState) -> Option<&Handle> {
        let is_dark = app_state.core().system_theme().theme_type.is_dark();
        if is_dark {
            APP_ICONS.get(MEMORY_USAGE_ICON_DARK_KEY)
        } else {
            APP_ICONS.get(MEMORY_USAGE_ICON_LIGHT_KEY)
        }
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

    fn text(&self, _app_config: &AppConfiguration) -> String {
        let used_gb = self.used_kib as f64 * 1024.0 / 1_000_000_000.0;

        format!("{:.1}GB", used_gb)
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.memory.hide_indicator
    }
}

impl DisplayItem for CpuStats {
    fn label_icon(&self, app_state: &AppState) -> Option<&Handle> {
        let is_dark = app_state.core().system_theme().theme_type.is_dark();
        if is_dark {
            APP_ICONS.get(CPU_USAGE_ICON_DARK_KEY)
        } else {
            APP_ICONS.get(CPU_USAGE_ICON_LIGHT_KEY)
        }
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

    fn text(&self, _app_config: &AppConfiguration) -> String {
        format!("{:.1}%", self.cpu_usage_percent)
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.cpu.hide_indicator
    }
}

impl DisplayItem for NetworkStats {
    fn label_icon(&self, app_state: &AppState) -> Option<&Handle> {
        let is_dark = app_state.core().system_theme().theme_type.is_dark();

        match self.direction {
            NetworkDirection::Download => {
                if is_dark {
                    APP_ICONS.get(NETWORK_RX_USAGE_ICON_DARK_KEY)
                } else {
                    APP_ICONS.get(NETWORK_RX_USAGE_ICON_LIGHT_KEY)
                }
            }
            NetworkDirection::Upload => {
                if is_dark {
                    APP_ICONS.get(NETWORK_TX_USAGE_ICON_DARK_KEY)
                } else {
                    APP_ICONS.get(NETWORK_TX_USAGE_ICON_LIGHT_KEY)
                }
            }
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

    fn text(&self, _app_config: &AppConfiguration) -> String {
        let mib = self.bytes as f64 / (1024.0 * 1024.0);
        format!("{:.1}MiB/s", mib)
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.network.hide_indicator
    }
}

impl DisplayItem for DiskStats {
    fn label_icon(&self, app_state: &AppState) -> Option<&Handle> {
        let is_dark = app_state.core().system_theme().theme_type.is_dark();
        match self.direction {
            DiskDirection::Read if is_dark => APP_ICONS.get(DISK_READ_ICON_DARK_KEY),
            DiskDirection::Read => APP_ICONS.get(DISK_READ_ICON_LIGHT_KEY),
            DiskDirection::Write if is_dark => APP_ICONS.get(DISK_WRITE_ICON_DARK_KEY),
            DiskDirection::Write => APP_ICONS.get(DISK_WRITE_ICON_LIGHT_KEY),
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

    fn text(&self, _app_config: &AppConfiguration) -> String {
        let mib_per_second = self.bytes as f64 / (1024.0 * 1024.0);
        format!("{:.1}MiB/s", mib_per_second)
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.disk.hide_indicator
    }
}
