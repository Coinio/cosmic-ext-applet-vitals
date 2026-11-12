use crate::configuration::app_configuration::{
    GPU_DEVICE_SETTING_KEY, GPU_SETTINGS_WINDOW_ID, GPU_SHOW_VRAM_SETTING_KEY, HIDE_ICON_SETTING_KEY,
    HIDE_INDICATOR_SETTING_KEY, ICON_COLOUR_SETTING_KEY, MAX_SAMPLES_SETTING_KEY, UPDATE_INTERVAL_SETTING_KEY,
};
use crate::configuration::validation::ConfigurationValidation;
use crate::core::app_colours::ACCENT_PINK;
use crate::core::settings::SettingsForm;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// The configuration for the GPU monitor
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
// Fields added in later versions are missing from older saved configurations, so they take their default.
#[serde(default)]
pub struct GpuConfiguration {
    /// Whether to hide the GPU indicator from the panel
    pub hide_indicator: bool,
    /// Whether to hide only the icon for this indicator
    pub hide_icon: bool,
    /// The duration between each update interval, i.e. 5 seconds
    pub update_interval: Duration,
    /// The number of samples to keep and average for the final result
    pub max_samples: usize,
    /// The indicator icon colour key
    pub icon_colour: Option<String>,
    /// The PCI address of the GPU shown in the panel, e.g. `0000:01:00.0`, or `None` to pick the discrete GPU
    /// automatically.
    pub device: Option<String>,
    /// Whether the panel shows the VRAM used after the usage.
    pub show_vram: bool,
}

impl Default for GpuConfiguration {
    fn default() -> Self {
        GpuConfiguration {
            hide_indicator: false,
            hide_icon: false,
            update_interval: Duration::from_secs(1),
            max_samples: 2,
            icon_colour: Some(ACCENT_PINK.to_string()),
            device: None,
            show_vram: true,
        }
    }
}

impl GpuConfiguration {
    pub fn update(&self, settings_form: &SettingsForm) -> Self {
        if settings_form.settings_window_id != GPU_SETTINGS_WINDOW_ID.clone() {
            panic!("Attempted to update GPU settings from a non-gpu settings window.")
        }

        GpuConfiguration {
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
            device: device_from_form_value(&form_value(settings_form, GPU_DEVICE_SETTING_KEY)),
            show_vram: ConfigurationValidation::sanitise_boolean_input(
                form_value(settings_form, GPU_SHOW_VRAM_SETTING_KEY),
                self.show_vram,
            ),
        }
    }
}

/// The settings form value of the panel's GPU, which is empty to pick it automatically.
pub fn device_to_form_value(device: Option<&str>) -> String {
    device.unwrap_or_default().to_string()
}

pub fn device_from_form_value(value: &str) -> Option<String> {
    Some(value.trim()).filter(|value| !value.is_empty()).map(str::to_string)
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
    use crate::configuration::app_configuration::AppConfiguration;

    #[test]
    fn a_configuration_saved_before_gpus_were_shown_gets_the_default() {
        let config: AppConfiguration = AppConfiguration::default();
        assert_eq!(config.gpu, GpuConfiguration::default());

        let config: GpuConfiguration =
            ron::from_str("(hide_indicator: true)").expect("A partial configuration should load");
        assert!(config.hide_indicator);
        assert!(config.show_vram);
        assert_eq!(config.device, None);
    }

    #[test]
    fn the_device_round_trips_through_the_saved_configuration() {
        let config = GpuConfiguration {
            device: Some("0000:01:00.0".to_string()),
            ..GpuConfiguration::default()
        };
        let saved = ron::to_string(&config).unwrap();
        assert_eq!(ron::from_str::<GpuConfiguration>(&saved).unwrap(), config);
    }

    #[test]
    fn update_reads_the_gpu_settings_from_the_form() {
        let configuration = GpuConfiguration::default();
        let mut form = SettingsForm::from(&configuration);

        assert_eq!(form.values.get(GPU_DEVICE_SETTING_KEY).unwrap().value, "");
        form.values.get_mut(GPU_DEVICE_SETTING_KEY).unwrap().value = "0000:10:00.0".to_string();
        form.values.get_mut(GPU_SHOW_VRAM_SETTING_KEY).unwrap().value = "false".to_string();

        let updated = configuration.update(&form);

        assert_eq!(
            updated,
            GpuConfiguration {
                device: Some("0000:10:00.0".to_string()),
                show_vram: false,
                ..configuration
            }
        );
    }

    #[test]
    fn an_empty_device_is_automatic() {
        assert_eq!(device_from_form_value(""), None);
        assert_eq!(device_from_form_value(&device_to_form_value(None)), None);
        assert_eq!(
            device_from_form_value(&device_to_form_value(Some("0000:01:00.0"))).as_deref(),
            Some("0000:01:00.0")
        );
    }
}
