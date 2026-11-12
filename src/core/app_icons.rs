use rust_embed::RustEmbed;
use std::collections::HashMap;

pub const APP_LOGO_ICON: &str = "app-logo-icon";
pub const UP_ARROW_ICON: &str = "up-arrow-icon";
pub const DOWN_ARROW_ICON: &str = "down-arrow-icon";
pub const READ_ICON: &str = "read-icon";
pub const WRITE_ICON: &str = "write-icon";
pub const CPU_ICON: &str = "cpu-icon";
pub const MEMORY_ICON: &str = "memory-icon";
pub const STORAGE_ICON: &str = "storage-icon";

#[derive(RustEmbed)]
#[folder = "res/icons/"]
struct EmbeddedIcons;

/// An embedded icon. The indicator icons' viewBoxes are cropped to their drawing plus the same small margin, so
/// they're narrower than they are tall and every icon sits the same visual distance from its neighbours.
pub struct AppIcon {
    pub handle: cosmic::widget::icon::Handle,
    /// Width / height, from the icon's viewBox.
    pub aspect_ratio: f32,
}

impl AppIcon {
    fn embedded(name: &str) -> Self {
        let bytes = EmbeddedIcons::get(name).expect(format!("Embedded icon {} missing", name).as_str());
        let aspect_ratio = std::str::from_utf8(&bytes.data)
            .ok()
            .and_then(view_box_aspect_ratio)
            .unwrap_or(1.0);

        Self {
            handle: cosmic::widget::icon::from_svg_bytes(bytes.data),
            aspect_ratio,
        }
    }

    /// The width to draw the icon at for a given height.
    pub fn width(&self, height: u16) -> u16 {
        ((f32::from(height) * self.aspect_ratio).round() as u16).max(1)
    }
}

fn view_box_aspect_ratio(svg: &str) -> Option<f32> {
    let start = svg.find("viewBox=\"")? + "viewBox=\"".len();
    let end = start + svg[start..].find('"')?;
    let values: Vec<f32> = svg[start..end]
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|value| !value.is_empty())
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;

    match values[..] {
        [_, _, width, height] if width > 0.0 && height > 0.0 => Some(width / height),
        _ => None,
    }
}

#[derive(Default)]
pub struct AppIcons {
    icons: HashMap<&'static str, AppIcon>,
}

impl AppIcons {
    pub fn new() -> Self {
        Self {
            icons: HashMap::from([
                (APP_LOGO_ICON, AppIcon::embedded("dev.eidolon.cosmic-ext-applet-vitals.svg")),
                (DOWN_ARROW_ICON, AppIcon::embedded("arrow-down.svg")),
                (UP_ARROW_ICON, AppIcon::embedded("arrow-up.svg")),
                (READ_ICON, AppIcon::embedded("read.svg")),
                (WRITE_ICON, AppIcon::embedded("write.svg")),
                (CPU_ICON, AppIcon::embedded("cpu.svg")),
                (MEMORY_ICON, AppIcon::embedded("memory.svg")),
                (STORAGE_ICON, AppIcon::embedded("storage.svg")),
            ]),
        }
    }

    pub fn get(&self, key: &str) -> Option<&AppIcon> {
        self.icons.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::{view_box_aspect_ratio, AppIcons, APP_LOGO_ICON, CPU_ICON, MEMORY_ICON, READ_ICON};

    #[test]
    fn aspect_ratio_comes_from_the_view_box() {
        assert_eq!(view_box_aspect_ratio(r#"<svg viewBox="3.5 0 9.3 16">"#), Some(9.3 / 16.0));
        assert_eq!(view_box_aspect_ratio(r#"<svg viewBox="0,0,128,128">"#), Some(1.0));
        assert_eq!(view_box_aspect_ratio(r#"<svg width="16">"#), None);
        assert_eq!(view_box_aspect_ratio(r#"<svg viewBox="0 0 0 16">"#), None);
    }

    #[test]
    fn icons_are_drawn_at_their_view_box_width() {
        let icons = AppIcons::new();
        assert_eq!(icons.get(APP_LOGO_ICON).unwrap().width(24), 24);
        assert_eq!(icons.get(CPU_ICON).unwrap().width(16), 16);
        assert_eq!(icons.get(MEMORY_ICON).unwrap().width(16), 18);
        assert!(icons.get(READ_ICON).unwrap().width(16) < 16);
    }
}
