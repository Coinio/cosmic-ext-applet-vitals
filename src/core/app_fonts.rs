use crate::configuration::general::IndicatorFont;
use crate::fl;
use cosmic::config::FontConfig;
use cosmic::font::Font;
use cosmic::iced::font::{Stretch, Style, Weight};
use cosmic::iced_renderer::graphics::text::font_system;
use log::error;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::sync::Arc;

pub const DEFAULT_FONT_FAMILY: &str = "JetBrains Mono";

const DEFAULT_FONT_BYTES: &[u8] = include_bytes!("../../res/fonts/JetBrainsMono-Regular.ttf");

/// Internal iced fonts that shouldn't be offered to the user.
const HIDDEN_FONT_FAMILIES: &[&str] = &["Iced-Icons"];

pub fn load_embedded_fonts() {
    match font_system().write() {
        Ok(mut font_system) => font_system.load_font(Cow::Borrowed(DEFAULT_FONT_BYTES)),
        Err(err) => error!("Failed to load the embedded fonts: {}", err),
    }
}

pub fn resolve_font(choice: &IndicatorFont, interface_font: &Font) -> Font {
    match choice {
        IndicatorFont::Default => Font::with_name(DEFAULT_FONT_FAMILY),
        IndicatorFont::System => *interface_font,
        // FontConfig interns the family name, giving iced its &'static str without leaking on every reload.
        IndicatorFont::Family(family) => FontConfig {
            family: family.clone(),
            weight: Weight::Normal,
            stretch: Stretch::Normal,
            style: Style::Normal,
        }
        .into(),
    }
}

#[derive(Debug, Clone, Default)]
pub struct FontOptions {
    pub labels: Vec<String>,
    /// Parallel to `labels`.
    pub values: Arc<[String]>,
}

impl FontOptions {
    /// The configured font is included even if it's no longer installed, so it still shows as selected.
    pub fn new(configured: &IndicatorFont) -> Self {
        let mut families = installed_font_families();

        if let IndicatorFont::Family(family) = configured {
            if !families.contains(family) {
                families.push(family.clone());
                sort_families(&mut families);
            }
        }

        let mut labels = vec![
            fl!("settings-font-default"),
            fl!("settings-font-system"),
        ];
        let mut values = vec![
            IndicatorFont::Default.to_form_value(),
            IndicatorFont::System.to_form_value(),
        ];

        for family in families {
            values.push(IndicatorFont::Family(family.clone()).to_form_value());
            labels.push(family);
        }

        FontOptions {
            labels,
            values: values.into(),
        }
    }

    pub fn position(&self, form_value: &str) -> Option<usize> {
        self.values.iter().position(|value| value == form_value)
    }
}

fn installed_font_families() -> Vec<String> {
    let Ok(mut font_system) = font_system().write() else {
        error!("Failed to read the installed fonts");
        return Vec::new();
    };

    let unique: BTreeSet<String> = font_system
        .raw()
        .db()
        .faces()
        // The first family name is the English (US) one.
        .filter_map(|face| face.families.first().map(|(family, _)| family.trim().to_string()))
        .filter(|family| !family.is_empty() && !HIDDEN_FONT_FAMILIES.contains(&family.as_str()))
        .collect();

    let mut families: Vec<String> = unique.into_iter().collect();
    sort_families(&mut families);
    families
}

fn sort_families(families: &mut [String]) {
    families.sort_by_cached_key(|family| family.to_lowercase());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cosmic_text_measurer::CosmicTextMeasurer;

    #[test]
    fn embedded_default_font_is_available() {
        load_embedded_fonts();

        assert!(installed_font_families().iter().any(|family| family == DEFAULT_FONT_FAMILY));

        // Only equal if the monospaced font was used rather than a proportional fallback.
        let font = resolve_font(&IndicatorFont::Default, &Font::default());
        let measurer = CosmicTextMeasurer::default();
        let narrow = measurer.measure_single("iiiii", 14, &font).expect("Failed to measure text");
        let wide = measurer.measure_single("MMMMM", 14, &font).expect("Failed to measure text");
        assert_eq!(narrow, wide);
    }

    #[test]
    fn font_options_start_with_default_and_system() {
        let options = FontOptions::new(&IndicatorFont::Default);

        assert_eq!(options.labels.len(), options.values.len());
        assert_eq!(options.position(&IndicatorFont::Default.to_form_value()), Some(0));
        assert_eq!(options.position(&IndicatorFont::System.to_form_value()), Some(1));
        assert!(!options.labels.iter().any(|label| HIDDEN_FONT_FAMILIES.contains(&label.as_str())));
    }

    #[test]
    fn font_options_include_configured_font_when_not_installed() {
        let missing = IndicatorFont::Family("Not An Installed Font 1234".to_string());
        let options = FontOptions::new(&missing);

        let index = options.position(&missing.to_form_value()).expect("Configured font missing from options");
        assert_eq!(options.labels[index], "Not An Installed Font 1234");
    }

    #[test]
    fn named_fonts_resolve_to_their_family() {
        let font = resolve_font(&IndicatorFont::Family("Fira Sans".to_string()), &Font::default());
        assert_eq!(font.family, cosmic::iced::font::Family::Name("Fira Sans"));
        assert_eq!(font.weight, Weight::Normal);
    }

    #[test]
    fn system_font_resolves_to_the_interface_font() {
        let interface_font = Font {
            weight: Weight::Bold,
            ..Font::with_name("Open Sans")
        };
        assert_eq!(resolve_font(&IndicatorFont::System, &interface_font), interface_font);
    }
}
