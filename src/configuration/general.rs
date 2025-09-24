use serde::{Deserialize, Serialize};

const DEFAULT_INDICATOR_FONT_SIZE: u16 = 14;
const DEFAULT_INDICATOR_ICON_SIZE: u16 = 16;

/// General configuration for the app, i.e. font size.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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
    pub horizontal_font_size_xl: u16
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
        }
    }
}
