use rust_embed::RustEmbed;
use std::collections::HashMap;

pub const APP_LOGO_ICON: &str = "app-logo-icon";
pub const CPU_USAGE_ICON_DARK_KEY: &str = "cpu-usage-icon-dark";
pub const MEMORY_USAGE_ICON_DARK_KEY: &str = "memory-usage-icon-dark";
pub const NETWORK_RX_USAGE_ICON_DARK_KEY: &str = "network-rx-usage-icon-dark";
pub const NETWORK_TX_USAGE_ICON_DARK_KEY: &str = "network-tx-usage-icon-dark";
pub const DISK_READ_ICON_DARK_KEY: &str = "disk-read-usage-icon-dark";
pub const DISK_WRITE_ICON_DARK_KEY: &str = "disk-write-usage-icon-dark";

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
                (CPU_USAGE_ICON_DARK_KEY, svg_handle("cpu.svg").into()),
                (MEMORY_USAGE_ICON_DARK_KEY, svg_handle("memory.svg").into()),
                (NETWORK_RX_USAGE_ICON_DARK_KEY, svg_handle("arrow-download.svg").into()),
                (NETWORK_TX_USAGE_ICON_DARK_KEY, svg_handle("arrow-upload.svg").into()),
                (DISK_READ_ICON_DARK_KEY, svg_handle("disk-download.svg").into()),
                (DISK_WRITE_ICON_DARK_KEY, svg_handle("disk-upload.svg").into()),
            ])
        }
    }
    pub fn get(&self, key: &str) -> Option<&cosmic::widget::icon::Handle> {
        self.icons.get(key)
    }
}
