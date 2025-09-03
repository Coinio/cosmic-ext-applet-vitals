use crate::app::{AppState};
use crate::configuration::app_configuration::{
    AppConfiguration, CPU_SETTINGS_WINDOW_ID, DISK_SETTINGS_WINDOW_ID, MEMORY_SETTINGS_WINDOW_ID,
    NETWORK_SETTINGS_WINDOW_ID,
};
use crate::monitors::cpu_monitor::CpuStats;
use crate::monitors::disk_monitor::{DiskDirection, DiskStats};
use crate::monitors::memory_monitor::MemoryStats;
use crate::monitors::network_monitor::{NetworkDirection, NetworkStats};
use cosmic::iced::window::Id;
use cosmic::widget::icon::Handle;
use crate::ui::icons::*;

/// This trait defines what will display for each resource, i.e. CPU, RAM, etc, on the panel
pub trait DisplayItem {
    fn settings_window_id(&self) -> cosmic::iced::window::Id;
    fn label_icon(&self, app_state: &AppState) -> Option<&Handle>;
    fn text(&self, app_config: &AppConfiguration) -> String;
}

impl DisplayItem for MemoryStats {
    fn settings_window_id(&self) -> cosmic::iced::window::Id {
        MEMORY_SETTINGS_WINDOW_ID.clone()
    }

    fn label_icon(&self, app_state: &AppState) -> Option<&Handle> {
        let is_dark = app_state.core().system_theme().theme_type.is_dark();
        if is_dark {
            ICONS.get(MEMORY_USAGE_ICON_DARK_KEY)
        } else {
            ICONS.get(MEMORY_USAGE_ICON_LIGHT_KEY)
        }
    }
    
    fn text(&self, app_config: &AppConfiguration) -> String {
        let used_gb = self.used_kib as f64 * 1024.0 / 1_000_000_000.0;

        format!("{:.1}GB", used_gb)
    }
}

impl DisplayItem for CpuStats {
    fn settings_window_id(&self) -> cosmic::iced::window::Id {
        CPU_SETTINGS_WINDOW_ID.clone()
    }

    fn label_icon(&self, app_state: &AppState) -> Option<&Handle> {
        let is_dark = app_state.core().system_theme().theme_type.is_dark();
        if is_dark {
            ICONS.get(CPU_USAGE_ICON_DARK_KEY)
        } else {
            ICONS.get(CPU_USAGE_ICON_LIGHT_KEY)
        }
    }

    fn text(&self, app_config: &AppConfiguration) -> String {
        format!("{:.1}%", self.cpu_usage_percent)
    }
}

impl DisplayItem for NetworkStats {
    fn settings_window_id(&self) -> Id {
        NETWORK_SETTINGS_WINDOW_ID.clone()
    }

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

    fn text(&self, app_config: &AppConfiguration) -> String {
        let mib = self.bytes as f64 / (1024.0 * 1024.0);
        format!("{:.1}MiB/s", mib)
    }
}

impl DisplayItem for DiskStats {
    fn settings_window_id(&self) -> Id {
        DISK_SETTINGS_WINDOW_ID.clone()
    }

    fn label_icon(&self, app_state: &AppState) -> Option<&Handle> {
        let is_dark = app_state.core().system_theme().theme_type.is_dark();
        match self.direction {
            DiskDirection::Read if is_dark => ICONS.get(DISK_READ_ICON_DARK_KEY),
            DiskDirection::Read => ICONS.get(DISK_READ_ICON_LIGHT_KEY),
            DiskDirection::Write if is_dark => ICONS.get(DISK_WRITE_ICON_DARK_KEY),
            DiskDirection::Write => ICONS.get(DISK_WRITE_ICON_LIGHT_KEY),
        }
    }

    fn text(&self, app_config: &AppConfiguration) -> String {
        let mib_per_second = self.bytes as f64 / (1024.0 * 1024.0);
        format!("{:.1}MiB/s", mib_per_second)
    }
}
