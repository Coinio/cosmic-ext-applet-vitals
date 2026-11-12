use crate::app::Message;
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_icons::{AppIcons, DOWN_ARROW_ICON, UP_ARROW_ICON};
use crate::fl;
use crate::monitors::network_monitor::{InterfaceStats, NetworkStats};
use crate::ui::components::detail_popup::{detail_popup, ROW_SPACING};
use crate::ui::components::indicator::IndicatorContent;
use crate::ui::indicators::{data_rate_content, detail_rates, DataRate};
use cosmic::iced::widget::{Column, Row};
use cosmic::iced::{Alignment, Length};
use cosmic::{widget, Element};

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

    /// The popup shown when hovering over the indicator, which breaks its totals down by interface.
    pub fn detail<'a>(&'a self, configuration: &'a AppConfiguration, app_icons: &'a AppIcons) -> Element<'a, Message> {
        let sections: Vec<Element<'a, Message>> = if self.interfaces.is_empty() {
            vec![widget::text::body(fl!("popup-network-none")).into()]
        } else {
            self.interfaces
                .iter()
                .map(|interface| interface_detail(interface, configuration, app_icons))
                .collect()
        };

        detail_popup(fl!("popup-network-title"), sections)
    }
}

/// The interface's name and IPv4 address, above its download and upload rates. A disconnected interface only says
/// that it's disconnected.
fn interface_detail<'a>(
    interface: &'a InterfaceStats,
    configuration: &'a AppConfiguration,
    app_icons: &'a AppIcons,
) -> Element<'a, Message> {
    let network = &configuration.network;

    let caption = if interface.is_up {
        interface.ipv4.map(|address| address.to_string())
    } else {
        Some(fl!("popup-network-disconnected"))
    };

    let heading = Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(widget::text::body(interface.name.as_str()).width(Length::Fill))
        .push_maybe(caption.map(widget::text::caption));

    let rates = interface.is_up.then(|| {
        detail_rates(
            configuration,
            app_icons,
            [
                DataRate {
                    icon_key: DOWN_ARROW_ICON,
                    colour: network.download_colour.as_deref(),
                    bytes_per_second: interface.rx_bytes,
                },
                DataRate {
                    icon_key: UP_ARROW_ICON,
                    colour: network.upload_colour.as_deref(),
                    bytes_per_second: interface.tx_bytes,
                },
            ],
        )
    });

    Column::new().spacing(ROW_SPACING).push(heading).push_maybe(rates).into()
}
