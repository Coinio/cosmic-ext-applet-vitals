use std::collections::HashMap;
use once_cell::sync::Lazy;

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

pub static ICONS: Lazy<HashMap<&'static str, cosmic::widget::icon::Handle>> = Lazy::new(|| {
    let mut cache = HashMap::new();

    cache.insert(
        CPU_USAGE_ICON_DARK_KEY,
        cosmic::widget::icon::from_path("res/icons/arrow-up-svgrepo-com-light.svg".into()).into(),
    );
    cache.insert(
        CPU_USAGE_ICON_LIGHT_KEY,
        cosmic::widget::icon::from_path("res/icons/cpu-svgrepo-com-light.svg".into()).into(),
    );
    cache.insert(
        MEMORY_USAGE_ICON_DARK_KEY,
        cosmic::widget::icon::from_path("res/icons/ram-memory-svgrepo-com-dark.svg".into()).into(),
    );
    cache.insert(
        MEMORY_USAGE_ICON_LIGHT_KEY,
        cosmic::widget::icon::from_path("res/icons/ram-memory-svgrepo-com-light.svg".into()).into(),
    );
    cache.insert(
        NETWORK_RX_USAGE_ICON_DARK_KEY,
        cosmic::widget::icon::from_path("res/icons/arrow-down-svgrepo-com-dark.svg".into()).into(),
    );
    cache.insert(
        NETWORK_RX_USAGE_ICON_LIGHT_KEY,
        cosmic::widget::icon::from_path("res/icons/arrow-down-svgrepo-com-light.svg".into()).into(),
    );
    cache.insert(
        NETWORK_TX_USAGE_ICON_DARK_KEY,
        cosmic::widget::icon::from_path("res/icons/arrow-up-svgrepo-com-dark.svg".into()).into(),
    );
    cache.insert(
        NETWORK_TX_USAGE_ICON_LIGHT_KEY,
        cosmic::widget::icon::from_path("res/icons/arrow-up-svgrepo-com-light.svg".into()).into(),
    );
    cache.insert(
        DISK_READ_ICON_DARK_KEY,
        cosmic::widget::icon::from_path("res/icons/disk-download-svgrepo-com-dark.svg".into()).into(),
    );
    cache.insert(
        DISK_READ_ICON_LIGHT_KEY,
        cosmic::widget::icon::from_path("res/icons/disk-download-svgrepo-com-light.svg".into()).into(),
    );
    cache.insert(
        DISK_WRITE_ICON_DARK_KEY,
        cosmic::widget::icon::from_path("res/icons/disk-upload-svgrepo-com-dark.svg".into()).into(),
    );
    cache.insert(
        DISK_WRITE_ICON_LIGHT_KEY,
        cosmic::widget::icon::from_path("res/icons/disk-upload-svgrepo-com-light.svg".into()).into(),
    );

    cache
});
