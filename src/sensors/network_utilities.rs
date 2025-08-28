use std::fs;

const SYS_CLASS_NET_PATH: &str = "/sys/class/net";

pub trait NetworkUtilsTrait {
    fn is_physical_interface(&self, device_name: &str) -> bool;
}

pub struct NetworkUtils;

impl NetworkUtils {
    pub fn new() -> Self {
        Self {}
    }
}

impl NetworkUtilsTrait for NetworkUtils {
    fn is_physical_interface(&self, device_name: &str) -> bool {
        let device_file_path = format!("{}/{}/device", SYS_CLASS_NET_PATH, device_name);

        if let Ok(value) = fs::exists(device_file_path) {
            value
        } else {
            false
        }
    }
}
