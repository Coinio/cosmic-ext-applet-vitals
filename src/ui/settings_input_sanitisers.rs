use crate::core::app_configuration::{SENSOR_INTERVAL_MINIMUM_IN_MS, SENSOR_MAX_LABEL_LENGTH, SENSOR_MAX_SAMPLES_MINIMUM};
use std::cmp;
use std::time::Duration;
use hex_color::HexColor;

pub struct FormInputValidation;

impl FormInputValidation {
    pub(crate) fn sanitise_label_colour(new_input: String, previous_colour: HexColor) -> HexColor {
        HexColor::parse(&new_input).unwrap_or_else(|_| previous_colour)
    }
}

impl FormInputValidation {
   
    pub fn sanitise_interval_input(new_input: String, previous_interval: Duration) -> Duration {
        match new_input.trim().parse() {
            Ok(value) => Duration::from_millis(cmp::max(value, SENSOR_INTERVAL_MINIMUM_IN_MS)),
            Err(_) => previous_interval,
        }
    }

    pub fn sanitise_max_samples(new_input: String, old_value: usize) -> usize {
        match new_input.trim().parse() {
            Ok(value) => cmp::max(value, SENSOR_MAX_SAMPLES_MINIMUM),
            Err(_) => old_value,
        }
    }

    pub fn sanitise_label_text(new_input: String) -> String {
        if new_input.len() > SENSOR_MAX_LABEL_LENGTH {
            new_input[..SENSOR_MAX_LABEL_LENGTH].to_string()
        } else {
            new_input.to_string()
        }
    }   
    
}
