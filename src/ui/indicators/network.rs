use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_icons::{DOWN_ARROW_ICON, UP_ARROW_ICON};
use crate::monitors::network_monitor::NetworkStats;
use crate::ui::components::indicator::IndicatorContent;
use crate::ui::indicators::{data_rate_content, DataRate};

impl NetworkStats {
    pub fn content(&self, configuration: &AppConfiguration) -> Option<IndicatorContent> {
        let network = &configuration.network;
        if network.hide_indicator {
            return None;
        }

        Some(data_rate_content(
            configuration,
            [
                DataRate {
                    icon_key: DOWN_ARROW_ICON,
                    colour: network.download_colour.as_deref(),
                    bytes_per_second: self.rx_bytes,
                },
                DataRate {
                    icon_key: UP_ARROW_ICON,
                    colour: network.upload_colour.as_deref(),
                    bytes_per_second: self.tx_bytes,
                },
            ],
        ))
    }
}
