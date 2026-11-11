use crate::configuration::app_configuration::{
    SENSOR_INTERVAL_MINIMUM_IN_MS, SENSOR_MAX_SAMPLES_MINIMUM,
};
use crate::configuration::general::{INDICATOR_FONT_SIZE_MAXIMUM, INDICATOR_FONT_SIZE_MINIMUM};
use crate::fl;
use std::cmp;
use std::time::Duration;

pub struct ConfigurationValidation;

impl ConfigurationValidation {
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

    pub fn is_valid_boolean(input: &str) -> Result<(), String> {
        _ = input
            .trim()
            .parse::<bool>()
            .map_err(|_| "Not valid boolean".to_string())?;

        Ok(())
    }

    pub fn sanitise_boolean_input(new_input: String, previous_value: bool) -> bool {
        let validation_result = Self::is_valid_boolean(new_input.as_str());

        if validation_result.is_err() {
            return previous_value;
        }

        let value = new_input
            .trim()
            .parse::<bool>()
            .expect("Failed to parse. Should always be valid here.");

        value
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


    /// Out of range sizes are clamped, as they can only come from the spin button's bounds or a hand-edited config.
    pub fn sanitise_font_size(new_input: &str, previous_value: u16) -> u16 {
        new_input
            .trim()
            .parse::<u16>()
            .map_or(previous_value, |value| {
                value.clamp(INDICATOR_FONT_SIZE_MINIMUM, INDICATOR_FONT_SIZE_MAXIMUM)
            })
    }
}

#[cfg(test)]
mod font_size_tests {
    use crate::configuration::general::{INDICATOR_FONT_SIZE_MAXIMUM, INDICATOR_FONT_SIZE_MINIMUM};
    use crate::configuration::validation::ConfigurationValidation;

    #[test]
    fn sanitise_font_size_accepts_values_in_range() {
        assert_eq!(ConfigurationValidation::sanitise_font_size("16", 14), 16);
        assert_eq!(ConfigurationValidation::sanitise_font_size(" 20 ", 14), 20);
    }

    #[test]
    fn sanitise_font_size_returns_previous_when_not_numeric() {
        assert_eq!(ConfigurationValidation::sanitise_font_size("abc", 14), 14);
        assert_eq!(ConfigurationValidation::sanitise_font_size("", 14), 14);
        assert_eq!(ConfigurationValidation::sanitise_font_size("-1", 14), 14);
    }

    #[test]
    fn sanitise_font_size_clamps_to_bounds() {
        assert_eq!(ConfigurationValidation::sanitise_font_size("1", 14), INDICATOR_FONT_SIZE_MINIMUM);
        assert_eq!(ConfigurationValidation::sanitise_font_size("500", 14), INDICATOR_FONT_SIZE_MAXIMUM);
    }
}

#[cfg(test)]
mod interval_tests {
    use crate::configuration::app_configuration::SENSOR_INTERVAL_MINIMUM_IN_MS;
    use crate::configuration::validation::ConfigurationValidation;
    use std::time::Duration;

    #[test]
    fn is_valid_interval_accepts_min_and_above() {
        let min = SENSOR_INTERVAL_MINIMUM_IN_MS;
        assert!(ConfigurationValidation::is_valid_interval(&min.to_string()).is_ok());
        assert!(ConfigurationValidation::is_valid_interval(&(min + 1).to_string()).is_ok());
    }

    #[test]
    fn is_valid_interval_rejects_below_min_or_non_numeric() {
        let min = SENSOR_INTERVAL_MINIMUM_IN_MS;
        assert!(ConfigurationValidation::is_valid_interval("-10").is_err());
        assert!(ConfigurationValidation::is_valid_interval(&(min - 1).to_string()).is_err());
        assert!(ConfigurationValidation::is_valid_interval("abc").is_err());
        assert!(ConfigurationValidation::is_valid_interval("").is_err());
        assert!(ConfigurationValidation::is_valid_interval("  ").is_err());
    }

    #[test]
    fn sanitise_interval_input_returns_previous_when_not_numeric() {
        let previous = Duration::from_millis(1_000);
        let result = ConfigurationValidation::sanitise_interval_input("abc".to_string(), previous);
        assert_eq!(result, previous);
    }

    #[test]
    fn sanitise_interval_input_returns_previous_when_below_minimum() {
        let previous = Duration::from_millis(1_000);
        let min = SENSOR_INTERVAL_MINIMUM_IN_MS;
        let below = (min - 1).to_string();
        let result = ConfigurationValidation::sanitise_interval_input(below, previous);
        assert_eq!(result, previous);
    }

    #[test]
    fn sanitise_interval_input_returns_min_when_min() {
        let previous = Duration::from_millis(1_000);
        let min = SENSOR_INTERVAL_MINIMUM_IN_MS;
        let res_min = ConfigurationValidation::sanitise_interval_input(min.to_string(), previous);
        assert_eq!(res_min, Duration::from_millis(min));
    }

    #[test]
    fn sanitise_interval_input_returns_value_when_above_min() {
        let previous = Duration::from_millis(1_000);
        let min = SENSOR_INTERVAL_MINIMUM_IN_MS;
        let val = min + 1;
        let res_above = ConfigurationValidation::sanitise_interval_input(val.to_string(), previous);
        assert_eq!(res_above, Duration::from_millis(val));
    }
}

#[cfg(test)]
mod max_samples_tests {
    use crate::configuration::app_configuration::SENSOR_MAX_SAMPLES_MINIMUM;
    use crate::configuration::validation::ConfigurationValidation;

    #[test]
    fn is_valid_max_samples_accepts_min_and_above() {
        let min = SENSOR_MAX_SAMPLES_MINIMUM;
        assert!(ConfigurationValidation::is_valid_max_samples(&min.to_string()).is_ok());
        assert!(ConfigurationValidation::is_valid_max_samples(&(min + 3).to_string()).is_ok());
    }

    #[test]
    fn is_valid_max_samples_rejects_below_min_or_non_numeric() {
        let min = SENSOR_MAX_SAMPLES_MINIMUM;
        assert!(ConfigurationValidation::is_valid_max_samples(&(min - 1).to_string()).is_err());
        assert!(ConfigurationValidation::is_valid_max_samples("abc").is_err());
        assert!(ConfigurationValidation::is_valid_max_samples("").is_err());
        assert!(ConfigurationValidation::is_valid_max_samples("  ").is_err());
    }

    #[test]
    fn sanitise_max_samples_returns_old_when_non_numeric() {
        let old_value = 10;
        let result = ConfigurationValidation::sanitise_max_samples("notnum".to_string(), old_value);
        assert_eq!(result, old_value);
    }

    #[test]
    fn sanitise_max_samples_returns_old_when_below_minimum() {
        let old_value = 10;
        let below = (SENSOR_MAX_SAMPLES_MINIMUM - 1).to_string();
        let result = ConfigurationValidation::sanitise_max_samples(below, old_value);
        assert_eq!(result, old_value);
    }

    #[test]
    fn sanitise_max_samples_returns_min_when_exact_minimum() {
        let old_value = 3;
        let min = SENSOR_MAX_SAMPLES_MINIMUM;
        let result = ConfigurationValidation::sanitise_max_samples(min.to_string(), old_value);
        assert_eq!(result, min);
    }

    #[test]
    fn sanitise_max_samples_returns_value_when_above_minimum() {
        let old_value = 3;
        let min = SENSOR_MAX_SAMPLES_MINIMUM;
        let test_value = min + 5;
        let result = ConfigurationValidation::sanitise_max_samples(test_value.to_string(), old_value);
        assert_eq!(result, test_value);
    }
}

#[cfg(test)]
mod boolean_tests {
    use super::ConfigurationValidation;

    #[test]
    fn is_valid_boolean_accepts_true_and_false() {
        assert!(ConfigurationValidation::is_valid_boolean("true").is_ok());
        assert!(ConfigurationValidation::is_valid_boolean("false").is_ok());
        assert!(ConfigurationValidation::is_valid_boolean("  true  ").is_ok());
        assert!(ConfigurationValidation::is_valid_boolean("\nfalse\t").is_ok());
    }

    #[test]
    fn is_valid_boolean_rejects_invalid_inputs() {
        assert!(ConfigurationValidation::is_valid_boolean("").is_err());
        assert!(ConfigurationValidation::is_valid_boolean(" ").is_err());
        assert!(ConfigurationValidation::is_valid_boolean("yes").is_err());
        assert!(ConfigurationValidation::is_valid_boolean("no").is_err());
        assert!(ConfigurationValidation::is_valid_boolean("1").is_err());
        assert!(ConfigurationValidation::is_valid_boolean("0").is_err());
        assert!(ConfigurationValidation::is_valid_boolean("TRUE").is_err());
        assert!(ConfigurationValidation::is_valid_boolean("FALSE").is_err());
    }

    #[test]
    fn sanitise_boolean_input_returns_previous_when_invalid() {
        let previous = true;
        let result = ConfigurationValidation::sanitise_boolean_input("maybe".to_string(), previous);
        assert_eq!(result, previous);
    }

    #[test]
    fn sanitise_boolean_input_parses_and_returns_value_when_valid() {
        let previous = false;
        let res_true = ConfigurationValidation::sanitise_boolean_input("true".to_string(), previous);
        assert_eq!(res_true, true);

        let previous2 = true;
        let res_false = ConfigurationValidation::sanitise_boolean_input("false".to_string(), previous2);
        assert_eq!(res_false, false);
    }

    #[test]
    fn sanitise_boolean_input_handles_whitespace() {
        let previous = false;
        let res = ConfigurationValidation::sanitise_boolean_input("  true  ".to_string(), previous);
        assert_eq!(res, true);
    }
}
