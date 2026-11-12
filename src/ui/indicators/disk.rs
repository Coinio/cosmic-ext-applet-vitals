use crate::app::Message;
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_icons::{AppIcons, READ_ICON, WRITE_ICON};
use crate::fl;
use crate::monitors::disk_monitor::{DiskDeviceStats, DiskStats};
use crate::ui::components::detail_popup::{
    card, detail_popup, heading_row, inline_bar, popup_header, PopupStyle, ROW_SPACING,
};
use crate::ui::components::indicator::IndicatorContent;
use crate::ui::indicators::{data_rate_content, detail_rates, DataRate};
use cosmic::iced::widget::Column;
use cosmic::Element;

/// The width of the "BUSY" label before each disk's busy bar.
const BUSY_LABEL_WIDTH: f32 = 34.0;

impl DiskStats {
    pub fn content(&self, configuration: &AppConfiguration) -> Option<IndicatorContent> {
        let disk = &configuration.disk;
        if disk.hide_indicator {
            return None;
        }

        Some(data_rate_content(
            configuration,
            rates(configuration, self.avg_bytes_read, self.avg_bytes_written),
        ))
    }

    /// The popup shown when hovering over the indicator, which breaks its totals down by disk.
    pub fn detail<'a>(
        &'a self,
        configuration: &'a AppConfiguration,
        app_icons: &'a AppIcons,
        style: PopupStyle,
    ) -> Element<'a, Message> {
        let cards: Vec<Element<'a, Message>> = if self.devices.is_empty() {
            vec![card(style.caption(fl!("popup-disk-none")))]
        } else {
            self.devices
                .iter()
                .map(|device| card(device_detail(style, device, configuration, app_icons)))
                .collect()
        };

        let totals = detail_rates(
            style,
            configuration,
            app_icons,
            rates(configuration, self.avg_bytes_read, self.avg_bytes_written),
            true,
        );
        let header = popup_header(style, None, None, fl!("popup-disk-title"), Some(totals));

        detail_popup(header, cards)
    }
}

/// The read and write rates, in the configured colours.
fn rates(configuration: &AppConfiguration, bytes_read: u64, bytes_written: u64) -> [DataRate<'_>; 2] {
    let disk = &configuration.disk;
    [
        DataRate {
            icon_key: READ_ICON,
            colour: disk.read_colour.as_deref(),
            bytes_per_second: bytes_read,
        },
        DataRate {
            icon_key: WRITE_ICON,
            colour: disk.write_colour.as_deref(),
            bytes_per_second: bytes_written,
        },
    ]
}

/// The disk's name and model, and whether it's mounted, above its read and write rates and how busy it is.
fn device_detail<'a>(
    style: PopupStyle,
    device: &'a DiskDeviceStats,
    configuration: &'a AppConfiguration,
    app_icons: &'a AppIcons,
) -> Element<'a, Message> {
    let caption = match (device.model.as_deref(), device.is_mounted) {
        (model, true) => model.map(str::to_string),
        (None, false) => Some(fl!("popup-disk-unmounted")),
        (Some(model), false) => Some(format!("{model} · {}", fl!("popup-disk-unmounted"))),
    };

    let rates = detail_rates(
        style,
        configuration,
        app_icons,
        rates(configuration, device.bytes_read, device.bytes_written),
        false,
    );

    Column::new()
        .spacing(ROW_SPACING)
        .push(heading_row(style, device.name.as_str(), caption))
        .push(rates)
        .push(busy_bar(style, device.busy_percent))
        .into()
}

/// How busy the disk is, as a bar and a percentage. A disk can be fully busy at a low data rate, e.g. with many small
/// reads, so this shows when it's the bottleneck.
fn busy_bar<'a>(style: PopupStyle, busy_percent: f64) -> Element<'a, Message> {
    inline_bar(
        style,
        style.section_heading(fl!("popup-disk-busy")),
        BUSY_LABEL_WIDTH,
        Some(busy_percent),
        (busy_percent.clamp(0.0, 100.0) / 100.0) as f32,
        None,
    )
}
