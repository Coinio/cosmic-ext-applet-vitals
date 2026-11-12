use crate::sensors::network_utilities::{ipv4_addresses, is_interface_up, is_physical_interface};
use crate::sensors::sensor_traits::SensorReader;
use std::fs::File;
use std::io::Read;
use std::net::Ipv4Addr;
use std::path::Path;

const PROC_NET_DEV_FILE: &str = "/proc/net/dev";
const PROC_NET_DEV_DEVICE_NAME_INDEX: usize = 0;
const PROC_NET_DEV_RX_BYTES_INDEX: usize = 1;
const PROC_NET_DEV_TX_BYTES_INDEX: usize = 9;

#[derive(Clone, Debug)]
pub struct ProcNetDevStatus {
    pub device_statuses: Vec<ProcNetDevDeviceStatus>,
}

impl ProcNetDevStatus {
    pub fn new(device_statuses: Vec<ProcNetDevDeviceStatus>) -> Self {
        Self { device_statuses }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProcNetDevDeviceStatus {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub is_physical_device: bool,
    /// Whether the link is up. Only read for physical devices.
    pub is_up: bool,
    /// The device's IPv4 address, if it has one. Only read for physical devices.
    pub ipv4: Option<Ipv4Addr>,
}

struct ProcNetDevLine {
    device_name: String,
    rx_bytes: u64,
    tx_bytes: u64,
}

pub struct ProcNetDevReader;

impl SensorReader for ProcNetDevReader {
    type Output = ProcNetDevStatus;

    fn read(&self) -> Result<Self::Output, String> {
        let path = Path::new(PROC_NET_DEV_FILE);

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(e) => return Err(format!("Unable to open file: {} {}", PROC_NET_DEV_FILE, e)),
        };

        let mut contents = String::new();

        if let Err(_) = file.read_to_string(&mut contents) {
            return Err(format!("Unable to read {}", PROC_NET_DEV_FILE));
        }

        let mut statuses = Vec::new();
        let mut addresses = None;

        for line in contents.lines().skip(2) {
            let line = self.parse_proc_file_line(line)?;
            let is_physical_device = is_physical_interface(&line.device_name);

            let (is_up, ipv4) = if is_physical_device {
                // Only looked up once there's a physical device to look it up for.
                let addresses = addresses.get_or_insert_with(ipv4_addresses);
                (is_interface_up(&line.device_name), addresses.get(&line.device_name).copied())
            } else {
                (false, None)
            };

            statuses.push(ProcNetDevDeviceStatus {
                name: line.device_name,
                rx_bytes: line.rx_bytes,
                tx_bytes: line.tx_bytes,
                is_physical_device,
                is_up,
                ipv4,
            });
        }

        Ok(ProcNetDevStatus::new(statuses))
    }
}

impl ProcNetDevReader {
    fn parse_proc_file_line(&self, line: &str) -> Result<ProcNetDevLine, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 17 {
            Err(format!("Invalid file on line: {}", line))?
        }

        let device_name = parts[PROC_NET_DEV_DEVICE_NAME_INDEX].replace(":", "");

        let rx_bytes = parts[PROC_NET_DEV_RX_BYTES_INDEX].parse::<u64>().unwrap_or_default();
        let tx_bytes = parts[PROC_NET_DEV_TX_BYTES_INDEX].parse::<u64>().unwrap_or_default();

        Ok(ProcNetDevLine {
            device_name,
            rx_bytes,
            tx_bytes,
        })
    }
}
