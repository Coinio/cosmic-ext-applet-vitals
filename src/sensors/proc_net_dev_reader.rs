use crate::sensors::sensor_traits::SensorReader;
use std::fs::File;
use std::io::Read;
use std::path::Path;

const PROC_NET_DEV_FILE: &str = "/proc/net/dev";
const PROC_NET_DEV_DEVICE_NAME_INDEX: usize = 0;
const PROC_NET_DEV_RX_BYTES_INDEX: usize = 1;
const PROC_NET_DEV_TX_BYTES_INDEX: usize = 9;

#[derive(Clone, Debug)]
pub struct ProcNetDevStatus {
    pub device_statuses: Vec<ProcNetDevDeviceStatus>
}

impl ProcNetDevStatus {
    pub fn new(device_statuses: Vec<ProcNetDevDeviceStatus>) -> Self {
        Self {
            device_statuses
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProcNetDevDeviceStatus {
    pub device_name: String,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
}

impl ProcNetDevDeviceStatus {
    pub fn new(device_name: String, rx_bytes: u64, tx_bytes: u64) -> Self {
        Self {
            device_name,
            tx_bytes,
            rx_bytes,
        }
    }
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
            return Err(format!("Unable to read {}", PROC_NET_DEV_FILE))
        }

        let mut device_statuses = Vec::new();

        for line in contents.lines().skip(2) {
            let result = self.parse_proc_file_line(line);

            if let Ok(device_status) = result {
                device_statuses.push(device_status);
            } else {
                Err(result.unwrap_err())?
            }
        }

        Ok(ProcNetDevStatus::new(device_statuses))
    }

}

impl ProcNetDevReader {
    fn parse_proc_file_line(&self, line: &str) -> Result<ProcNetDevDeviceStatus, String> {

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 17 {
            Err(format!("Invalid file on line: {}", line))?
        }

        let device_name = parts[PROC_NET_DEV_DEVICE_NAME_INDEX].replace(":", "");

        let rx_bytes = parts[PROC_NET_DEV_TX_BYTES_INDEX].parse::<u64>().unwrap_or_default();
        let tx_bytes = parts[PROC_NET_DEV_RX_BYTES_INDEX].parse::<u64>().unwrap_or_default();

        Ok(ProcNetDevDeviceStatus::new(device_name, rx_bytes, tx_bytes))
    }
}