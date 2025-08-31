use crate::configuration::app_configuration::{
    DISK_SETTINGS_WINDOW_ID, MAX_SAMPLES_SETTING_KEY, UPDATE_INTERVAL_SETTING_KEY,
};
use crate::configuration::validation::ConfigurationValidation;
use crate::fl;
use crate::ui::settings_form::{SettingsForm, SettingsFormItem};
use hex_color::HexColor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

pub const DISK_READ_LABEL_TEXT_SETTING_KEY: &'static str = "settings-disk-read-label-text";
pub const DISK_WRITE_LABEL_TEXT_SETTING_KEY: &'static str = "settings-disk-write-label-text";
pub const DISK_READ_LABEL_COLOUR_SETTING_KEY: &'static str = "settings-disk-read-label-colour";
pub const DISK_WRITE_LABEL_COLOUR_SETTING_KEY: &'static str = "settings-disk-write-label-colour";

/// The configuration for the memory monitor
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiskConfiguration {
    /// The duration between each update interval, i.e. 5 seconds
    pub update_interval: Duration,
    /// The number of samples to keep and average for the final result
    pub max_samples: usize,
    /// The download label text
    pub read_label_text: String,
    /// The download label colour in hex format
    pub read_label_colour: HexColor,
    /// The upload label text
    pub write_label_text: String,
    /// The upload label colour in hex format
    pub write_label_colour: HexColor,
}

impl Default for DiskConfiguration {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_secs(1),
            max_samples: 3,
            read_label_text: "R".to_string(),
            read_label_colour: "#029BAC".parse().unwrap(),
            write_label_text: "W".to_string(),
            write_label_colour: "#029BAC".parse().unwrap(),
        }
    }
}

impl DiskConfiguration {
    pub fn from(&self, settings_form: &SettingsForm) -> Self {
        if settings_form.settings_window_id != DISK_SETTINGS_WINDOW_ID.clone() {
            panic!("Attempted to update disk settings from a non-disk settings window.")
        }

        DiskConfiguration {
            update_interval: ConfigurationValidation::sanitise_interval_input(
                settings_form
                    .values
                    .get(crate::configuration::app_configuration::UPDATE_INTERVAL_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
                self.update_interval,
            ),
            max_samples: ConfigurationValidation::sanitise_max_samples(
                settings_form.values.get(MAX_SAMPLES_SETTING_KEY).unwrap().value.clone(),
                self.max_samples,
            ),
            read_label_text: ConfigurationValidation::sanitise_label_text(
                settings_form
                    .values
                    .get(DISK_READ_LABEL_TEXT_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
            ),
            read_label_colour: ConfigurationValidation::sanitise_label_colour(
                settings_form
                    .values
                    .get(DISK_READ_LABEL_COLOUR_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
                self.read_label_colour,
            ),
            write_label_text: ConfigurationValidation::sanitise_label_text(
                settings_form
                    .values
                    .get(DISK_READ_LABEL_TEXT_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
            ),
            write_label_colour: ConfigurationValidation::sanitise_label_colour(
                settings_form
                    .values
                    .get(DISK_READ_LABEL_COLOUR_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
                self.write_label_colour,
            ),
        }
    }

    pub fn to_settings_form(&self) -> SettingsForm {
        SettingsForm {
            settings_window_id: DISK_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-disk-title"),
            values: HashMap::from([
                (
                    DISK_READ_LABEL_TEXT_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-disk-read-label-text"),
                        value: self.read_label_text.clone(),
                        validator: Some(ConfigurationValidation::is_valid_label_text),
                    },
                ),
                (
                    DISK_WRITE_LABEL_TEXT_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-disk-write-label-text"),
                        value: self.write_label_text.clone(),
                        validator: Some(ConfigurationValidation::is_valid_label_text),
                    },
                ),
                (
                    DISK_READ_LABEL_COLOUR_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-disk-read-label-colour"),
                        value: self.read_label_colour.display_rgba().to_string(),
                        validator: Some(ConfigurationValidation::is_valid_colour),
                    },
                ),
                (
                    DISK_WRITE_LABEL_COLOUR_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-disk-write-label-colour"),
                        value: self.write_label_colour.display_rgba().to_string(),
                        validator: Some(ConfigurationValidation::is_valid_colour),
                    },
                ),
                (
                    UPDATE_INTERVAL_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-update-interval"),
                        value: self.update_interval.as_millis().to_string(),
                        validator: Some(ConfigurationValidation::is_valid_interval),
                    },
                ),
                (
                    MAX_SAMPLES_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-max-samples"),
                        value: self.max_samples.to_string(),
                        validator: Some(ConfigurationValidation::is_valid_max_samples),
                    },
                ),
            ]),
        }
    }
}
