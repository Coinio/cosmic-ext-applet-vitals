use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_icons::STORAGE_ICON;
use crate::monitors::storage_monitor::StorageStats;
use crate::ui::components::indicator::{IndicatorContent, IndicatorIcon, IndicatorLine};

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

    #[test]
    fn formats_iec_storage() {
        let config = config(true);

        assert_eq!(format_storage_bytes(10 * 1024 * 1024 * 1024, &config), "10.0GiB");
        assert_eq!(format_storage_bytes(2 * 1024 * 1024 * 1024 * 1024, &config), "2.0TiB");
    }
}
