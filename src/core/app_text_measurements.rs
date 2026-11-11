use crate::core::cosmic_text_measurer::CosmicTextMeasurer;
use cosmic::font::Font;
use std::collections::HashMap;

/// Contains the current text measurements for the indicator labels. This is essentially a
/// cache for label widths so they aren't measured every render.
#[derive(Default)]
pub struct AppTextMeasurements {
    /// The cosmic text measurer that is used to measure text as rendered
    cosmic_text_measurer: CosmicTextMeasurer,
    /// The text measurements for the labels, keyed by the label text, font size and font.
    text_measurements: std::cell::RefCell<HashMap<(&'static str, u16, Font), f32>>,
}

impl AppTextMeasurements {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn measure(&self, text: &'static str, font_size: u16, font: &Font) -> Option<f32> {
        let key = (text, font_size, *font);

        if let Some(value) = self.text_measurements.borrow().get(&key).copied() {
            return Some(value);
        }

        let new_measurement = self
            .cosmic_text_measurer
            .measure_single(text, font_size, font)
            .unwrap_or_default();

        if new_measurement <= 0.0 {
            return None;
        }

        self.text_measurements
            .borrow_mut()
            .insert(key, new_measurement);

        Some(new_measurement)

    }

    pub fn reset(&self) {
        self.text_measurements.borrow_mut().clear();       
    }
}
