use crate::app::GLOBAL_APP_ID;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

pub const APP_LOGO_ICON: &str = "app-logo-icon";
pub const CPU_USAGE_ICON_DARK_KEY: &str = "cpu-usage-icon-dark";
pub const CPU_USAGE_ICON_LIGHT_KEY: &str = "cpu-usage-icon-light";
pub const MEMORY_USAGE_ICON_DARK_KEY: &str = "memory-usage-icon-dark";
pub const MEMORY_USAGE_ICON_LIGHT_KEY: &str = "memory-usage-icon-light";
pub const NETWORK_RX_USAGE_ICON_DARK_KEY: &str = "network-rx-usage-icon-dark";
pub const NETWORK_RX_USAGE_ICON_LIGHT_KEY: &str = "network-rx-usage-icon-light";
pub const NETWORK_TX_USAGE_ICON_DARK_KEY: &str = "network-tx-usage-icon-dark";
pub const NETWORK_TX_USAGE_ICON_LIGHT_KEY: &str = "network-tx-usage-icon-light";
pub const DISK_READ_ICON_DARK_KEY: &str = "disk-read-usage-icon-dark";
pub const DISK_READ_ICON_LIGHT_KEY: &str = "disk-read-usage-icon-light";
pub const DISK_WRITE_ICON_DARK_KEY: &str = "disk-write-usage-icon-dark";
pub const DISK_WRITE_ICON_LIGHT_KEY: &str = "disk-write-usage-icon-light";

fn icon_base_dir() -> PathBuf {
    // In debug, look in res/icons
    if cfg!(debug_assertions) {
        return PathBuf::from("res/icons");
    }

    // Candidate data dirs: $XDG_DATA_HOME, $XDG_DATA_DIRS, plus common defaults
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(xdg_dirs) = env::var("XDG_DATA_DIRS") {
        for path in xdg_dirs.split(':') {
            if !path.is_empty() {
                candidates.push(PathBuf::from(path));
            }
        }
    } else {
        candidates.push(PathBuf::from("/usr/local/share"));
        candidates.push(PathBuf::from("/usr/share"));
    }

    for base in candidates {
        let path = base.join(GLOBAL_APP_ID).join("icons");
        if path.exists() {
            return path;
        }
    }

    // Fallback
    PathBuf::from("res/icons")
}

pub static ICONS: Lazy<HashMap<&'static str, cosmic::widget::icon::Handle>> = Lazy::new(|| {
    let mut cache = HashMap::new();
    let base = icon_base_dir();

    let path = |name: &str| base.join(name).to_string_lossy().to_string();

    cache.insert(
        APP_LOGO_ICON,
        cosmic::widget::icon::from_path(path("dev.eidolon.cosmic-vitals-applet.svg").into()).into(),
    );

    cache.insert(
        CPU_USAGE_ICON_DARK_KEY,
        cosmic::widget::icon::from_path(path("cpu-svgrepo-com-dark.svg").into()).into(),
    );
    cache.insert(
        CPU_USAGE_ICON_LIGHT_KEY,
        cosmic::widget::icon::from_path(path("cpu-svgrepo-com-light.svg").into()).into(),
    );
    cache.insert(
        MEMORY_USAGE_ICON_DARK_KEY,
        cosmic::widget::icon::from_path(path("ram-memory-svgrepo-com-dark.svg").into()).into(),
    );
    cache.insert(
        MEMORY_USAGE_ICON_LIGHT_KEY,
        cosmic::widget::icon::from_path(path("ram-memory-svgrepo-com-light.svg").into()).into(),
    );
    cache.insert(
        NETWORK_RX_USAGE_ICON_DARK_KEY,
        cosmic::widget::icon::from_path(path("arrow-down-svgrepo-com-dark.svg").into()).into(),
    );
    cache.insert(
        NETWORK_RX_USAGE_ICON_LIGHT_KEY,
        cosmic::widget::icon::from_path(path("arrow-down-svgrepo-com-light.svg").into()).into(),
    );
    cache.insert(
        NETWORK_TX_USAGE_ICON_DARK_KEY,
        cosmic::widget::icon::from_path(path("arrow-up-svgrepo-com-dark.svg").into()).into(),
    );
    cache.insert(
        NETWORK_TX_USAGE_ICON_LIGHT_KEY,
        cosmic::widget::icon::from_path(path("arrow-up-svgrepo-com-light.svg").into()).into(),
    );
    cache.insert(
        DISK_READ_ICON_DARK_KEY,
        cosmic::widget::icon::from_path(path("disk-download-svgrepo-com-dark.svg").into()).into(),
    );
    cache.insert(
        DISK_READ_ICON_LIGHT_KEY,
        cosmic::widget::icon::from_path(path("disk-download-svgrepo-com-light.svg").into()).into(),
    );
    cache.insert(
        DISK_WRITE_ICON_DARK_KEY,
        cosmic::widget::icon::from_path(path("disk-upload-svgrepo-com-dark.svg").into()).into(),
    );
    cache.insert(
        DISK_WRITE_ICON_LIGHT_KEY,
        cosmic::widget::icon::from_path(path("disk-upload-svgrepo-com-light.svg").into()).into(),
    );

    cache
});
