use crate::app::Message;
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_colours::AppColour;
use crate::core::app_icons::STORAGE_ICON;
use crate::fl;
use crate::monitors::storage_monitor::StorageStats;
use crate::sensors::storage_reader::MountUsage;
use crate::ui::components::detail_popup::{coloured_bar_class, detail_popup};
use crate::ui::components::indicator::{IndicatorContent, IndicatorIcon, IndicatorLine};
use cosmic::iced::widget::{progress_bar, Column, Row};
use cosmic::iced::{Alignment, Length};
use cosmic::{widget, Element};

/// The thickness of the used space bar under each mount.
const USED_SPACE_BAR_GIRTH: u16 = 6;

impl StorageStats {
    pub fn content(&self, configuration: &AppConfiguration, horizontal: bool) -> Option<IndicatorContent> {
        let storage = &configuration.storage;
        if storage.hide_indicator {
            return None;
        }

        let available = format_storage_bytes(self.available_bytes, configuration);
        let total = format_storage_bytes(self.total_bytes, configuration);

        // There's room for "available/total" on a single line in a horizontal panel, but not in a vertical one.
        let lines = if horizontal {
            vec![IndicatorLine::text(format!("{available}/{total}"))]
        } else {
            vec![IndicatorLine::text(available), IndicatorLine::text(total)]
        };

        let max_label_text = match (configuration.general.use_iec_units, horizontal) {
            (true, true) => "99.9TiB/99.9TiB",
            (true, false) => "99.9TiB",
            (false, true) => "99.9TB/99.9TB",
            (false, false) => "99.9TB",
        };

        Some(IndicatorContent {
            icon: (!storage.hide_icon).then(|| IndicatorIcon::new(STORAGE_ICON, storage.icon_colour.as_deref())),
            lines,
            max_label_text,
        })
    }

    /// The popup shown when hovering over the indicator, which lists every mount, not only the ones in the totals.
    pub fn detail(&self, configuration: &AppConfiguration) -> Element<'_, Message> {
        let bar_colour = configuration.storage.icon_colour.as_deref().and_then(AppColour::parse);
        let mounts = self
            .mounts
            .iter()
            .map(|mount| mount_detail(mount, bar_colour, configuration));

        detail_popup(fl!("popup-storage-title"), mounts)
    }
}

/// A mount point and its free space, above a bar showing how much of it is used.
fn mount_detail<'a>(
    mount: &'a MountUsage,
    bar_colour: Option<AppColour>,
    configuration: &AppConfiguration,
) -> Element<'a, Message> {
    let space = fl!(
        "popup-storage-space",
        available = format_storage_bytes(mount.available_bytes, configuration),
        total = format_storage_bytes(mount.total_bytes, configuration)
    );

    let label = Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(widget::text::body(mount.mount_point.as_str()).width(Length::Fill))
        .push(widget::text::caption(space));

    let bar = progress_bar(0.0..=1.0, used_fraction(mount))
        .girth(USED_SPACE_BAR_GIRTH)
        .class(coloured_bar_class(bar_colour));

    Column::new().spacing(4).push(label).push(bar).into()
}

/// How much of the mount is used, from 0 to 1. Blocks reserved for root count as used, matching the free space shown.
fn used_fraction(mount: &MountUsage) -> f32 {
    if mount.total_bytes == 0 {
        return 0.0;
    }
    let used_bytes = mount.total_bytes.saturating_sub(mount.available_bytes);
    (used_bytes as f64 / mount.total_bytes as f64) as f32
}

/// Bytes as gigabytes, or terabytes once there are a thousand gigabytes, in at most 4 significant figures.
fn format_storage_bytes(bytes: u64, app_config: &AppConfiguration) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn config(use_iec_units: bool) -> AppConfiguration {
        let mut config = AppConfiguration::default();
        config.general.use_iec_units = use_iec_units;
        config
    }

    #[test]
    fn formats_si_storage() {
        let config = config(false);

        assert_eq!(format_storage_bytes(0, &config), "0.0GB");
        assert_eq!(format_storage_bytes(12_340_000_000, &config), "12.3GB");
        assert_eq!(format_storage_bytes(412_000_000_000, &config), "412GB");
        assert_eq!(format_storage_bytes(999_700_000_000, &config), "1.0TB");
        assert_eq!(format_storage_bytes(2_000_000_000_000, &config), "2.0TB");
        assert_eq!(format_storage_bytes(120_000_000_000_000, &config), "120TB");
    }

    fn mount(total_bytes: u64, available_bytes: u64) -> MountUsage {
        MountUsage {
            source: "/dev/sda1".to_string(),
            mount_point: "/".to_string(),
            total_bytes,
            available_bytes,
        }
    }

    #[test]
    fn used_fraction_is_the_space_that_is_not_available() {
        assert_eq!(used_fraction(&mount(1000, 250)), 0.75);
        assert_eq!(used_fraction(&mount(1000, 1000)), 0.0);
        assert_eq!(used_fraction(&mount(1000, 0)), 1.0);
    }

    #[test]
    fn used_fraction_of_an_empty_filesystem_is_zero() {
        assert_eq!(used_fraction(&mount(0, 0)), 0.0);
    }

    #[test]
    fn formats_iec_storage() {
        let config = config(true);

        assert_eq!(format_storage_bytes(10 * 1024 * 1024 * 1024, &config), "10.0GiB");
        assert_eq!(format_storage_bytes(2 * 1024 * 1024 * 1024 * 1024, &config), "2.0TiB");
    }
}
