use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_icons::{READ_ICON, WRITE_ICON};
use crate::monitors::disk_monitor::DiskStats;
use crate::ui::components::indicator::IndicatorContent;
use crate::ui::indicators::{data_rate_content, DataRate};

impl DiskStats {
    pub fn content(&self, configuration: &AppConfiguration) -> Option<IndicatorContent> {
        let disk = &configuration.disk;
        if disk.hide_indicator {
            return None;
        }

        Some(data_rate_content(
            configuration,
            [
                DataRate {
                    icon_key: READ_ICON,
                    colour: disk.read_colour.as_deref(),
                    bytes_per_second: self.avg_bytes_read,
                },
                DataRate {
                    icon_key: WRITE_ICON,
                    colour: disk.write_colour.as_deref(),
                    bytes_per_second: self.avg_bytes_written,
                },
            ],
        ))
    }
}
