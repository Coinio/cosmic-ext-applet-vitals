use crate::configuration::app_configuration::*;
use crate::configuration::validation::ConfigurationValidation;
use crate::core::app_colours::{ACCENT_GREEN, ACCENT_RED};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::core::settings::SettingsForm;

/// The configuration for the network monitor
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
// Fields added in later versions are missing from older saved configurations, so they take their default.
#[serde(default)]
pub struct NetworkConfiguration {
    /// Whether to hide the CPU indicator from the panel
    pub hide_indicator: bool,
    /// The duration between each update interval, i.e. 5 seconds
    pub update_interval: Duration,
    /// The number of samples to keep and average for the final result
    pub max_samples: usize,
    /// The download icon colour
    pub download_colour: Option<String>,
    /// The upload icon colour
    pub upload_colour: Option<String>,
}

impl Default for NetworkConfiguration {
    fn default() -> Self {
        NetworkConfiguration {
            hide_indicator: false,
            update_interval: Duration::from_secs(1),
            max_samples: 4,
            download_colour: Some(ACCENT_GREEN.to_string()),
            upload_colour: Some(ACCENT_RED.to_string()),
        }
    }
}

impl NetworkConfiguration {
    pub fn update(&self, settings_form: &SettingsForm) -> Self {
        if settings_form.settings_window_id != NETWORK_SETTINGS_WINDOW_ID.clone() {
            panic!("Attempted to update network settings from a non-network settings window.")
        }

        NetworkConfiguration {
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
            download_colour: Some(form_value(settings_form, DOWNLOAD_COLOUR_SETTING_KEY)),
            upload_colour: Some(form_value(settings_form, UPLOAD_COLOUR_SETTING_KEY)),
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
            max_samples: 6,
        )";

        let config: NetworkConfiguration = ron::from_str(saved).expect("Older configuration should still load");

        assert!(config.hide_indicator);
        assert_eq!(config.update_interval, Duration::from_secs(2));
        assert_eq!(config.max_samples, 6);
        assert_eq!(config.download_colour, Some(ACCENT_GREEN.to_string()));
        assert_eq!(config.upload_colour, Some(ACCENT_RED.to_string()));
    }
}
