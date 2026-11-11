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
    /// The vertical centre of the rendered digits, keyed by font size, line height and font.
    digit_centres: std::cell::RefCell<HashMap<(u16, u16, Font), Option<f32>>>,
}

/// The line height of indicator text for a font size. It is set explicitly, rather than left to the applet's text
/// presets, so the line box that icons are aligned within is known and matches what is measured.
pub fn indicator_line_height(font_size: u16) -> u16 {
    (font_size as f32 * 1.2).ceil() as u16
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

    /// The vertical centre of the digits as rendered, in pixels from the top of their line box. Icons are aligned to
    /// this rather than to the line box centre, as where the digits sit in the line box varies between fonts and with
    /// the renderer's pixel snapping.
    pub fn digit_centre(&self, font_size: u16, line_height: u16, font: &Font) -> Option<f32> {
        let key = (font_size, line_height, *font);

        if let Some(value) = self.digit_centres.borrow().get(&key).copied() {
            return value;
        }

        let centre = self
            .cosmic_text_measurer
            .measure_ink_centre("0123456789", font_size, line_height, font);
        self.digit_centres.borrow_mut().insert(key, centre);

        centre
    }

    pub fn reset(&self) {
        self.text_measurements.borrow_mut().clear();
        self.digit_centres.borrow_mut().clear();
    }
}
