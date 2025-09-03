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
use cosmic::iced::Color;
use cosmic::widget::icon::Handle;
use crate::ui::icons::*;

/// This trait defines what will display for each resource, i.e. CPU, RAM, etc, on the panel
pub trait DisplayItem {
    fn settings_window_id(&self) -> cosmic::iced::window::Id;
    fn label_icon(&self, app_state: &AppState) -> Option<&Handle>;
    fn label(&self, app_config: &AppConfiguration) -> String;
    fn text_color(&self, app_state: &AppState) -> cosmic::iced_core::Color;
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

    fn label(&self, app_config: &AppConfiguration) -> String {
        app_config.memory.label_text.clone()
    }

    fn text_color(&self, app_state: &AppState) -> cosmic::iced_core::Color {
        let is_dark = app_state.core().system_theme().theme_type.is_dark();
        // RAM (indigo)
        // Light: #3C4074, Dark: #C7C9F5
        match is_dark {
            true => cosmic::iced_core::Color::from_rgba(0xC7 as f32 / 255.0, 0xC9 as f32 / 255.0, 0xF5 as f32 / 255.0, 1.0),
            false => cosmic::iced_core::Color::from_rgba(0x3C as f32 / 255.0, 0x40 as f32 / 255.0, 0x74 as f32 / 255.0, 1.0),
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

    fn label(&self, app_config: &AppConfiguration) -> String {
        app_config.cpu.label_text.to_string()
    }

    fn label_icon(&self, app_state: &AppState) -> Option<&Handle> {
        let is_dark = app_state.core().system_theme().theme_type.is_dark();
        if is_dark {
            ICONS.get(CPU_USAGE_ICON_DARK_KEY)
        } else {
            ICONS.get(CPU_USAGE_ICON_LIGHT_KEY)
        }
    }

    fn text_color(&self, app_state: &AppState) -> cosmic::iced_core::Color {
        let is_dark = app_state.core().system_theme().theme_type.is_dark();
        // CPU (amber)
        // Light: #5A4223, Dark: #F0C38F
        match is_dark {
            true => cosmic::iced_core::Color::from_rgba(0xF0 as f32 / 255.0, 0xC3 as f32 / 255.0, 0x8F as f32 / 255.0, 1.0),
            false => cosmic::iced_core::Color::from_rgba(0x5A as f32 / 255.0, 0x42 as f32 / 255.0, 0x23 as f32 / 255.0, 1.0),
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

    fn label(&self, app_config: &AppConfiguration) -> String {
        match self.direction {
            NetworkDirection::Download => app_config.network.rx_label_text.clone(),
            NetworkDirection::Upload => app_config.network.tx_label_text.clone(),
        }
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

    fn text_color(&self, app_state: &AppState) -> Color {
        let is_dark = app_state.core().system_theme().theme_type.is_dark();
        match self.direction {
            // Network Download (red): Light #6A2E2A, Dark #F0A5A3
            NetworkDirection::Download if is_dark => cosmic::iced_core::Color::from_rgba(0xF0 as f32 / 255.0, 0xA5 as f32 / 255.0, 0xA3 as f32 / 255.0, 1.0),
            NetworkDirection::Download => cosmic::iced_core::Color::from_rgba(0x6A as f32 / 255.0, 0x2E as f32 / 255.0, 0x2A as f32 / 255.0, 1.0),
            // Network Upload (green): Light #2E4A36, Dark #A3D7AE
            NetworkDirection::Upload if is_dark => cosmic::iced_core::Color::from_rgba(0xA3 as f32 / 255.0, 0xD7 as f32 / 255.0, 0xAE as f32 / 255.0, 1.0),
            NetworkDirection::Upload => cosmic::iced_core::Color::from_rgba(0x2E as f32 / 255.0, 0x4A as f32 / 255.0, 0x36 as f32 / 255.0, 1.0),
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

    fn label(&self, app_config: &AppConfiguration) -> String {
        match self.direction {
            DiskDirection::Read => app_config.disk.read_label_text.clone(),
            DiskDirection::Write => app_config.disk.write_label_text.clone(),
        }
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

    fn text_color(&self, app_state: &AppState) -> Color {
        let is_dark = app_state.core().system_theme().theme_type.is_dark();
        match self.direction {
            // Disk Read (Download - teal): Light #2C5456, Dark #A9E1E3
            DiskDirection::Read if is_dark => cosmic::iced_core::Color::from_rgba(0xA9 as f32 / 255.0, 0xE1 as f32 / 255.0, 0xE3 as f32 / 255.0, 1.0),
            DiskDirection::Read => cosmic::iced_core::Color::from_rgba(0x2C as f32 / 255.0, 0x54 as f32 / 255.0, 0x56 as f32 / 255.0, 1.0),
            // Disk Write (Upload - blue): Light #2E447A, Dark #AFC8F2
            DiskDirection::Write if is_dark => cosmic::iced_core::Color::from_rgba(0xAF as f32 / 255.0, 0xC8 as f32 / 255.0, 0xF2 as f32 / 255.0, 1.0),
            DiskDirection::Write => cosmic::iced_core::Color::from_rgba(0x2E as f32 / 255.0, 0x44 as f32 / 255.0, 0x7A as f32 / 255.0, 1.0),
        }
    }

    fn text(&self, app_config: &AppConfiguration) -> String {
        let mib_per_second = self.bytes as f64 / (1024.0 * 1024.0);
        format!("{:.1}MiB/s", mib_per_second)
    }
}
