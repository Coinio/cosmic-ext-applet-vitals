use crate::app::Message;
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_icons::{AppIcons, DOWN_ARROW_ICON, UP_ARROW_ICON};
use crate::fl;
use crate::monitors::network_monitor::{InterfaceStats, NetworkStats};
use crate::ui::components::detail_popup::{card, detail_popup, heading_row, popup_header, PopupStyle, ROW_SPACING};
use crate::ui::components::indicator::IndicatorContent;
use crate::ui::indicators::{data_rate_content, detail_rates, DataRate};
use cosmic::iced::widget::Column;
use cosmic::Element;

impl NetworkStats {
    pub fn content(&self, configuration: &AppConfiguration) -> Option<IndicatorContent> {
        let network = &configuration.network;
        if network.hide_indicator {
            return None;
        }

        Some(data_rate_content(configuration, rates(configuration, self.rx_bytes, self.tx_bytes)))
    }

    /// The popup shown when hovering over the indicator, which breaks its totals down by interface.
    pub fn detail<'a>(
        &'a self,
        configuration: &'a AppConfiguration,
        app_icons: &'a AppIcons,
        style: PopupStyle,
    ) -> Element<'a, Message> {
        let cards: Vec<Element<'a, Message>> = if self.interfaces.is_empty() {
            vec![card(style.caption(fl!("popup-network-none")))]
        } else {
            self.interfaces
                .iter()
                .map(|interface| card(interface_detail(style, interface, configuration, app_icons)))
                .collect()
        };

        let totals = detail_rates(
            style,
            configuration,
            app_icons,
            rates(configuration, self.rx_bytes, self.tx_bytes),
            true,
        );
        let header = popup_header(style, None, None, fl!("popup-network-title"), Some(totals));

        detail_popup(header, cards)
    }
}

/// The download and upload rates, in the configured colours.
fn rates(configuration: &AppConfiguration, rx_bytes: u64, tx_bytes: u64) -> [DataRate<'_>; 2] {
    let network = &configuration.network;
    [
        DataRate {
            icon_key: DOWN_ARROW_ICON,
            colour: network.download_colour.as_deref(),
            bytes_per_second: rx_bytes,
        },
        DataRate {
            icon_key: UP_ARROW_ICON,
            colour: network.upload_colour.as_deref(),
            bytes_per_second: tx_bytes,
        },
    ]
}

/// The interface's name and IPv4 address, above its download and upload rates. A disconnected interface only says
/// that it's disconnected.
fn interface_detail<'a>(
    style: PopupStyle,
    interface: &'a InterfaceStats,
    configuration: &'a AppConfiguration,
    app_icons: &'a AppIcons,
) -> Element<'a, Message> {
    let caption = if interface.is_up {
        interface.ipv4.map(|address| address.to_string())
    } else {
        Some(fl!("popup-network-disconnected"))
    };

    let rates = interface.is_up.then(|| {
        detail_rates(
            style,
            configuration,
            app_icons,
            rates(configuration, interface.rx_bytes, interface.tx_bytes),
            false,
        )
    });

    Column::new()
        .spacing(ROW_SPACING)
        .push(heading_row(style, interface.name.as_str(), caption))
        .push_maybe(rates)
        .into()
}
