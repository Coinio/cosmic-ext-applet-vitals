use crate::configuration::app_configuration::{MAX_SAMPLES_SETTING_KEY, NETWORK_SETTINGS_WINDOW_ID, UPDATE_INTERVAL_SETTING_KEY};
use crate::configuration::validation::ConfigurationValidation;
use crate::fl;
use crate::ui::settings_form::{
    SettingsForm, SettingsFormItem
};
use hex_color::HexColor;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

pub const NETWORK_RX_LABEL_TEXT_SETTING_KEY: &'static str = "settings-network-rx-label-text";
pub const NETWORK_TX_LABEL_TEXT_SETTING_KEY: &'static str = "settings-network-tx-label-text";
pub const NETWORK_RX_LABEL_COLOUR_SETTING_KEY: &'static str = "settings-network-rx-label-colour";
pub const NETWORK_TX_LABEL_COLOUR_SETTING_KEY: &'static str = "settings-network-tx-label-colour";

/// The configuration for the network monitor
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct NetworkConfiguration {
    /// The duration between each update interval, i.e. 5 seconds
    pub update_interval: Duration,
    /// The number of samples to keep and average for the final result
    pub max_samples: usize,
    /// The download label text
    pub rx_label_text: String,
    /// The download label colour in hex format
    pub rx_label_colour: HexColor,
    /// The upload label text
    pub tx_label_text: String,
    /// The upload label colour in hex format
    pub tx_label_colour: HexColor,
}

impl Default for NetworkConfiguration {
    fn default() -> Self {
        NetworkConfiguration {
            update_interval: Duration::from_secs(1),
            max_samples: 4,
            rx_label_text: "RX".to_string(),
            rx_label_colour: "#029BAC".parse().unwrap(),
            tx_label_text: "TX".to_string(),
            tx_label_colour: "#029BAC".parse().unwrap(),
        }
    }
}

impl NetworkConfiguration {
    pub fn from(&self, settings_form: &SettingsForm) -> Self {
        if settings_form.settings_window_id != NETWORK_SETTINGS_WINDOW_ID.clone() {
            panic!("Attempted to update network settings from a non-network settings window.")
        }

        NetworkConfiguration {
            update_interval: ConfigurationValidation::sanitise_interval_input(
                settings_form
                    .values
                    .get(UPDATE_INTERVAL_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
                self.update_interval,
            ),
            max_samples: ConfigurationValidation::sanitise_max_samples(
                settings_form.values.get(MAX_SAMPLES_SETTING_KEY).unwrap().value.clone(),
                self.max_samples,
            ),
            rx_label_text: ConfigurationValidation::sanitise_label_text(
                settings_form
                    .values
                    .get(NETWORK_RX_LABEL_TEXT_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
            ),
            rx_label_colour: ConfigurationValidation::sanitise_label_colour(
                settings_form
                    .values
                    .get(NETWORK_RX_LABEL_COLOUR_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
                self.rx_label_colour,
            ),
            tx_label_text: ConfigurationValidation::sanitise_label_text(
                settings_form
                    .values
                    .get(NETWORK_TX_LABEL_TEXT_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
            ),
            tx_label_colour: ConfigurationValidation::sanitise_label_colour(
                settings_form
                    .values
                    .get(NETWORK_TX_LABEL_COLOUR_SETTING_KEY)
                    .unwrap()
                    .value
                    .clone(),
                self.tx_label_colour,
            ),
        }
    }

    pub fn to_settings_form(&self) -> SettingsForm {
        SettingsForm {
            settings_window_id: NETWORK_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-network-title"),
            values: BTreeMap::from([
                (
                    NETWORK_RX_LABEL_TEXT_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-network-rx-label-text"),
                        value: self.rx_label_text.clone(),
                        validator: Some(ConfigurationValidation::is_valid_label_text)
                    },
                ),

                (
                    NETWORK_RX_LABEL_COLOUR_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-network-rx-label-colour"),
                        value: self.rx_label_colour.display_rgba().to_string(),
                        validator: Some(ConfigurationValidation::is_valid_colour)
                    },
                ),
                (
                    NETWORK_TX_LABEL_TEXT_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-network-tx-label-text"),
                        value: self.tx_label_text.clone(),
                        validator: Some(ConfigurationValidation::is_valid_label_text)
                    },
                ),
                (
                    NETWORK_TX_LABEL_COLOUR_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-network-tx-label-colour"),
                        value: self.tx_label_colour.display_rgba().to_string(),
                        validator: Some(ConfigurationValidation::is_valid_colour)
                    },
                ),
                (
                    UPDATE_INTERVAL_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-update-interval"),
                        value: self.update_interval.as_millis().to_string(),
                        validator: Some(ConfigurationValidation::is_valid_interval)
                    },
                ),
                (
                    MAX_SAMPLES_SETTING_KEY,
                    SettingsFormItem {
                        label: fl!("settings-max-samples"),
                        value: self.max_samples.to_string(),
                        validator: Some(ConfigurationValidation::is_valid_max_samples)
                    },
                ),
            ]),
        }
    }
}
