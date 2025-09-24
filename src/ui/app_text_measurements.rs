use crate::ui::cosmic_text_measurer::CosmicTextMeasurer;
use std::collections::HashMap;

/// Contains the current text measurements for the indicator labels. This is essentially a
/// cache for label widths so they aren't measured every render.
#[derive(Default)]
pub struct AppTextMeasurements {
    /// The cosmic text measurer that is used to measure text as rendered
    cosmic_text_measurer: CosmicTextMeasurer,
    /// The text measurements for the labels, keyed by the label text.
    text_measurements: std::cell::RefCell<HashMap<(&'static str, u16), f32>>,

}

impl AppTextMeasurements {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn measure(&self, text: &'static str, font_size: u16) -> Option<f32> {

        if let Some(value) = self.text_measurements.borrow().get(&(text, font_size)).copied() {
            return Some(value);
        }

        let new_measurement = self
            .cosmic_text_measurer
            .measure_single(text, font_size)
            .unwrap_or_default();

        if new_measurement <= 0.0 {
            return None;
        }

        self.text_measurements
            .borrow_mut()
            .insert((text, font_size), new_measurement);

        Some(new_measurement)

    }

    pub fn reset(&self) {
        self.text_measurements.borrow_mut().clear();       
    }
}
