use serde::{Deserialize, Serialize};

use crate::configuration::app_configuration::{GENERAL_SETTINGS_WINDOW_ID, FIX_INDICATOR_SIZE_SETTING_KEY, USE_IEC_UNITS_SETTING_KEY};
use crate::configuration::validation::ConfigurationValidation;
use crate::core::settings::SettingsForm;

const DEFAULT_INDICATOR_FONT_SIZE: u16 = 14;

/// General configuration for the app, i.e. font size.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfiguration {
    pub vertical_font_size_xs: u16,
    pub vertical_font_size_sm: u16,
    pub vertical_font_size_md: u16,
    pub vertical_font_size_lg: u16,
    pub vertical_font_size_xl: u16,
    pub horizontal_font_size_xs: u16,
    pub horizontal_font_size_sm: u16,
    pub horizontal_font_size_md: u16,
    pub horizontal_font_size_lg: u16,
    pub horizontal_font_size_xl: u16,
    /// When true, indicator value widths are measured and fixed based on their maximum text size;
    /// when false, they resize to the text
    pub fix_indicator_size: bool,
    /// When true, use IEC units (MiB, GiB); when false, use SI (MB, GB).
    pub use_iec_units: bool,
}

impl Default for GeneralConfiguration {
    fn default() -> Self {
        GeneralConfiguration {
            vertical_font_size_xs: DEFAULT_INDICATOR_FONT_SIZE - 5,
            vertical_font_size_sm: DEFAULT_INDICATOR_FONT_SIZE - 3,
            vertical_font_size_md: DEFAULT_INDICATOR_FONT_SIZE,
            vertical_font_size_lg: DEFAULT_INDICATOR_FONT_SIZE,
            vertical_font_size_xl: DEFAULT_INDICATOR_FONT_SIZE,
            horizontal_font_size_xs: DEFAULT_INDICATOR_FONT_SIZE,
            horizontal_font_size_sm: DEFAULT_INDICATOR_FONT_SIZE + 2,
            horizontal_font_size_md: DEFAULT_INDICATOR_FONT_SIZE + 4,
            horizontal_font_size_lg: DEFAULT_INDICATOR_FONT_SIZE + 6,
            horizontal_font_size_xl: DEFAULT_INDICATOR_FONT_SIZE + 8,
            fix_indicator_size: true,
            use_iec_units: false,
        }
    }
}

impl GeneralConfiguration {
    pub fn update(&self, settings_form: &SettingsForm) -> Self {
        if settings_form.settings_window_id != GENERAL_SETTINGS_WINDOW_ID.clone() {
            panic!("Attempted to update General settings from a non-general settings window.")
        }

        GeneralConfiguration {
            fix_indicator_size: ConfigurationValidation::sanitise_boolean_input(
                settings_form
                    .values
                    .get(FIX_INDICATOR_SIZE_SETTING_KEY)
                    .expect("Fix indicator size missing from settings form options")
                    .value
                    .clone(),
                self.fix_indicator_size,
            ),
            use_iec_units: ConfigurationValidation::sanitise_boolean_input(
                settings_form
                    .values
                    .get(USE_IEC_UNITS_SETTING_KEY)
                    .expect("Use IEC units missing from settings form options")
                    .value
                    .clone(),
                self.use_iec_units,
            ),
            ..self.clone()
        }
    }
}
