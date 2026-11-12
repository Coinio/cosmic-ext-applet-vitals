use crate::app::Message;
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_icons::{AppIcons, READ_ICON, WRITE_ICON};
use crate::fl;
use crate::monitors::disk_monitor::{DiskDeviceStats, DiskStats};
use crate::ui::components::detail_popup::{coloured_bar_class, detail_popup, ROW_SPACING};
use crate::ui::components::indicator::IndicatorContent;
use crate::ui::indicators::{data_rate_content, detail_rates, DataRate};
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::widget::{progress_bar, Column, Row};
use cosmic::iced::{Alignment, Length};
use cosmic::{widget, Element};

/// The thickness of each disk's busy bar.
const BUSY_BAR_GIRTH: u16 = 6;
/// The width of the percentage after the busy bar, which fits "100%".
const BUSY_PERCENT_WIDTH: f32 = 34.0;

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

    /// The popup shown when hovering over the indicator, which breaks its totals down by disk.
    pub fn detail<'a>(&'a self, configuration: &'a AppConfiguration, app_icons: &'a AppIcons) -> Element<'a, Message> {
        let sections: Vec<Element<'a, Message>> = if self.devices.is_empty() {
            vec![widget::text::body(fl!("popup-disk-none")).into()]
        } else {
            self.devices
                .iter()
                .map(|device| device_detail(device, configuration, app_icons))
                .collect()
        };

        detail_popup(fl!("popup-disk-title"), sections)
    }
}

/// The disk's name and model, and whether it's mounted, above its read and write rates and how busy it is.
fn device_detail<'a>(
    device: &'a DiskDeviceStats,
    configuration: &'a AppConfiguration,
    app_icons: &'a AppIcons,
) -> Element<'a, Message> {
    let disk = &configuration.disk;

    let caption = match (device.model.as_deref(), device.is_mounted) {
        (model, true) => model.map(str::to_string),
        (None, false) => Some(fl!("popup-disk-unmounted")),
        (Some(model), false) => Some(format!("{model} · {}", fl!("popup-disk-unmounted"))),
    };

    let heading = Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(widget::text::body(device.name.as_str()).width(Length::Fill))
        .push_maybe(caption.map(widget::text::caption));

    let rates = detail_rates(
        configuration,
        app_icons,
        [
            DataRate {
                icon_key: READ_ICON,
                colour: disk.read_colour.as_deref(),
                bytes_per_second: device.bytes_read,
            },
            DataRate {
                icon_key: WRITE_ICON,
                colour: disk.write_colour.as_deref(),
                bytes_per_second: device.bytes_written,
            },
        ],
    );

    Column::new()
        .spacing(ROW_SPACING)
        .push(heading)
        .push(rates)
        .push(busy_bar(device.busy_percent))
        .into()
}

/// How busy the disk is, as a bar and a percentage. A disk can be fully busy at a low data rate, e.g. with many small
/// reads, so this shows when it's the bottleneck.
fn busy_bar<'a>(busy_percent: f64) -> Element<'a, Message> {
    let busy_percent = busy_percent.clamp(0.0, 100.0);

    Row::new()
        .spacing(6)
        .align_y(Alignment::Center)
        .push(widget::text::caption(fl!("popup-disk-busy")))
        .push(
            progress_bar(0.0..=1.0, (busy_percent / 100.0) as f32)
                .girth(BUSY_BAR_GIRTH)
                .class(coloured_bar_class(None)),
        )
        .push(
            widget::text::caption(format!("{busy_percent:.0}%"))
                .width(Length::Fixed(BUSY_PERCENT_WIDTH))
                .align_x(Horizontal::Right),
        )
        .into()
}
