use crate::app::Message;
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_icons::AppIcons;
use crate::ui::components::indicator::{IndicatorContent, IndicatorIcon, IndicatorLine};
use crate::ui::components::svg_icon::{svg_icon, SvgIconProps};
use cosmic::iced::widget::Row;
use cosmic::iced::{Alignment, Length};
use cosmic::{widget, Element};

/// The size of the direction icons before the data rates in a detail popup.
const DETAIL_RATE_ICON_SIZE: u16 = 12;

pub mod disk;
pub mod network;
pub mod cpu;
pub mod memory;
pub mod storage;

fn format_bytes_per_second(bytes_per_sec: u64, app_config: &AppConfiguration) -> String {
    if app_config.general.use_iec_units {
        let mib_per_second = bytes_per_sec as f64 / 1024.0 / 1024.0;
        if mib_per_second > 999.9 {
            let gib_per_second = bytes_per_sec as f64 / 1024.0 / 1024.0 / 1024.0;
            format!("{:.1}GiB/s", gib_per_second)
        } else if mib_per_second > 99.9 {
            format!("{:.0}MiB/s", mib_per_second.round())
        } else {
            format!("{:.1}MiB/s", mib_per_second)
        }
    } else {
        let mb_per_second = bytes_per_sec as f64 / 1_000_000.0;
        if mb_per_second > 999.9 {
            let gb_per_second = bytes_per_sec as f64 / 1_000_000_000.0;
            format!("{:.1}GB/s", gb_per_second)
        } else if mb_per_second > 99.9 {
            format!("{:.0}MB/s", mb_per_second.round())
        } else {
            format!("{:.1}MB/s", mb_per_second)
        }
    }
}
/// Bytes as gigabytes, or terabytes once there are a thousand gigabytes, in at most 4 significant figures.
fn format_size_bytes(bytes: u64, app_config: &AppConfiguration) -> String {
    let (base, giga, tera) = if app_config.general.use_iec_units {
        (1024.0_f64, "GiB", "TiB")
    } else {
        (1000.0_f64, "GB", "TB")
    };

    let gigabytes = bytes as f64 / base.powi(3);
    // Rounded first, so a value that rounds up to 1000 moves to the next unit instead of showing as "1000GB".
    let (value, unit) = if gigabytes.round() >= 1000.0 {
        (gigabytes / base, tera)
    } else {
        (gigabytes, giga)
    };

    if value > 99.9 {
        format!("{:.0}{unit}", value.round())
    } else {
        format!("{value:.1}{unit}")
    }
}

/// One direction of a data rate indicator, e.g. download.
struct DataRate<'a> {
    icon_key: &'static str,
    colour: Option<&'a str>,
    bytes_per_second: u64,
}

/// Both directions of a data rate in a detail popup, side by side, each with the indicator's icon and colour. Each
/// takes half the width, so the rates of every interface or disk line up.
fn detail_rates<'a>(app_config: &AppConfiguration, app_icons: &'a AppIcons, rates: [DataRate; 2]) -> Element<'a, Message> {
    let rates = rates.into_iter().map(|rate| {
        let icon = IndicatorIcon::new(rate.icon_key, rate.colour);
        Row::new()
            .spacing(4)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .push_maybe(svg_icon(SvgIconProps {
                icon: app_icons.get(icon.key),
                size: DETAIL_RATE_ICON_SIZE,
                colour: icon.colour,
            }))
            .push(widget::text::body(format_bytes_per_second(rate.bytes_per_second, app_config)))
            .into()
    });

    Row::with_children(rates).spacing(8).into()
}

/// A data rate indicator, such as network or disk, with a line per direction.
fn data_rate_content(app_config: &AppConfiguration, rates: [DataRate; 2]) -> IndicatorContent {
    IndicatorContent {
        icon: None,
        lines: rates
            .into_iter()
            .map(|rate| IndicatorLine {
                icon: Some(IndicatorIcon::new(rate.icon_key, rate.colour)),
                text: format_bytes_per_second(rate.bytes_per_second, app_config),
            })
            .collect(),
        max_label_text: if app_config.general.use_iec_units { "99.9MiB/s" } else { "99.9MB/s" },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(use_iec_units: bool) -> AppConfiguration {
        let mut config = AppConfiguration::default();
        config.general.use_iec_units = use_iec_units;
        config
    }

    #[test]
    fn formats_si_sizes() {
        let config = config(false);

        assert_eq!(format_size_bytes(0, &config), "0.0GB");
        assert_eq!(format_size_bytes(12_340_000_000, &config), "12.3GB");
        assert_eq!(format_size_bytes(412_000_000_000, &config), "412GB");
        assert_eq!(format_size_bytes(999_700_000_000, &config), "1.0TB");
        assert_eq!(format_size_bytes(2_000_000_000_000, &config), "2.0TB");
        assert_eq!(format_size_bytes(120_000_000_000_000, &config), "120TB");
    }

    #[test]
    fn formats_iec_sizes() {
        let config = config(true);

        assert_eq!(format_size_bytes(10 * 1024 * 1024 * 1024, &config), "10.0GiB");
        assert_eq!(format_size_bytes(2 * 1024 * 1024 * 1024 * 1024, &config), "2.0TiB");
    }
}
