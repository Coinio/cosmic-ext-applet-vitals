use std::collections::HashMap;
use std::ffi::CStr;
use std::fs;
use std::net::Ipv4Addr;

const SYS_CLASS_NET_PATH: &str = "/sys/class/net";

pub fn is_physical_interface(device_name: &str) -> bool {
    let device_file_path = format!("{}/{}/device", SYS_CLASS_NET_PATH, device_name);

    if let Ok(value) = fs::exists(device_file_path) {
        value
    } else {
        false
    }
}

/// Whether the interface's link is up, e.g. a cable is plugged in or Wi-Fi is connected. Drivers that don't report
/// their link state are taken to be up.
pub fn is_interface_up(device_name: &str) -> bool {
    let operstate_path = format!("{}/{}/operstate", SYS_CLASS_NET_PATH, device_name);

    match fs::read_to_string(operstate_path) {
        Ok(state) => matches!(state.trim(), "up" | "unknown"),
        Err(_) => false,
    }
}

/// The first IPv4 address of each interface that has one, by interface name.
pub fn ipv4_addresses() -> HashMap<String, Ipv4Addr> {
    let mut addresses = HashMap::new();
    let mut list: *mut libc::ifaddrs = std::ptr::null_mut();

    // SAFETY: getifaddrs only writes the list pointer, which is freed below with freeifaddrs.
    if unsafe { libc::getifaddrs(&mut list) } != 0 {
        return addresses;
    }

    let mut entry = list;
    while !entry.is_null() {
        // SAFETY: entry is a non-null node of the list getifaddrs returned, which stays valid until it's freed.
        let interface = unsafe { &*entry };
        entry = interface.ifa_next;

        if interface.ifa_addr.is_null() || interface.ifa_name.is_null() {
            continue;
        }
        // SAFETY: ifa_addr is non-null, and every address starts with its family.
        if i32::from(unsafe { (*interface.ifa_addr).sa_family }) != libc::AF_INET {
            continue;
        }

        // SAFETY: an AF_INET address is a sockaddr_in, and ifa_name is a non-null, nul terminated string.
        let address = unsafe { &*(interface.ifa_addr as *const libc::sockaddr_in) };
        let name = unsafe { CStr::from_ptr(interface.ifa_name) }.to_string_lossy().into_owned();

        addresses
            .entry(name)
            .or_insert_with(|| Ipv4Addr::from(u32::from_be(address.sin_addr.s_addr)));
    }

    // SAFETY: list came from a successful getifaddrs and isn't used after this.
    unsafe { libc::freeifaddrs(list) };

    addresses
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks the address is read in the right byte order. Some build sandboxes leave loopback without an address,
    /// so it's only checked when there is one.
    #[test]
    fn loopback_address_is_a_loopback_address() {
        if let Some(address) = ipv4_addresses().get("lo") {
            assert!(address.is_loopback(), "{address} isn't a loopback address");
        }
    }
}
