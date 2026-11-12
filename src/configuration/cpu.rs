use crate::configuration::app_configuration::{
    CPU_COMBINE_CORES_SETTING_KEY, CPU_FAN_SENSOR_SETTING_KEY, CPU_SETTINGS_WINDOW_ID,
    CPU_TEMPERATURE_SENSOR_SETTING_KEY, CPU_USE_FAHRENHEIT_SETTING_KEY, HIDE_ICON_SETTING_KEY,
    HIDE_INDICATOR_SETTING_KEY, ICON_COLOUR_SETTING_KEY, MAX_SAMPLES_SETTING_KEY, UPDATE_INTERVAL_SETTING_KEY,
};
use crate::configuration::validation::ConfigurationValidation;
use crate::core::app_colours::ACCENT_BLUE;
use crate::core::settings::SettingsForm;
use crate::sensors::hwmon_reader::HwmonSensorId;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// The configuration for the CPU monitor
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
// Fields added in later versions are missing from older saved configurations, so they take their default.
#[serde(default)]
pub struct CpuConfiguration {
    /// Whether to hide the CPU indicator from the panel
    pub hide_indicator: bool,
    /// Whether to hide only the icon for this indicator
    pub hide_icon: bool,
    /// The duration between each update interval, i.e. 5 seconds
    pub update_interval: Duration,
    /// The number of samples to keep and average for the final result
    pub max_samples: usize,
    /// The indicator icon colour key
    pub icon_colour: Option<String>,
    /// Whether the detail popup combines the threads of each physical core, instead of showing each thread.
    pub combine_cores: bool,
    /// The temperature sensor shown in the detail popup, or `None` to pick the CPU's own sensor automatically.
    pub temperature_sensor: Option<HwmonSensorId>,
    /// The fan sensor shown in the detail popup, or `None` to pick the CPU fan automatically.
    pub fan_sensor: Option<HwmonSensorId>,
    /// Whether temperatures are shown in Fahrenheit instead of Celsius.
    pub use_fahrenheit: bool,
}

impl Default for CpuConfiguration {
    fn default() -> Self {
        CpuConfiguration {
            update_interval: Duration::from_secs(1),
            max_samples: 4,
            hide_indicator: false,
            hide_icon: false,
            icon_colour: Some(ACCENT_BLUE.to_string()),
            combine_cores: false,
            temperature_sensor: None,
            fan_sensor: None,
            use_fahrenheit: false,
        }
    }
}

impl CpuConfiguration {
    pub fn update(&self, settings_form: &SettingsForm) -> Self {
        if settings_form.settings_window_id != CPU_SETTINGS_WINDOW_ID.clone() {
            panic!("Attempted to update CPU settings from a non-cpu settings window.")
        }

        CpuConfiguration {
            hide_indicator: ConfigurationValidation::sanitise_boolean_input(
                form_value(settings_form, HIDE_INDICATOR_SETTING_KEY),
                self.hide_indicator,
            ),
            hide_icon: ConfigurationValidation::sanitise_boolean_input(
                form_value(settings_form, HIDE_ICON_SETTING_KEY),
                self.hide_icon,
            ),
            update_interval: ConfigurationValidation::sanitise_interval_input(
                form_value(settings_form, UPDATE_INTERVAL_SETTING_KEY),
                self.update_interval,
            ),
            max_samples: ConfigurationValidation::sanitise_max_samples(
                form_value(settings_form, MAX_SAMPLES_SETTING_KEY),
                self.max_samples,
            ),
            icon_colour: Some(form_value(settings_form, ICON_COLOUR_SETTING_KEY)),
            combine_cores: ConfigurationValidation::sanitise_boolean_input(
                form_value(settings_form, CPU_COMBINE_CORES_SETTING_KEY),
                self.combine_cores,
            ),
            temperature_sensor: HwmonSensorId::from_form_value(&form_value(
                settings_form,
                CPU_TEMPERATURE_SENSOR_SETTING_KEY,
            )),
            fan_sensor: HwmonSensorId::from_form_value(&form_value(settings_form, CPU_FAN_SENSOR_SETTING_KEY)),
            use_fahrenheit: ConfigurationValidation::sanitise_boolean_input(
                form_value(settings_form, CPU_USE_FAHRENHEIT_SETTING_KEY),
                self.use_fahrenheit,
            ),
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
    fn loads_a_configuration_saved_before_the_detail_popup() {
        let config: CpuConfiguration = ron::from_str(
            "(hide_indicator: false, hide_icon: true, update_interval: (secs: 2, nanos: 0), max_samples: 3, \
             icon_colour: Some(\"accent-blue\"))",
        )
        .expect("Configuration from before the detail popup should load");

        assert!(config.hide_icon);
        assert_eq!(config.update_interval, Duration::from_secs(2));
        assert_eq!(config.max_samples, 3);
        assert!(!config.combine_cores);
        assert_eq!(config.temperature_sensor, None);
        assert_eq!(config.fan_sensor, None);
        assert!(!config.use_fahrenheit);
    }

    #[test]
    fn sensors_round_trip_through_the_saved_configuration() {
        let config = CpuConfiguration {
            fan_sensor: Some(HwmonSensorId {
                chip: "nct6775".to_string(),
                device: "nct6775.656".to_string(),
                channel: "fan2".to_string(),
            }),
            ..CpuConfiguration::default()
        };

        let saved = ron::to_string(&config).unwrap();
        assert_eq!(ron::from_str::<CpuConfiguration>(&saved).unwrap(), config);
    }

    #[test]
    fn update_reads_the_detail_settings_from_the_form() {
        let configuration = CpuConfiguration::default();
        let mut form = SettingsForm::from(&configuration);

        let fan = HwmonSensorId {
            chip: "nct6683".to_string(),
            device: "nct6683.2592".to_string(),
            channel: "fan1".to_string(),
        };
        form.values.get_mut(CPU_COMBINE_CORES_SETTING_KEY).unwrap().value = "true".to_string();
        form.values.get_mut(CPU_FAN_SENSOR_SETTING_KEY).unwrap().value = HwmonSensorId::to_form_value(Some(&fan));
        form.values.get_mut(CPU_USE_FAHRENHEIT_SETTING_KEY).unwrap().value = "true".to_string();

        let updated = configuration.update(&form);

        assert_eq!(
            updated,
            CpuConfiguration {
                combine_cores: true,
                fan_sensor: Some(fan),
                use_fahrenheit: true,
                ..configuration
            }
        );
    }
}
