use crate::configuration::app_configuration::{
    SENSOR_INTERVAL_MINIMUM_IN_MS, SENSOR_MAX_LABEL_LENGTH, SENSOR_MAX_SAMPLES_MINIMUM,
};
use crate::fl;
use hex_color::HexColor;
use std::cmp;
use std::time::Duration;

pub struct ConfigurationValidation;

impl ConfigurationValidation {
    pub fn is_valid_colour(input: &str) -> Result<(), String> {
        match HexColor::parse(input) {
            Ok(value) => Ok(()),
            Err(_) => Err(fl!("settings-colour-error")),
        }
    }

    pub fn sanitise_label_colour(new_input: String, previous_colour: HexColor) -> HexColor {
        let validation_result = Self::is_valid_colour(new_input.as_str());

        match validation_result {
            Ok(_) => HexColor::parse(new_input.as_str()).expect("Failed to parse. Should always be valid here."),
            Err(_) => previous_colour,
        }
    }

    pub fn is_valid_interval(input: &str) -> Result<(), String> {
        let error_message = fl!(
            "settings-interval-error",
            min = SENSOR_INTERVAL_MINIMUM_IN_MS.to_string()
        );

        let value = input.trim().parse::<u64>().map_err(|_| error_message.clone())?;

        if value >= SENSOR_INTERVAL_MINIMUM_IN_MS {
            Ok(())
        } else {
            Err(error_message)
        }
    }
    pub fn sanitise_interval_input(new_input: String, previous_interval: Duration) -> Duration {
        let validation_result = Self::is_valid_interval(new_input.as_str());

        if validation_result.is_err() {
            return previous_interval;
        }

        let value = new_input
            .trim()
            .parse::<u64>()
            .expect("Failed to parse. Should always be valid here.");

        Duration::from_millis(cmp::max(value, SENSOR_INTERVAL_MINIMUM_IN_MS))
    }

    pub fn is_valid_max_samples(input: &str) -> Result<(), String> {
        let error_message = fl!(
            "settings-max-samples-error",
            min = SENSOR_MAX_SAMPLES_MINIMUM.to_string()
        );

        let value = input.trim().parse::<usize>().map_err(|_| error_message.clone())?;

        if value >= SENSOR_MAX_SAMPLES_MINIMUM {
            Ok(())
        } else {
            Err(error_message)
        }
    }

    pub fn sanitise_max_samples(new_input: String, old_value: usize) -> usize {
        let validation_result = Self::is_valid_max_samples(new_input.as_str());

        if validation_result.is_err() {
            return old_value;
        }

        let parsed_value = new_input
            .trim()
            .parse::<usize>()
            .expect("Failed to parse. Should always be valid here.");

        cmp::max(parsed_value, SENSOR_MAX_SAMPLES_MINIMUM)
    }

    pub fn is_valid_label_text(input: &str) -> Result<(), String> {
        if input.len() > SENSOR_MAX_LABEL_LENGTH {
            Err(fl!("settings-label-text-error", max_length = SENSOR_MAX_LABEL_LENGTH))
        } else {
            Ok(())
        }
    }

    pub fn sanitise_label_text(new_input: String) -> String {
        let validation_result = Self::is_valid_label_text(new_input.as_str());

        if validation_result.is_err() {
            return new_input[..SENSOR_MAX_LABEL_LENGTH].to_string();
        }

        new_input.trim().to_string()
    }
}
