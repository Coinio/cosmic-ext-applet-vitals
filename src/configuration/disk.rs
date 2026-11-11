use crate::configuration::app_configuration::{
    DISK_SETTINGS_WINDOW_ID, HIDE_INDICATOR_SETTING_KEY, MAX_SAMPLES_SETTING_KEY, READ_COLOUR_SETTING_KEY,
    UPDATE_INTERVAL_SETTING_KEY, WRITE_COLOUR_SETTING_KEY,
};
use crate::configuration::validation::ConfigurationValidation;
use crate::core::app_colours::{ACCENT_GREEN, ACCENT_RED};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::core::settings::SettingsForm;

/// The configuration for the memory monitor
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
// Fields added in later versions are missing from older saved configurations, so they take their default.
#[serde(default)]
pub struct DiskConfiguration {
    /// Whether to hide the CPU indicator from the panel
    pub hide_indicator: bool,
    /// The duration between each update interval, i.e. 5 seconds
    pub update_interval: Duration,
    /// The number of samples to keep and average for the final result
    pub max_samples: usize,
    /// The read icon colour
    pub read_colour: Option<String>,
    /// The write icon colour
    pub write_colour: Option<String>,
}

impl Default for DiskConfiguration {
    fn default() -> Self {
        Self {
            hide_indicator: false,
            update_interval: Duration::from_secs(1),
            max_samples: 3,
            read_colour: Some(ACCENT_GREEN.to_string()),
            write_colour: Some(ACCENT_RED.to_string()),
        }
    }
}

impl DiskConfiguration {
    pub fn update(&self, settings_form: &SettingsForm) -> Self {
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
            read_colour: Some(form_value(settings_form, READ_COLOUR_SETTING_KEY)),
            write_colour: Some(form_value(settings_form, WRITE_COLOUR_SETTING_KEY)),
        }
    }
}

fn form_value(settings_form: &SettingsForm, key: &str) -> String {
    settings_form
        .values
        .get(key)
        .unwrap_or_else(|| panic!("{key} missing from settings form options"))
        .value
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_configuration_saved_before_colours_were_added() {
        let saved = "(
            hide_indicator: true,
            update_interval: (secs: 2, nanos: 0),
            max_samples: 5,
        )";

        let config: DiskConfiguration = ron::from_str(saved).expect("Older configuration should still load");

        assert!(config.hide_indicator);
        assert_eq!(config.update_interval, Duration::from_secs(2));
        assert_eq!(config.max_samples, 5);
        assert_eq!(config.read_colour, Some(ACCENT_GREEN.to_string()));
        assert_eq!(config.write_colour, Some(ACCENT_RED.to_string()));
    }
}
