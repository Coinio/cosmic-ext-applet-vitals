use cosmic::cosmic_theme::palette::Srgba;
use cosmic::cosmic_theme::CosmicPaletteInner;
use indexmap::IndexMap;

// Define constants for all colour keys
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
pub const WHITE: &str = "white";
pub const BLACK: &str = "black";


#[derive(Default)]
pub struct AppColours {
    pub colours: IndexMap<&'static str, Srgba>,
}

impl AppColours {
    pub fn get(&self, key: &str) -> Option<&Srgba> {
        self.colours.get(key)
    }
}

impl From<&CosmicPaletteInner> for AppColours {
    fn from(palette: &CosmicPaletteInner) -> Self {
        AppColours {
            colours: IndexMap::from([
                (BRIGHT_GREEN, palette.bright_green),
                (ACCENT_GREEN, palette.accent_green),
                (ACCENT_INDIGO, palette.accent_indigo),
                (EXT_INDIGO, palette.ext_indigo),
                (ACCENT_ORANGE, palette.accent_orange),
                (BRIGHT_ORANGE, palette.bright_orange),
                (EXT_ORANGE, palette.ext_orange),
                (ACCENT_RED, palette.accent_red),
                (BRIGHT_RED, palette.bright_red),
                (ACCENT_PURPLE, palette.accent_purple),
                (EXT_PURPLE, palette.ext_purple),
                (ACCENT_WARM_GREY, palette.accent_warm_grey),
                (EXT_WARM_GREY, palette.ext_warm_grey),
                (ACCENT_PINK, palette.accent_pink),
                (EXT_PINK, palette.ext_pink),
                (ACCENT_YELLOW, palette.accent_yellow),
                (EXT_YELLOW, palette.ext_yellow),
                (ACCENT_BLUE, palette.accent_blue),
                (EXT_BLUE, palette.ext_blue),
                (WHITE, Srgba::new(1.0, 1.0, 1.0, 1.0)),
                (BLACK, Srgba::new(0.0, 0.0, 0.0, 1.0)),
            ]),
        }
    }
}
