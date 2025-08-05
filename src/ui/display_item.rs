use cosmic::widget::icon;
use crate::sensors::proc_meminfo_reader::MemoryInfo;

/// This trait defines what will display for each resource, i.e. CPU, RAM, etc, on the panel
pub trait DisplayItem
{
    fn text(&self) -> String;
    fn icon(&self) -> cosmic::widget::Icon;
}

impl DisplayItem for MemoryInfo {
    fn text(&self) -> String {
        let used_gb = self.used_kibibytes as f64 * 1024.0 / 1_000_000_000.0;
        let total_gb = self.total_kibibytes as f64 * 1024.0 / 1_000_000_000.0;
        
        format!("{:.1}GB/{:.1}GB", used_gb, total_gb)
    }

    fn icon(&self) -> cosmic::widget::Icon {
        icon::from_name("display-symbolic").icon()
    }
}