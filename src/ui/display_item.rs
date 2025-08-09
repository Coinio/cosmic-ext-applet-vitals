use crate::app::AppState;
use crate::monitors::cpu_monitor::CpuStats;
use crate::monitors::memory_monitor::MemoryStats;

/// This trait defines what will display for each resource, i.e. CPU, RAM, etc, on the panel
pub trait DisplayItem
{
    fn label(&self, app_state: &AppState) -> String;
    fn label_color(&self, app_state: &AppState) -> cosmic::iced_core::Color;
    fn text(&self, app_state: &AppState) -> String;
}

impl DisplayItem for MemoryStats {
    fn label(&self, app_state: &AppState) -> String {
        "RAM".to_string()
    }

    fn label_color(&self, app_state: &AppState) -> cosmic::iced_core::Color {
        cosmic::iced_core::Color::from_rgb8(0x15, 0xAC, 0x64)
    }

    fn text(&self, app_state: &AppState) -> String {
        let used_gb = self.used_kibibytes as f64 * 1024.0 / 1_000_000_000.0;
        
        format!("{:.1}GB", used_gb)
    }
}

impl DisplayItem for CpuStats {
    fn label(&self, app_state: &AppState) -> String {
        "CPU".to_string()
    }

    fn label_color(&self, app_state: &AppState) -> cosmic::iced_core::Color {
        cosmic::iced_core::Color::from_rgb8(0x02, 0x9B, 0xAC)        
    }

    fn text(&self, app_state: &AppState) -> String {
        format!("{:.1}%", self.cpu_usage_percent)
    }

}