use cosmic::iced;
use cosmic::iced_renderer::graphics::text::cosmic_text;
use cosmic::iced_renderer::graphics::text::cosmic_text::{Attrs, Buffer, Metrics, Shaping};
use cosmic::iced_renderer::graphics::text::font_system;

/// Uses iced's global font system so measurements match rendering and include the bundled font.
#[derive(Default)]
pub struct CosmicTextMeasurer;

/// This was mostly taken from this repo: https://github.com/cosmic-utils/minimon-applet/
/// I really struggled to figure this out, gave up and found this so all credit goes to them.
impl CosmicTextMeasurer {
    pub fn measure_single(&self, text: &str, font_size: u16, font: &iced::Font) -> Option<f32> {
        let family = match font.family {
            iced::font::Family::Monospace => cosmic_text::Family::Monospace,
            iced::font::Family::Serif => cosmic_text::Family::Serif,
            iced::font::Family::SansSerif => cosmic_text::Family::SansSerif,
            iced::font::Family::Name(name) => cosmic_text::Family::Name(name),
            iced::font::Family::Cursive => cosmic_text::Family::Cursive,
            iced::font::Family::Fantasy => cosmic_text::Family::Fantasy,
        };

        let weight = match font.weight {
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

        let style = match font.style {
            iced::font::Style::Normal => cosmic_text::Style::Normal,
            iced::font::Style::Italic => cosmic_text::Style::Italic,
            iced::font::Style::Oblique => cosmic_text::Style::Oblique,
        };

        let attrs = Attrs::new().family(family).weight(weight).style(style);

        self.measure_single_line_text(text, font_size, &attrs)
    }

    fn measure_single_line_text(&self, text: &str, font_size: u16, attrs: &Attrs) -> Option<f32> {
        let metrics = Metrics::new(font_size.into(), font_size.into());

        let mut font_system = font_system().write().ok()?;
        let font_system = font_system.raw();

        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_text(font_system, text, attrs, Shaping::Advanced, None);

        buffer
            .lines
            .first()
            .and_then(|line| line.layout_opt())
            .and_then(|layouts| layouts.first().map(|layout| layout.w.ceil() + 2.0))
    }
}
