use rust_embed::RustEmbed;
use std::collections::HashMap;

pub const APP_LOGO_ICON: &str = "app-logo-icon";
pub const UP_ARROW_ICON: &str = "up-arrow-icon";
pub const DOWN_ARROW_ICON: &str = "down-arrow-icon";
pub const READ_ICON: &str = "read-icon";
pub const WRITE_ICON: &str = "write-icon";

#[derive(RustEmbed)]
#[folder = "res/icons/"]
struct EmbeddedIcons;

fn svg_handle(name: &str) -> cosmic::widget::icon::Handle {
    let bytes = EmbeddedIcons::get(name).expect(format!("Embedded icon {} missing", name).as_str());
    cosmic::widget::icon::from_svg_bytes(bytes.data)
}

#[derive(Default)]
pub struct AppIcons {
    icons: HashMap<&'static str, cosmic::widget::icon::Handle>
}

impl AppIcons {

    pub fn new() -> Self {
        Self {
            icons: HashMap::from([
                (APP_LOGO_ICON, svg_handle("dev.eidolon.cosmic-vitals-applet.svg").into()),
                (DOWN_ARROW_ICON, svg_handle("arrow-download.svg").into()),
                (UP_ARROW_ICON, svg_handle("arrow-upload.svg").into()),
                (READ_ICON, svg_handle("read.svg").into()),
                (WRITE_ICON, svg_handle("write.svg").into()),
            ])
        }
    }
    pub fn get(&self, key: &str) -> Option<&cosmic::widget::icon::Handle> {
        self.icons.get(key)
    }
}
