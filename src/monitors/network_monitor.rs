use crate::configuration::app_configuration::AppConfiguration;
use crate::sensors::proc_net_dev_reader::ProcNetDevStatus;
use crate::sensors::sensor_traits::SensorReader;
use log::info;
use std::collections::VecDeque;
use std::{cmp, fs};

const SYS_CLASS_NET_PATH: &str = "/sys/class/net";

#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

impl NetworkStats {
    pub fn new(rx_bytes: u64, tx_bytes: u64) -> Self {
        Self { rx_bytes, tx_bytes }
    }
}

pub struct NetworkMonitor<S: SensorReader<Output = ProcNetDevStatus>> {
    sensor_reader: S,
    sample_buffer: VecDeque<NetworkStats>,
    previous_rx_bytes: u64,
    previous_tx_bytes: u64,
    max_samples: usize,
}

impl<S: SensorReader<Output = ProcNetDevStatus>> NetworkMonitor<S> {
    pub fn new(sensor_reader: S, configuration: &AppConfiguration) -> Self {
        info!("Creating new network monitor {:?}", configuration);
        Self {
            sensor_reader,
            sample_buffer: VecDeque::with_capacity(configuration.memory.max_samples),
            previous_rx_bytes: 0,
            previous_tx_bytes: 0,
            max_samples: configuration.memory.max_samples,
        }
    }

    pub fn poll(&mut self) -> Result<NetworkStats, String> {
        let current = match self.sensor_reader.read() {
            Ok(value) => value,
            Err(err) => return Err(err),
        };

        let mut current_rx_total = 0u64;
        let mut current_tx_total = 0u64;

        for device_status in current.device_statuses {
            let device_file_path = format!("{}/{}/device", SYS_CLASS_NET_PATH, device_status.device_name);

            // TODO: Is it worth caching these checks?
            let device_file_exists = if let Ok(value) = fs::exists(device_file_path) {
                value
            } else {
                false
            };

            if !device_file_exists {
                continue;
            }

            current_rx_total += device_status.rx_bytes;
            current_tx_total += device_status.tx_bytes;
        }

        let delta_rx = cmp::max(current_rx_total - self.previous_rx_bytes, 0);
        let delta_tx = cmp::max(current_tx_total - self.previous_tx_bytes, 0);

        self.previous_rx_bytes = current_rx_total;
        self.previous_tx_bytes = current_tx_total;

        self.sample_buffer
            .push_back(NetworkStats::new(delta_rx, delta_tx));

        if self.sample_buffer.len() > self.max_samples {
            self.sample_buffer.pop_front();
        };

        let average_rx_bytes =
            self.sample_buffer.iter()
                .map(|sample| sample.rx_bytes).sum::<u64>() / self.sample_buffer.len() as u64;

        let average_tx_bytes =
            self.sample_buffer.iter()
                .map(|sample| sample.tx_bytes).sum::<u64>() / self.sample_buffer.len() as u64;

        Ok(NetworkStats::new(average_rx_bytes, average_tx_bytes))

    }
}
