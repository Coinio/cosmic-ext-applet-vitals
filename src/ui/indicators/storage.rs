use crate::app::{AppState, Message};
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_icons::STORAGE_ICON;
use crate::monitors::storage_monitor::StorageStats;
use crate::ui::components::indicator::{indicator, IndicatorProps, IndicatorValueItem};
use crate::ui::indicators::{configured_colour, text_line_metrics, themed_icon};
use cosmic::Element;

impl StorageStats {
    pub fn draw<'app>(&self, app_state: &'app AppState, horizontal: bool) -> Option<Element<'app, Message>> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        if self.is_hidden(configuration) {
            return None;
        }

        let font_size = app_state.font_size(horizontal);
        let (line_height, digit_centre) = text_line_metrics(app_state, font_size);
        let icon = if configuration.storage.hide_icon {
            None
        } else {
            Some(themed_icon(
                app_state,
                STORAGE_ICON,
                configured_colour(configuration.storage.icon_colour.as_deref()),
                font_size,
            ))
        };

        let max_text_width = if configuration.general.fix_indicator_size {
            app_state.app_text_measurements().measure(
                self.max_label_text(configuration, horizontal),
                font_size,
                app_state.indicator_font(),
            )
        } else {
            None
        };

        let available = format_storage_bytes(self.available_bytes, configuration);
        let total = format_storage_bytes(self.total_bytes, configuration);

        // There's room for "available/total" on a single line in a horizontal panel, but not in a vertical one.
        let values = if horizontal {
            vec![IndicatorValueItem {
                text: format!("{available}/{total}"),
                icon: None,
            }]
        } else {
            vec![
                IndicatorValueItem {
                    text: available,
                    icon: None,
                },
                IndicatorValueItem { text: total, icon: None },
            ]
        };

        indicator(
            core,
            IndicatorProps {
                icon,
                font_size,
                font: app_state.indicator_font(),
                line_height,
                digit_centre,
                value_width: max_text_width,
                horizontal,
                spacing: app_state.core().applet.suggested_padding(true).1,
                icon_spacing: 2,
                values,
            },
        )
    }

    fn max_label_text(&self, app_config: &AppConfiguration, horizontal: bool) -> &'static str {
        match (app_config.general.use_iec_units, horizontal) {
            (true, true) => "99.9TiB/99.9TiB",
            (true, false) => "99.9TiB",
            (false, true) => "99.9TB/99.9TB",
            (false, false) => "99.9TB",
        }
    }

    fn is_hidden(&self, app_config: &AppConfiguration) -> bool {
        app_config.storage.hide_indicator
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
