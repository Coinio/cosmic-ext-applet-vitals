use cosmic::applet::cosmic_panel_config::PanelSize;
use serde::{Deserialize, Serialize};

use crate::configuration::app_configuration::{
    BOLD_LABELS_SETTING_KEY, FIX_INDICATOR_SIZE_SETTING_KEY, FONT_SETTING_KEY, FONT_SIZE_SETTING_KEY,
    GENERAL_SETTINGS_WINDOW_ID, USE_IEC_UNITS_SETTING_KEY,
};
use crate::configuration::validation::ConfigurationValidation;
use crate::core::settings::SettingsForm;

const DEFAULT_INDICATOR_FONT_SIZE: u16 = 14;
pub const INDICATOR_FONT_SIZE_MINIMUM: u16 = 6;
pub const INDICATOR_FONT_SIZE_MAXIMUM: u16 = 48;

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

/// The named panel/dock sizes, each of which has its own configured font size.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PanelSizeName {
    XS,
    S,
    M,
    L,
    XL,
}

impl PanelSizeName {
    pub fn label(&self) -> &'static str {
        match self {
            PanelSizeName::XS => "XS",
            PanelSizeName::S => "S",
            PanelSizeName::M => "M",
            PanelSizeName::L => "L",
            PanelSizeName::XL => "XL",
        }
    }
}

/// Identifies which of the configured font sizes applies to the panel/dock the applet is running in.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FontSizeSlot {
    pub horizontal: bool,
    pub size: PanelSizeName,
}

impl FontSizeSlot {
    /// Custom and hardcoded sizes have no configured font size, so they have no slot.
    pub fn from_applet_size(size: &cosmic::applet::Size, horizontal: bool) -> Option<Self> {
        let size = match size {
            cosmic::applet::Size::PanelSize(PanelSize::XS) => PanelSizeName::XS,
            cosmic::applet::Size::PanelSize(PanelSize::S) => PanelSizeName::S,
            cosmic::applet::Size::PanelSize(PanelSize::M) => PanelSizeName::M,
            cosmic::applet::Size::PanelSize(PanelSize::L) => PanelSizeName::L,
            cosmic::applet::Size::PanelSize(PanelSize::XL) => PanelSizeName::XL,
            _ => return None,
        };

        Some(FontSizeSlot { horizontal, size })
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
    pub fn font_size(&self, slot: FontSizeSlot) -> u16 {
        match (slot.horizontal, slot.size) {
            (true, PanelSizeName::XS) => self.horizontal_font_size_xs,
            (true, PanelSizeName::S) => self.horizontal_font_size_sm,
            (true, PanelSizeName::M) => self.horizontal_font_size_md,
            (true, PanelSizeName::L) => self.horizontal_font_size_lg,
            (true, PanelSizeName::XL) => self.horizontal_font_size_xl,
            (false, PanelSizeName::XS) => self.vertical_font_size_xs,
            (false, PanelSizeName::S) => self.vertical_font_size_sm,
            (false, PanelSizeName::M) => self.vertical_font_size_md,
            (false, PanelSizeName::L) => self.vertical_font_size_lg,
            (false, PanelSizeName::XL) => self.vertical_font_size_xl,
        }
    }

    fn font_size_mut(&mut self, slot: FontSizeSlot) -> &mut u16 {
        match (slot.horizontal, slot.size) {
            (true, PanelSizeName::XS) => &mut self.horizontal_font_size_xs,
            (true, PanelSizeName::S) => &mut self.horizontal_font_size_sm,
            (true, PanelSizeName::M) => &mut self.horizontal_font_size_md,
            (true, PanelSizeName::L) => &mut self.horizontal_font_size_lg,
            (true, PanelSizeName::XL) => &mut self.horizontal_font_size_xl,
            (false, PanelSizeName::XS) => &mut self.vertical_font_size_xs,
            (false, PanelSizeName::S) => &mut self.vertical_font_size_sm,
            (false, PanelSizeName::M) => &mut self.vertical_font_size_md,
            (false, PanelSizeName::L) => &mut self.vertical_font_size_lg,
            (false, PanelSizeName::XL) => &mut self.vertical_font_size_xl,
        }
    }

    /// Only the font size for `font_size_slot` is editable, as the form only shows the current panel/dock size.
    pub fn update(&self, settings_form: &SettingsForm, font_size_slot: Option<FontSizeSlot>) -> Self {
        if settings_form.settings_window_id != GENERAL_SETTINGS_WINDOW_ID.clone() {
            panic!("Attempted to update General settings from a non-general settings window.")
        }

        let mut configuration = GeneralConfiguration {
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
        };

        if let Some(slot) = font_size_slot {
            if let Some(item) = settings_form.values.get(FONT_SIZE_SETTING_KEY) {
                let font_size = configuration.font_size_mut(slot);
                *font_size = ConfigurationValidation::sanitise_font_size(&item.value, *font_size);
            }
        }

        configuration
    }
}

#[cfg(test)]
mod tests {
    use super::{FontSizeSlot, GeneralConfiguration, IndicatorFont, PanelSizeName};
    use crate::configuration::app_configuration::FONT_SIZE_SETTING_KEY;
    use crate::core::settings::SettingsForm;
    use cosmic::applet::cosmic_panel_config::PanelSize;

    #[test]
    fn named_panel_sizes_have_a_font_size_slot() {
        let slot = FontSizeSlot::from_applet_size(&cosmic::applet::Size::PanelSize(PanelSize::XL), true);
        assert_eq!(
            slot,
            Some(FontSizeSlot {
                horizontal: true,
                size: PanelSizeName::XL
            })
        );
    }

    #[test]
    fn custom_panel_sizes_have_no_font_size_slot() {
        assert_eq!(
            FontSizeSlot::from_applet_size(&cosmic::applet::Size::PanelSize(PanelSize::Custom(40)), true),
            None
        );
        assert_eq!(
            FontSizeSlot::from_applet_size(&cosmic::applet::Size::Hardcoded((10, 10)), false),
            None
        );
    }

    #[test]
    fn font_size_field_is_hidden_without_a_slot() {
        let form = SettingsForm::from_general(&GeneralConfiguration::default(), None);
        assert!(form.values.get(FONT_SIZE_SETTING_KEY).is_none());
    }

    #[test]
    fn update_only_changes_the_font_size_for_the_current_slot() {
        let slot = FontSizeSlot {
            horizontal: false,
            size: PanelSizeName::L,
        };
        let configuration = GeneralConfiguration::default();
        let mut form = SettingsForm::from_general(&configuration, Some(slot));

        let item = form.values.get_mut(FONT_SIZE_SETTING_KEY).expect("Font size missing from form");
        assert_eq!(item.value, configuration.vertical_font_size_lg.to_string());
        item.value = "21".to_string();

        let updated = configuration.update(&form, Some(slot));

        assert_eq!(
            updated,
            GeneralConfiguration {
                vertical_font_size_lg: 21,
                ..configuration
            }
        );
    }

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
