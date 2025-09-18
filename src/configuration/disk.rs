use crate::configuration::app_configuration::{
    DISK_READ_COLOUR_SETTING_KEY, DISK_SETTINGS_WINDOW_ID, DISK_WRITE_COLOUR_SETTING_KEY, HIDE_INDICATOR_SETTING_KEY,
    MAX_SAMPLES_SETTING_KEY, UPDATE_INTERVAL_SETTING_KEY,
};
use crate::configuration::validation::ConfigurationValidation;
use crate::ui::settings_form::SettingsForm;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::ui::app_colours::{ACCENT_GREEN, ACCENT_PINK};

/// The configuration for the memory monitor
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiskConfiguration {
    /// Whether to hide the CPU indicator from the panel
    pub hide_indicator: bool,
    /// The duration between each update interval, i.e. 5 seconds
    pub update_interval: Duration,
    /// The number of samples to keep and average for the final result
    pub max_samples: usize,
    /// The read indicator icon colour key
    pub label_colour_read: Option<String>,
    /// The write indicator icon colour key
    pub label_colour_write: Option<String>,
}

impl Default for DiskConfiguration {
    fn default() -> Self {
        Self {
            hide_indicator: false,
            update_interval: Duration::from_secs(1),
            max_samples: 3,
            label_colour_read: Some(ACCENT_GREEN.to_string()),
            label_colour_write: Some(ACCENT_PINK.to_string()),
        }
    }
}

impl DiskConfiguration {
    pub fn from(&self, settings_form: &SettingsForm) -> Self {
        if settings_form.settings_window_id != DISK_SETTINGS_WINDOW_ID.clone() {
            panic!("Attempted to update disk settings from a non-disk settings window.")
        }

        DiskConfiguration {
            hide_indicator: ConfigurationValidation::sanitise_boolean_input(
                settings_form
                    .values
                    .get(HIDE_INDICATOR_SETTING_KEY)
                    .expect("Hide indicator missing from settings form options")
                    .value
                    .clone(),
                self.hide_indicator,
            ),
            update_interval: ConfigurationValidation::sanitise_interval_input(
                settings_form
                    .values
                    .get(UPDATE_INTERVAL_SETTING_KEY)
                    .expect("Update interval missing from settings form options")
                    .value
                    .clone(),
                self.update_interval,
            ),
            max_samples: ConfigurationValidation::sanitise_max_samples(
                settings_form
                    .values
                    .get(MAX_SAMPLES_SETTING_KEY)
                    .expect("Max samples missing from settings form options")
                    .value
                    .clone(),
                self.max_samples,
            ),
            label_colour_read: Some(
                settings_form
                    .values
                    .get(DISK_READ_COLOUR_SETTING_KEY)
                    .expect("Read colour missing from settings form options")
                    .value
                    .clone(),
            ),
            label_colour_write: Some(
                settings_form
                    .values
                    .get(DISK_WRITE_COLOUR_SETTING_KEY)
                    .expect("Write colour missing from settings form options")
                    .value
                    .clone(),
            ),
        }
    }
}
