use once_cell::sync::Lazy;
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
struct Icons;

fn svg_handle(name: &str) -> cosmic::widget::icon::Handle {
    let bytes = Icons::get(name).expect(format!("Embedded icon {} missing", name).as_str());
    cosmic::widget::icon::from_svg_bytes(bytes.data)
}

pub static APP_ICONS: Lazy<HashMap<&'static str, cosmic::widget::icon::Handle>> = Lazy::new(|| {
    let mut cache = HashMap::new();

    cache.insert(APP_LOGO_ICON, svg_handle("dev.eidolon.cosmic-vitals-applet.svg").into());

    cache.insert(CPU_USAGE_ICON_DARK_KEY, svg_handle("cpu.svg").into());
    cache.insert(
        MEMORY_USAGE_ICON_DARK_KEY,
        svg_handle("memory.svg").into(),
    );

    cache.insert(
        NETWORK_RX_USAGE_ICON_DARK_KEY,
        svg_handle("arrow-download.svg").into(),
    );

    cache.insert(
        NETWORK_TX_USAGE_ICON_DARK_KEY,
        svg_handle("arrow-upload.svg").into(),
    );

    cache.insert(
        DISK_READ_ICON_DARK_KEY,
        svg_handle("disk-download.svg").into(),
    );

    cache.insert(
        DISK_WRITE_ICON_DARK_KEY,
        svg_handle("disk-upload.svg").into(),
    );

    cache
});
