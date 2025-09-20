use cosmic::config::COSMIC_TK;
use cosmic::iced;
use cosmic::iced_renderer::graphics::text::cosmic_text;
use cosmic::iced_renderer::graphics::text::cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping};
use std::cell::RefCell;

pub struct CosmicTextMeasurer {
    /// The font system used by the application, used for measuring label text.
    font_system: RefCell<FontSystem>,
}

impl Default for CosmicTextMeasurer {
    fn default() -> Self {
        Self::new()
    }
}

/// This was mostly taken from this repo: https://github.com/cosmic-utils/minimon-applet/
/// I really struggled to figure this out, gave up and found this so all credit goes to them.
impl CosmicTextMeasurer {
    pub fn new() -> Self {
        Self {
            font_system: RefCell::new(FontSystem::new()),
        }
    }

    ///
    pub fn measure_single(&self, text: &str, font_size: u16) -> Option<f32> {
        // TODO: Look at this unwrap. Is it safe?
        let interface_font = COSMIC_TK.read().unwrap().interface_font.clone();

        let active_font: iced::Font = interface_font.into();

        let family = match active_font.family {
            iced::font::Family::Monospace => cosmic_text::Family::Monospace,
            iced::font::Family::Serif => cosmic_text::Family::Serif,
            iced::font::Family::SansSerif => cosmic_text::Family::SansSerif,
            iced::font::Family::Name(name) => cosmic_text::Family::Name(name),
            iced::font::Family::Cursive => cosmic_text::Family::Cursive,
            iced::font::Family::Fantasy => cosmic_text::Family::Fantasy,
        };

        let weight = match active_font.weight {
            iced::font::Weight::Thin => cosmic_text::Weight::THIN,
            iced::font::Weight::ExtraLight => cosmic_text::Weight::EXTRA_LIGHT,
            iced::font::Weight::Light => cosmic_text::Weight::LIGHT,
            iced::font::Weight::Normal => cosmic_text::Weight::NORMAL,
            iced::font::Weight::Medium => cosmic_text::Weight::MEDIUM,
            iced::font::Weight::Bold => cosmic_text::Weight::BOLD,
            iced::font::Weight::ExtraBold => cosmic_text::Weight::EXTRA_BOLD,
            iced::font::Weight::Black => cosmic_text::Weight::BLACK,
            iced::font::Weight::Semibold => cosmic_text::Weight::SEMIBOLD,
        };

        let style = match active_font.style {
            iced::font::Style::Normal => cosmic_text::Style::Normal,
            iced::font::Style::Italic => cosmic_text::Style::Italic,
            iced::font::Style::Oblique => cosmic_text::Style::Oblique,
        };

        let attrs = Attrs::new().family(family).weight(weight).style(style);

        self.measure_single_line_text(text, font_size, &attrs)
    }

    fn measure_single_line_text(&self, text: &str, font_size: u16, attrs: &Attrs) -> Option<f32> {
        let metrics = Metrics::new(font_size.into(), font_size.into());

        let mut font_system = self.font_system.borrow_mut();

        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_text(&mut font_system, text, attrs, Shaping::Advanced);

        buffer
            .lines
            .first()
            .and_then(|line| line.layout_opt())
            .and_then(|layouts| layouts.first().map(|layout| layout.w.ceil() + 2.0))
    }
}
