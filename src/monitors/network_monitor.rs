use crate::configuration::app_configuration::AppConfiguration;
use crate::sensors::network_utilities::{NetworkUtils, NetworkUtilsTrait};
use crate::sensors::proc_net_dev_reader::{ProcNetDevStatus};
use crate::sensors::sensor_traits::SensorReader;
use log::info;
use std::collections::VecDeque;
use std::{cmp};

pub const NETWORK_STAT_RX_INDEX: usize = 0;
pub const NETWORK_STAT_TX_INDEX: usize = 1;

#[derive(Debug, Clone, Default)]
struct NetworkSample {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

impl NetworkSample {
    pub fn new(rx_bytes: u64, tx_bytes: u64) -> Self {
        Self { rx_bytes, tx_bytes }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkDirection {
    Download,
    Upload,
}

impl Default for NetworkDirection {
    fn default() -> Self {
        Self::Download
    }
}

#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub bytes: u64,
    pub direction: NetworkDirection,
}

pub struct NetworkMonitor<S: SensorReader<Output = ProcNetDevStatus>, U: NetworkUtilsTrait> {
    sensor_reader: S,
    network_device: U,
    sample_buffer: VecDeque<NetworkSample>,
    previous_rx_bytes: u64,
    previous_tx_bytes: u64,
    max_samples: usize,
}

impl<S: SensorReader<Output = ProcNetDevStatus>, U: NetworkUtilsTrait> NetworkMonitor<S, U> {
    pub fn new(sensor_reader: S, network_device: U, configuration: &AppConfiguration) -> Self {
        info!("Creating new network monitor {:?}", configuration);
        Self {
            sensor_reader,
            network_device,
            sample_buffer: VecDeque::with_capacity(configuration.memory.max_samples),
            previous_rx_bytes: 0,
            previous_tx_bytes: 0,
            max_samples: configuration.memory.max_samples,
        }
    }

    pub fn poll(&mut self) -> Result<[NetworkStats; 2], String> {
        let current = match self.sensor_reader.read() {
            Ok(value) => value,
            Err(err) => return Err(err),
        };

        let mut current_rx_total = 0u64;
        let mut current_tx_total = 0u64;

        for device_status in current.device_statuses {
            if !self.network_device.is_physical_interface(&device_status.device_name) {
                continue;
            }

            current_rx_total += device_status.rx_bytes;
            current_tx_total += device_status.tx_bytes;
        }

        let delta_rx = cmp::max(current_rx_total - self.previous_rx_bytes, 0);
        let delta_tx = cmp::max(current_tx_total - self.previous_tx_bytes, 0);

        self.previous_rx_bytes = current_rx_total;
        self.previous_tx_bytes = current_tx_total;

        self.sample_buffer.push_back(NetworkSample::new(delta_rx, delta_tx));

        if self.sample_buffer.len() > self.max_samples {
            self.sample_buffer.pop_front();
        };

        let average_rx_bytes =
            self.sample_buffer.iter().map(|sample| sample.rx_bytes).sum::<u64>() / self.sample_buffer.len() as u64;

        let average_tx_bytes =
            self.sample_buffer.iter().map(|sample| sample.tx_bytes).sum::<u64>() / self.sample_buffer.len() as u64;

        let result = [
            NetworkStats {
                bytes: average_rx_bytes,
                direction: NetworkDirection::Download,
            },
            NetworkStats {
                bytes: average_tx_bytes,
                direction: NetworkDirection::Upload,
            },
        ];

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use super::*;
    use crate::sensors::proc_net_dev_reader::{ProcNetDevStatus, ProcNetDevDeviceStatus};

    struct MockProcNetDevReader {
        readings: Vec<Result<ProcNetDevStatus, String>>,
        index: Cell<usize>,
    }

    impl MockProcNetDevReader {
        fn new(readings: Vec<Result<ProcNetDevStatus, String>>) -> Self {
            Self { readings, index: Cell::new(0) }
        }
    }

    impl SensorReader for MockProcNetDevReader {
        type Output = ProcNetDevStatus;
        fn read(&self) -> Result<Self::Output, String> {
            let i = self.index.get();
            let result = self.readings[i].clone();
            self.index.set(i + 1);
            result
        }
    }

    struct MockNetworkUtilities {
        physical_devices: Vec<String>,
    }

    impl MockNetworkUtilities {
        fn new<T: Into<String>>(physical_devices: Vec<T>) -> Self {
            Self { physical_devices: physical_devices.into_iter().map(|s| s.into()).collect() }
        }
    }

    impl NetworkUtilsTrait for MockNetworkUtilities {
        fn is_physical_interface(&self, device_name: &str) -> bool {
            self.physical_devices.iter().any(|d| d == device_name)
        }
    }

    fn make_config(max_samples: usize) -> AppConfiguration {
        let mut cfg = AppConfiguration::default();
        cfg.memory.max_samples = max_samples;
        cfg
    }

    fn create_device_status(name: &str, rx: u64, tx: u64) -> ProcNetDevDeviceStatus {
        ProcNetDevDeviceStatus::new(name.to_string(), rx, tx)
    }

    #[test]
    fn single_poll_gives_expected_result() {
        // Add a physical and a virtual interface to the status. Virtual should be ignored.
        let status = ProcNetDevStatus::new(vec![
            create_device_status("eth0", 2000, 1000), // physical
            create_device_status("lo", 7000, 5000),   // virtual, ignored
        ]);

        let reader = MockProcNetDevReader::new(vec![Ok(status)]);
        let utils = MockNetworkUtilities::new(vec!["eth0"]);

        let mut monitor = NetworkMonitor::new(reader, utils, &make_config(3));
        let result = monitor.poll();

        assert!(result.is_ok());
        let arr = result.unwrap();

        assert_eq!(arr[NETWORK_STAT_RX_INDEX].bytes, 2000);
        assert_eq!(arr[NETWORK_STAT_RX_INDEX].direction, NetworkDirection::Download);
        assert_eq!(arr[NETWORK_STAT_TX_INDEX].bytes, 1000);
        assert_eq!(arr[NETWORK_STAT_TX_INDEX].direction, NetworkDirection::Upload);
    }

    #[test]
    fn multiple_poll_gives_expected_result() {
        // Add a physical and a virtual interface to the status. Virtual should be ignored.
        let sample1 = ProcNetDevStatus::new(vec![create_device_status("eth0", 2000, 1000),
                                            create_device_status("lo", 10, 10)]);
        let sample2 = ProcNetDevStatus::new(vec![create_device_status("eth0", 6000, 3000),
                                            create_device_status("lo", 20, 40)]);
        let sample3 = ProcNetDevStatus::new(vec![create_device_status("eth0", 9000, 6000),
                                            create_device_status("lo", 30, 70)]);

        let reader = MockProcNetDevReader::new(vec![Ok(sample1), Ok(sample2), Ok(sample3)]);
        let utils = MockNetworkUtilities::new(vec!["eth0"]);

        let mut monitor = NetworkMonitor::new(reader, utils, &make_config(3));

        let result1 = monitor.poll().unwrap();
        assert_eq!(result1[NETWORK_STAT_RX_INDEX].bytes, 2000);
        assert_eq!(result1[NETWORK_STAT_TX_INDEX].bytes, 1000);

        let result2 = monitor.poll().unwrap();
        assert_eq!(result2[NETWORK_STAT_RX_INDEX].bytes, (2000 + 4000) / 2);
        assert_eq!(result2[NETWORK_STAT_TX_INDEX].bytes, (1000 + 2000) / 2);

        let result3 = monitor.poll().unwrap();
        assert_eq!(result3[NETWORK_STAT_RX_INDEX].bytes, (2000 + 4000 + 3000) / 3);
        assert_eq!(result3[NETWORK_STAT_TX_INDEX].bytes, (1000 + 2000 + 3000) / 3);
    }

    #[test]
    fn samples_buffer_trims_to_max_size() {
        let sample1 = ProcNetDevStatus::new(vec![create_device_status("eth0", 2000, 1000)]);
        let sample2 = ProcNetDevStatus::new(vec![create_device_status("eth0", 5000, 3000)]);
        let sample3 = ProcNetDevStatus::new(vec![create_device_status("eth0", 9000, 4000)]);
        let sample4 = ProcNetDevStatus::new(vec![create_device_status("eth0", 12000, 6000)]);

        let reader = MockProcNetDevReader::new(vec![Ok(sample1), Ok(sample2), Ok(sample3), Ok(sample4)]);
        let utils = MockNetworkUtilities::new(vec!["eth0"]);

        let mut monitor = NetworkMonitor::new(reader, utils, &make_config(2));

        // Throw away first two samples
        let _ = monitor.poll().unwrap();
        let _ = monitor.poll().unwrap();

        let result3 = monitor.poll().unwrap();
        assert_eq!(result3[NETWORK_STAT_RX_INDEX].bytes, (3000 + 4000) / 2);
        assert_eq!(result3[NETWORK_STAT_TX_INDEX].bytes, (2000 + 1000) / 2);

        let result4 = monitor.poll().unwrap();
        assert_eq!(result4[NETWORK_STAT_RX_INDEX].bytes, (4000 + 3000) / 2);
        assert_eq!(result4[NETWORK_STAT_TX_INDEX].bytes, (1000 + 2000) / 2);
    }

    #[test]
    fn error_is_propagated() {
        let reader = MockProcNetDevReader::new(vec![Err("boom".to_string())]);
        let utils = MockNetworkUtilities::new(vec!["eth0"]);
        let mut monitor = NetworkMonitor::new(reader, utils, &make_config(1));

        let err = monitor.poll().unwrap_err();
        assert_eq!(err, "boom");
    }
}
