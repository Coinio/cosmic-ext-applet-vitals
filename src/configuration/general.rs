use serde::{Deserialize, Serialize};

use crate::configuration::app_configuration::{
    BOLD_LABELS_SETTING_KEY, FIX_INDICATOR_SIZE_SETTING_KEY, FONT_SETTING_KEY, GENERAL_SETTINGS_WINDOW_ID, USE_IEC_UNITS_SETTING_KEY,
};
use crate::configuration::validation::ConfigurationValidation;
use crate::core::settings::SettingsForm;

const DEFAULT_INDICATOR_FONT_SIZE: u16 = 14;

const FONT_FORM_VALUE_SYSTEM: &str = "system";
const FONT_FORM_VALUE_FAMILY_PREFIX: &str = "family:";

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum IndicatorFont {
    /// The COSMIC interface font
    #[default]
    System,
    Family(String),
}

impl IndicatorFont {
    /// Family names are prefixed so a font called "system" can't clash with that option.
    pub fn to_form_value(&self) -> String {
        match self {
            IndicatorFont::System => FONT_FORM_VALUE_SYSTEM.to_string(),
            IndicatorFont::Family(family) => format!("{FONT_FORM_VALUE_FAMILY_PREFIX}{family}"),
        }
    }

    pub fn from_form_value(value: &str) -> Option<Self> {
        match value {
            FONT_FORM_VALUE_SYSTEM => Some(IndicatorFont::System),
            _ => value
                .strip_prefix(FONT_FORM_VALUE_FAMILY_PREFIX)
                .map(str::trim)
                .filter(|family| !family.is_empty())
                .map(|family| IndicatorFont::Family(family.to_string())),
        }
    }
}

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
    pub font: IndicatorFont,
    /// When true, the indicator labels use a bold weight of the indicator font.
    pub bold_labels: bool,
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
            horizontal_font_size_sm: DEFAULT_INDICATOR_FONT_SIZE,
            horizontal_font_size_md: DEFAULT_INDICATOR_FONT_SIZE + 4,
            horizontal_font_size_lg: DEFAULT_INDICATOR_FONT_SIZE + 6,
            horizontal_font_size_xl: DEFAULT_INDICATOR_FONT_SIZE + 8,
            fix_indicator_size: true,
            use_iec_units: false,
            font: IndicatorFont::System,
            bold_labels: true,
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
            font: settings_form
                .values
                .get(FONT_SETTING_KEY)
                .and_then(|item| IndicatorFont::from_form_value(&item.value))
                .unwrap_or_else(|| self.font.clone()),
            bold_labels: ConfigurationValidation::sanitise_boolean_input(
                settings_form
                    .values
                    .get(BOLD_LABELS_SETTING_KEY)
                    .expect("Bold labels missing from settings form options")
                    .value
                    .clone(),
                self.bold_labels,
            ),
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IndicatorFont;

    #[test]
    fn font_form_value_round_trips() {
        for font in [
            IndicatorFont::System,
            IndicatorFont::Family("JetBrains Mono".to_string()),
            IndicatorFont::Family("system".to_string()),
        ] {
            assert_eq!(IndicatorFont::from_form_value(&font.to_form_value()), Some(font));
        }
    }

    #[test]
    fn invalid_font_form_values_are_rejected() {
        assert_eq!(IndicatorFont::from_form_value(""), None);
        assert_eq!(IndicatorFont::from_form_value("Fira Sans"), None);
        assert_eq!(IndicatorFont::from_form_value("family:"), None);
        assert_eq!(IndicatorFont::from_form_value("family:   "), None);
    }
}
