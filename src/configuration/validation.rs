use crate::configuration::app_configuration::{
    SENSOR_INTERVAL_MINIMUM_IN_MS, SENSOR_MAX_LABEL_LENGTH, SENSOR_MAX_SAMPLES_MINIMUM,
};
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

    pub fn is_valid_label_text(input: &str) -> Result<(), String> {
        if input.trim().is_empty() || input.len() > SENSOR_MAX_LABEL_LENGTH {
            Err(fl!("settings-label-text-error", max_length = SENSOR_MAX_LABEL_LENGTH))
        } else {
            Ok(())
        }
    }

    pub fn sanitise_label_text(new_input: String, previous_input: String) -> String {
        let validation_result = Self::is_valid_label_text(new_input.as_str());

        if validation_result.is_err() {
            if new_input.trim().is_empty() {
                return previous_input.trim().to_string();
            }

            return new_input[..SENSOR_MAX_LABEL_LENGTH].trim().to_string();
        }

        new_input.trim().to_string()
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

#[cfg(test)]
mod label_text_tests {
    use super::ConfigurationValidation;
    use crate::configuration::app_configuration::SENSOR_MAX_LABEL_LENGTH;

    #[test]
    fn is_valid_label_text_accepts_max_length() {
        let max = SENSOR_MAX_LABEL_LENGTH;
        let max_length_string = "X".repeat(max);
        assert!(ConfigurationValidation::is_valid_label_text(&max_length_string).is_ok());
    }

    #[test]
    fn is_valid_label_text_rejects_over_max_length() {
        let max = SENSOR_MAX_LABEL_LENGTH;
        let too_long_string = "Y".repeat(max + 1);
        assert!(ConfigurationValidation::is_valid_label_text(&too_long_string).is_err());
    }

    #[test]
    fn is_valid_label_text_rejects_empty() {
        assert!(ConfigurationValidation::is_valid_label_text("").is_err());
        assert!(ConfigurationValidation::is_valid_label_text("    ").is_err());
    }

    #[test]
    fn sanitise_label_text_truncates_over_length() {
        let max = SENSOR_MAX_LABEL_LENGTH;
        let too_long = "Z".repeat(max + 5);
        let truncated = ConfigurationValidation::sanitise_label_text(too_long.clone(), too_long.clone());
        assert_eq!(truncated.len(), max);
        assert_eq!(truncated, "Z".repeat(max));
    }

    #[test]
    fn sanitise_label_text_trims_whitespace() {
        let spaced = "   Hello  ".to_string();
        let trimmed = ConfigurationValidation::sanitise_label_text(spaced.clone(), spaced.clone());
        assert_eq!(trimmed, "Hello");
    }
    
    #[test]
    fn sanitise_label_text_returns_previous_when_empty() {
        let previous = "Hello".to_string();
        let empty = "".to_string();
        let result = ConfigurationValidation::sanitise_label_text(empty.clone(), previous.clone());
        assert_eq!(result, previous);
    }
    
    #[test]
    fn sanitise_label_text_trims_text_when_too_long() {
        let trimmed = "X".repeat(SENSOR_MAX_LABEL_LENGTH);
        let too_long = "X".repeat(SENSOR_MAX_LABEL_LENGTH + 1);
        let result = ConfigurationValidation::sanitise_label_text(too_long.clone(), "X".repeat(SENSOR_MAX_LABEL_LENGTH));
        assert_eq!(result, trimmed);   
    }
}
