use cosmic::cosmic_theme::palette::Srgba;
use crate::app::AppState;
use crate::configuration::app_configuration::AppConfiguration;
use crate::monitors::cpu_monitor::CpuStats;
use crate::monitors::disk_monitor::{DiskDirection, DiskStats};
use crate::monitors::memory_monitor::MemoryStats;
use crate::monitors::network_monitor::{NetworkDirection, NetworkStats};
use crate::ui::icons::*;
use cosmic::widget::icon::Handle;

/// This trait defines what will display for each resource, i.e. CPU, RAM, etc, on the panel
pub trait DisplayItem {
    fn label_icon(&self, app_state: &AppState) -> Option<&Handle>;
    fn label_icon_color(&self, app_state: &AppState) -> Srgba;
    fn text(&self, app_config: &AppConfiguration) -> String;
    fn is_hidden(&self, app_config: &AppConfiguration) -> bool;
}

impl DisplayItem for MemoryStats {
    fn label_icon(&self, app_state: &AppState) -> Option<&Handle> {
        let is_dark = app_state.core().system_theme().theme_type.is_dark();
        if is_dark {
            ICONS.get(MEMORY_USAGE_ICON_DARK_KEY)
        } else {
            ICONS.get(MEMORY_USAGE_ICON_LIGHT_KEY)
        }
    }

    fn label_icon_color(&self, app_state: &AppState) -> Srgba {
        app_state.core().system_theme().cosmic().palette.accent_orange
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
            ICONS.get(CPU_USAGE_ICON_DARK_KEY)
        } else {
            ICONS.get(CPU_USAGE_ICON_LIGHT_KEY)
        }
    }

    fn label_icon_color(&self, app_state: &AppState) -> Srgba {
        app_state.core().system_theme().cosmic().palette.accent_indigo
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
                    ICONS.get(NETWORK_RX_USAGE_ICON_DARK_KEY)
                } else {
                    ICONS.get(NETWORK_RX_USAGE_ICON_LIGHT_KEY)
                }
            }
            NetworkDirection::Upload => {
                if is_dark {
                    ICONS.get(NETWORK_TX_USAGE_ICON_DARK_KEY)
                } else {
                    ICONS.get(NETWORK_TX_USAGE_ICON_LIGHT_KEY)
                }
            }
        }
    }

    fn label_icon_color(&self, app_state: &AppState) -> Srgba {
        match self.direction {
            NetworkDirection::Download => app_state.core().system_theme().cosmic().palette
                .accent_green,
            NetworkDirection::Upload => app_state.core().system_theme().cosmic().palette.accent_red
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
            DiskDirection::Read if is_dark => ICONS.get(DISK_READ_ICON_DARK_KEY),
            DiskDirection::Read => ICONS.get(DISK_READ_ICON_LIGHT_KEY),
            DiskDirection::Write if is_dark => ICONS.get(DISK_WRITE_ICON_DARK_KEY),
            DiskDirection::Write => ICONS.get(DISK_WRITE_ICON_LIGHT_KEY),
        }
    }

    fn label_icon_color(&self, app_state: &AppState) -> Srgba {
        match self.direction {
            DiskDirection::Read => app_state.core().system_theme().cosmic().palette
                .accent_purple,
            DiskDirection::Write => app_state.core().system_theme().cosmic().palette.accent_warm_grey
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
