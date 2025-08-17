use crate::core::app_configuration::{AppConfiguration, CPU_SETTINGS_WINDOW_ID, MEMORY_SETTINGS_WINDOW_ID};
use crate::monitors::cpu_monitor::CpuStats;
use crate::monitors::memory_monitor::MemoryStats;

/// This trait defines what will display for each resource, i.e. CPU, RAM, etc, on the panel
pub trait DisplayItem
{
    fn settings_window_id(&self) -> cosmic::iced::window::Id;
    fn label(&self, app_config: &AppConfiguration) -> String;
    fn label_color(&self, app_config: &AppConfiguration) -> cosmic::iced_core::Color;
    fn text(&self, app_config: &AppConfiguration) -> String;
}

impl DisplayItem for MemoryStats {
    fn settings_window_id(&self) -> cosmic::iced::window::Id {
        MEMORY_SETTINGS_WINDOW_ID.clone()
    }

    fn label(&self, app_config: &AppConfiguration) -> String {
        // TODO: Can we get away from this clone?
        app_config.memory.label_text.clone()
    }

    fn label_color(&self, app_config: &AppConfiguration) -> cosmic::iced_core::Color {
        let hex = app_config.memory.label_colour;
        
        cosmic::iced_core::Color::from_rgba(
            hex.r as f32 / 255.0,
            hex.g as f32 / 255.0,
            hex.b as f32 / 255.0,
            hex.a as f32 / 255.0,
        )
    }

    fn text(&self, app_config: &AppConfiguration) -> String {
        let used_gb = self.used_kibibytes as f64 * 1024.0 / 1_000_000_000.0;
        
        format!("{:.1}GB", used_gb)
    }
}

impl DisplayItem for CpuStats {
    fn settings_window_id(&self) -> cosmic::iced::window::Id {
        CPU_SETTINGS_WINDOW_ID.clone()
    }

    fn label(&self, app_config: &AppConfiguration) -> String {
        // TODO: Can we get away from this clone?
        app_config.cpu.label_text.to_string()
    }

    fn label_color(&self, app_config: &AppConfiguration) -> cosmic::iced_core::Color {
        let hex = app_config.cpu.label_colour;

        cosmic::iced_core::Color::from_rgba(
            hex.r as f32 / 255.0,
            hex.g as f32 / 255.0,
            hex.b as f32 / 255.0,
            hex.a as f32 / 255.0,
        )
    }

    fn text(&self, app_config: &AppConfiguration) -> String {
        format!("{:.1}%", self.cpu_usage_percent)
    }

}