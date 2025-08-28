use std::fs;

const SYS_CLASS_NET_PATH: &str = "/sys/class/net";

pub fn is_physical_interface(device_name: &str) -> bool {
    let device_file_path = format!("{}/{}/device", SYS_CLASS_NET_PATH, device_name);

    if let Ok(value) = fs::exists(device_file_path) {
        value
    } else {
        false
    }
}
