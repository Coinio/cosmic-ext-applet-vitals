use cosmic::iced;
use cosmic::iced::advanced::graphics::text::cosmic_text;
use cosmic::iced::advanced::graphics::text::cosmic_text::{Attrs, Buffer, Metrics, Shaping, SwashCache};
use cosmic::iced::advanced::graphics::text::font_system;

/// Uses iced's global font system so measurements match rendering.
#[derive(Default)]
pub struct CosmicTextMeasurer;

/// This was mostly taken from this repo: https://github.com/cosmic-utils/minimon-applet/
/// I really struggled to figure this out, gave up and found this so all credit goes to them.
impl CosmicTextMeasurer {
    pub fn measure_single(&self, text: &str, font_size: u16, font: &iced::Font) -> Option<f32> {
        let metrics = Metrics::new(font_size.into(), font_size.into());

        let mut font_system = font_system().write().ok()?;
        let font_system = font_system.raw();

        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_text(text, &attrs(font), Shaping::Advanced, None);
        buffer.shape_until_scroll(font_system, false);

        buffer
            .lines
            .first()
            .and_then(|line| line.layout_opt())
            .and_then(|layouts| layouts.first().map(|layout| layout.w.ceil() + 2.0))
    }

    /// The vertical centre of the inked pixels of `text`, in pixels down from the top of a line box `line_height`
    /// tall. The glyphs are rasterised and snapped to whole pixels the same way glyphon does when drawing
    /// (baseline at `round(line_y)`), so this reflects the font's real glyph shapes and the renderer's rounding,
    /// which font metrics alone don't. Assumes the line box starts on a whole pixel at a scale factor of 1.
    pub fn measure_ink_centre(&self, text: &str, font_size: u16, line_height: u16, font: &iced::Font) -> Option<f32> {
        let metrics = Metrics::new(font_size.into(), line_height.into());

        let mut font_system = font_system().write().ok()?;
        let font_system = font_system.raw();

        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_text(text, &attrs(font), Shaping::Advanced, None);
        buffer.shape_until_scroll(font_system, false);

        let mut swash_cache = SwashCache::new();
        let mut ink: Option<(i32, i32)> = None;
        for run in buffer.layout_runs() {
            let baseline = run.line_y.round() as i32;
            for glyph in run.glyphs {
                let physical = glyph.physical((0.0, 0.0), 1.0);
                let Some(image) = swash_cache.get_image_uncached(font_system, physical.cache_key) else {
                    continue;
                };
                if image.placement.height == 0 {
                    continue;
                }
                let top = baseline + physical.y - image.placement.top;
                let bottom = top + image.placement.height as i32;
                ink = Some(ink.map_or((top, bottom), |(t, b)| (t.min(top), b.max(bottom))));
            }
        }

        ink.map(|(top, bottom)| (top + bottom) as f32 / 2.0)
    }
}

/// The cosmic-text attributes matching an iced font.
fn attrs(font: &iced::Font) -> Attrs<'static> {
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

    Attrs::new().family(family).weight(weight).style(style)
}
