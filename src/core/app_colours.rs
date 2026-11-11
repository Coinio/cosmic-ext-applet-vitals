use cosmic::cosmic_theme::{CosmicPaletteInner, Theme};
use cosmic::iced::Color;

/// The theme's accent colour, which follows the accent chosen in COSMIC Settings.
pub const THEME_ACCENT: &str = "accent";

pub const BRIGHT_GREEN: &str = "bright_green";
pub const ACCENT_GREEN: &str = "accent_green";
pub const ACCENT_INDIGO: &str = "accent_indigo";
pub const EXT_INDIGO: &str = "ext_indigo";
pub const ACCENT_ORANGE: &str = "accent_orange";
pub const BRIGHT_ORANGE: &str = "bright_orange";
pub const EXT_ORANGE: &str = "ext_orange";
pub const ACCENT_RED: &str = "accent_red";
pub const BRIGHT_RED: &str = "bright_red";
pub const ACCENT_PURPLE: &str = "accent_purple";
pub const EXT_PURPLE: &str = "ext_purple";
pub const ACCENT_WARM_GREY: &str = "accent_warm_grey";
pub const EXT_WARM_GREY: &str = "ext_warm_grey";
pub const ACCENT_PINK: &str = "accent_pink";
pub const EXT_PINK: &str = "ext_pink";
pub const ACCENT_YELLOW: &str = "accent_yellow";
pub const EXT_YELLOW: &str = "ext_yellow";
pub const ACCENT_BLUE: &str = "accent_blue";
pub const EXT_BLUE: &str = "ext_blue";

/// Every palette key that can be stored in the configuration. Earlier versions offered all of these in the colour
/// picker, so they must keep resolving even though only the accent colours are offered now.
const PALETTE_KEYS: [&str; 19] = [
    BRIGHT_GREEN,
    ACCENT_GREEN,
    ACCENT_INDIGO,
    EXT_INDIGO,
    ACCENT_ORANGE,
    BRIGHT_ORANGE,
    EXT_ORANGE,
    ACCENT_RED,
    BRIGHT_RED,
    ACCENT_PURPLE,
    EXT_PURPLE,
    ACCENT_WARM_GREY,
    EXT_WARM_GREY,
    ACCENT_PINK,
    EXT_PINK,
    ACCENT_YELLOW,
    EXT_YELLOW,
    ACCENT_BLUE,
    EXT_BLUE,
];

/// The colours offered as presets in the colour picker. The accent palette is the one COSMIC adjusts for contrast
/// in light and dark themes, unlike the extended palette which stays the same in both.
pub const PRESET_COLOURS: [AppColour; 10] = [
    AppColour::ThemeAccent,
    AppColour::Palette(ACCENT_BLUE),
    AppColour::Palette(ACCENT_INDIGO),
    AppColour::Palette(ACCENT_PURPLE),
    AppColour::Palette(ACCENT_PINK),
    AppColour::Palette(ACCENT_RED),
    AppColour::Palette(ACCENT_ORANGE),
    AppColour::Palette(ACCENT_YELLOW),
    AppColour::Palette(ACCENT_GREEN),
    AppColour::Palette(ACCENT_WARM_GREY),
];

/// A colour as stored in the configuration. Theme colours are resolved against the theme they are drawn with, so
/// they follow light/dark and accent changes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppColour {
    /// The theme's accent colour.
    ThemeAccent,
    /// A named colour from the theme palette, one of `PALETTE_KEYS`.
    Palette(&'static str),
    /// A fixed colour chosen with the custom colour picker, stored as `#RRGGBB`.
    Custom(Color),
}

impl AppColour {
    /// Parses a configuration value. Returns `None` for values that aren't a known palette key or a hex colour.
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        if value == THEME_ACCENT {
            return Some(AppColour::ThemeAccent);
        }
        if let Some(key) = PALETTE_KEYS.iter().find(|key| **key == value) {
            return Some(AppColour::Palette(key));
        }
        parse_hex(value).map(AppColour::Custom)
    }

    /// The value stored in the configuration for this colour.
    pub fn to_config_value(self) -> String {
        match self {
            AppColour::ThemeAccent => THEME_ACCENT.to_string(),
            AppColour::Palette(key) => key.to_string(),
            AppColour::Custom(colour) => to_hex(colour),
        }
    }

    pub fn resolve(self, theme: &Theme) -> Color {
        match self {
            AppColour::ThemeAccent => theme.accent_color().into(),
            AppColour::Palette(key) => palette_colour(&theme.palette, key).into(),
            AppColour::Custom(colour) => colour,
        }
    }

    pub fn is_preset(self) -> bool {
        PRESET_COLOURS.contains(&self)
    }
}

fn palette_colour(palette: &CosmicPaletteInner, key: &str) -> cosmic::cosmic_theme::palette::Srgba {
    match key {
        BRIGHT_GREEN => palette.bright_green,
        ACCENT_GREEN => palette.accent_green,
        ACCENT_INDIGO => palette.accent_indigo,
        EXT_INDIGO => palette.ext_indigo,
        ACCENT_ORANGE => palette.accent_orange,
        BRIGHT_ORANGE => palette.bright_orange,
        EXT_ORANGE => palette.ext_orange,
        ACCENT_RED => palette.accent_red,
        BRIGHT_RED => palette.bright_red,
        ACCENT_PURPLE => palette.accent_purple,
        EXT_PURPLE => palette.ext_purple,
        ACCENT_WARM_GREY => palette.accent_warm_grey,
        EXT_WARM_GREY => palette.ext_warm_grey,
        ACCENT_PINK => palette.accent_pink,
        EXT_PINK => palette.ext_pink,
        ACCENT_YELLOW => palette.accent_yellow,
        EXT_YELLOW => palette.ext_yellow,
        ACCENT_BLUE => palette.accent_blue,
        EXT_BLUE => palette.ext_blue,
        _ => unreachable!("Unknown palette key: {key}"),
    }
}

/// Parses `#RRGGBB`, with or without the `#`.
pub fn parse_hex(value: &str) -> Option<Color> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 || !hex.is_ascii() {
        return None;
    }
    let channel = |index: usize| u8::from_str_radix(&hex[index..index + 2], 16).ok();
    Some(Color::from_rgb8(channel(0)?, channel(2)?, channel(4)?))
}

/// Formats a colour as `#RRGGBB`, ignoring its alpha.
pub fn to_hex(colour: Color) -> String {
    let [red, green, blue, _] = colour.into_rgba8();
    format!("#{red:02X}{green:02X}{blue:02X}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_palette_key_from_earlier_versions() {
        for key in PALETTE_KEYS {
            assert_eq!(AppColour::parse(key), Some(AppColour::Palette(key)));
        }
    }

    #[test]
    fn parses_theme_accent() {
        assert_eq!(AppColour::parse(THEME_ACCENT), Some(AppColour::ThemeAccent));
    }

    #[test]
    fn parses_hex_colours() {
        assert_eq!(AppColour::parse("#FF8000"), Some(AppColour::Custom(Color::from_rgb8(255, 128, 0))));
        assert_eq!(AppColour::parse("ff8000"), Some(AppColour::Custom(Color::from_rgb8(255, 128, 0))));
    }

    #[test]
    fn rejects_invalid_values() {
        for value in ["", "#FFF", "#GGGGGG", "not_a_colour", "#FF80000", "#ÿÿÿ"] {
            assert_eq!(AppColour::parse(value), None, "{value}");
        }
    }

    #[test]
    fn round_trips_through_config_value() {
        for colour in [
            AppColour::ThemeAccent,
            AppColour::Palette(EXT_BLUE),
            AppColour::Custom(Color::from_rgb8(18, 52, 86)),
        ] {
            assert_eq!(AppColour::parse(&colour.to_config_value()), Some(colour));
        }
    }

    #[test]
    fn resolves_every_palette_key() {
        let theme = Theme::dark_default();
        for key in PALETTE_KEYS {
            AppColour::Palette(key).resolve(&theme);
        }
    }
}
