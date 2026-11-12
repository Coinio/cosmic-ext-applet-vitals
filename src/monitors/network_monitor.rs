use crate::configuration::app_configuration::AppConfiguration;
use crate::monitors::per_second;
use crate::sensors::proc_net_dev_reader::ProcNetDevStatus;
use crate::sensors::sensor_traits::SensorReader;
use log::info;
use std::collections::{HashMap, VecDeque};
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
struct NetworkSample {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    /// The time the bytes were counted over.
    pub elapsed: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    /// The bytes received per second by all the physical interfaces, averaged over the samples.
    pub rx_bytes: u64,
    /// The bytes sent per second by all the physical interfaces, averaged over the samples.
    pub tx_bytes: u64,
    /// Each physical interface, connected ones first.
    pub interfaces: Vec<InterfaceStats>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InterfaceStats {
    pub name: String,
    /// The bytes received per second, averaged over the samples.
    pub rx_bytes: u64,
    /// The bytes sent per second, averaged over the samples.
    pub tx_bytes: u64,
    pub is_up: bool,
    pub ipv4: Option<Ipv4Addr>,
}

/// The samples of one interface, which are kept apart so an interface coming or going doesn't skew the others.
#[derive(Debug, Default)]
struct InterfaceHistory {
    previous: NetworkSample,
    samples: VecDeque<NetworkSample>,
}

pub struct NetworkMonitor<S: SensorReader<Output = ProcNetDevStatus>> {
    sensor_reader: S,
    interfaces: HashMap<String, InterfaceHistory>,
    max_samples: usize,
    /// When the totals were last read.
    last_poll: Option<Instant>,
}

impl<S: SensorReader<Output = ProcNetDevStatus>> NetworkMonitor<S> {
    pub fn new(sensor_reader: S, configuration: &AppConfiguration) -> Self {
        info!("Creating new network monitor {:?}", configuration.network);
        Self {
            sensor_reader,
            interfaces: HashMap::new(),
            max_samples: configuration.network.max_samples,
            last_poll: None,
        }
    }

    pub fn poll(&mut self) -> Result<NetworkStats, String> {
        self.poll_at(Instant::now())
    }

    /// Poll, with the totals read at `now`.
    fn poll_at(&mut self, now: Instant) -> Result<NetworkStats, String> {
        let current = self.sensor_reader.read()?;
        let elapsed = self.last_poll.map_or(Duration::ZERO, |last| now.saturating_duration_since(last));
        self.last_poll = Some(now);

        let mut interfaces = Vec::new();
        let mut previous_interfaces = std::mem::take(&mut self.interfaces);

        for device in current.device_statuses {
            if !device.is_physical_device {
                continue;
            }

            let totals = NetworkSample {
                rx_bytes: device.rx_bytes,
                tx_bytes: device.tx_bytes,
                elapsed: Duration::ZERO,
            };

            // A new interface starts from its current totals, so its first sample is 0 over no time, rather than
            // everything since boot.
            let (mut history, elapsed) = match previous_interfaces.remove(&device.name) {
                Some(history) => (history, elapsed),
                None => (
                    InterfaceHistory {
                        previous: totals.clone(),
                        samples: VecDeque::with_capacity(self.max_samples),
                    },
                    Duration::ZERO,
                ),
            };

            // Saturating, as the totals restart from 0 if the driver is reloaded.
            history.samples.push_back(NetworkSample {
                rx_bytes: totals.rx_bytes.saturating_sub(history.previous.rx_bytes),
                tx_bytes: totals.tx_bytes.saturating_sub(history.previous.tx_bytes),
                elapsed,
            });
            while history.samples.len() > self.max_samples {
                history.samples.pop_front();
            }
            history.previous = totals;

            let elapsed = history.samples.iter().map(|sample| sample.elapsed).sum();
            interfaces.push(InterfaceStats {
                name: device.name.clone(),
                rx_bytes: per_second(history.samples.iter().map(|sample| sample.rx_bytes).sum(), elapsed),
                tx_bytes: per_second(history.samples.iter().map(|sample| sample.tx_bytes).sum(), elapsed),
                is_up: device.is_up,
                ipv4: device.ipv4,
            });

            self.interfaces.insert(device.name, history);
        }

        interfaces.sort_by(|a, b| b.is_up.cmp(&a.is_up).then_with(|| a.name.cmp(&b.name)));

        Ok(NetworkStats {
            rx_bytes: interfaces.iter().map(|interface| interface.rx_bytes).sum(),
            tx_bytes: interfaces.iter().map(|interface| interface.tx_bytes).sum(),
            interfaces,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::proc_net_dev_reader::{ProcNetDevDeviceStatus, ProcNetDevStatus};
    use std::cell::Cell;

    struct MockProcNetDevReader {
        readings: Vec<Result<ProcNetDevStatus, String>>,
        index: Cell<usize>,
    }

    impl MockProcNetDevReader {
        fn new(readings: Vec<Result<ProcNetDevStatus, String>>) -> Self {
            Self {
                readings,
                index: Cell::new(0),
            }
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

    impl NetworkMonitor<MockProcNetDevReader> {
        /// Poll a second after the last poll, as at the default update interval.
        fn poll_next_second(&mut self) -> Result<NetworkStats, String> {
            let now = self.last_poll.map_or_else(Instant::now, |last| last + Duration::from_secs(1));
            self.poll_at(now)
        }
    }

    fn make_config(max_samples: usize) -> AppConfiguration {
        let mut cfg = AppConfiguration::default();
        cfg.network.max_samples = max_samples;
        cfg
    }

    /// A physical device is named eth0, and a virtual one virt0.
    fn create_device_status(rx: u64, tx: u64, is_physical_device: bool) -> ProcNetDevDeviceStatus {
        named_device_status(if is_physical_device { "eth0" } else { "virt0" }, rx, tx, is_physical_device)
    }

    fn named_device_status(name: &str, rx: u64, tx: u64, is_physical_device: bool) -> ProcNetDevDeviceStatus {
        ProcNetDevDeviceStatus {
            name: name.to_string(),
            rx_bytes: rx,
            tx_bytes: tx,
            is_physical_device,
            is_up: is_physical_device,
            ipv4: None,
        }
    }

    /// This should give a zero result as we don't have any previous readings to compare against.
    #[test]
    fn first_poll_gives_zero_result() {
        // Add a physical and a virtual interface to the status. Virtual should be ignored.
        let status = ProcNetDevStatus::new(vec![
            create_device_status(2000, 1000, true),  // physical
            create_device_status(7000, 5000, false), // virtual, ignored
        ]);

        let reader = MockProcNetDevReader::new(vec![Ok(status)]);

        let mut monitor = NetworkMonitor::new(reader, &make_config(3));
        let result = monitor.poll_next_second();

        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result.rx_bytes, 0);
        assert_eq!(result.tx_bytes, 0);
    }

    #[test]
    fn second_poll_gives_expected_result() {
        // Add a physical and a virtual interface to the status. Virtual should be ignored.
        let sample1 = ProcNetDevStatus::new(vec![
            create_device_status(2000, 1000, true),  // physical
            create_device_status(7000, 5000, false), // virtual, ignored
        ]);

        let sample2 = ProcNetDevStatus::new(vec![
            create_device_status(3000, 1500, true),
            create_device_status(1000, 1000, false),
        ]);

        let reader = MockProcNetDevReader::new(vec![Ok(sample1), Ok(sample2)]);

        let mut monitor = NetworkMonitor::new(reader, &make_config(2));

        // Throw away first result as we don't have any previous readings to compare against.
        let _ = monitor.poll_next_second();
        let result = monitor.poll_next_second();

        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result.rx_bytes, 3000 - 2000);
        assert_eq!(result.tx_bytes, 1500 - 1000);
    }

    #[test]
    fn multiple_poll_gives_expected_result() {
        // Add a physical and a virtual interface to the status. Virtual should be ignored.
        let sample1 = ProcNetDevStatus::new(vec![
            create_device_status(2000, 1000, true),
            create_device_status(10, 10, false),
        ]);
        let sample2 = ProcNetDevStatus::new(vec![
            create_device_status(6000, 3000, true),
            create_device_status(20, 40, false),
        ]);
        let sample3 = ProcNetDevStatus::new(vec![
            create_device_status(9000, 6000, true),
            create_device_status(30, 70, false),
        ]);
        let sample4 = ProcNetDevStatus::new(vec![
            create_device_status(12000, 7000, true),
            create_device_status(30, 70, false),
        ]);

        let reader = MockProcNetDevReader::new(vec![Ok(sample1), Ok(sample2), Ok(sample3), Ok(sample4)]);

        let mut monitor = NetworkMonitor::new(reader, &make_config(3));

        _ = monitor.poll_next_second();

        let result2 = monitor.poll_next_second().unwrap();
        assert_eq!(result2.rx_bytes, 6000 - 2000);
        assert_eq!(result2.tx_bytes, 3000 - 1000);

        // Over 2 seconds.
        let result3 = monitor.poll_next_second().unwrap();
        assert_eq!(result3.rx_bytes, (9000 - 2000) / 2);
        assert_eq!(result3.tx_bytes, (6000 - 1000) / 2);

        // The first sample, over no time, has been dropped, leaving 3 seconds.
        let result4 = monitor.poll_next_second().unwrap();
        assert_eq!(result4.rx_bytes, (12000 - 2000) / 3);
        assert_eq!(result4.tx_bytes, (7000 - 1000) / 3);
    }

    #[test]
    fn samples_buffer_trims_to_max_size() {
        let sample1 = ProcNetDevStatus::new(vec![create_device_status(2000, 1000, true)]);
        let sample2 = ProcNetDevStatus::new(vec![create_device_status(5000, 3000, true)]);
        let sample3 = ProcNetDevStatus::new(vec![create_device_status(9000, 4000, true)]);
        let sample4 = ProcNetDevStatus::new(vec![create_device_status(12000, 6000, true)]);

        let reader = MockProcNetDevReader::new(vec![Ok(sample1), Ok(sample2), Ok(sample3), Ok(sample4)]);

        let mut monitor = NetworkMonitor::new(reader, &make_config(2));

        let _ = monitor.poll_next_second();
        let _ = monitor.poll_next_second();
        let _ = monitor.poll_next_second();

        assert_eq!(monitor.interfaces["eth0"].samples.len(), 2);
    }

    /// Rates are per second whatever the update interval, and however late the poll is.
    #[test]
    fn rates_are_per_second_of_elapsed_time() {
        let reader = MockProcNetDevReader::new(vec![
            Ok(ProcNetDevStatus::new(vec![create_device_status(0, 0, true)])),
            Ok(ProcNetDevStatus::new(vec![create_device_status(4000, 2000, true)])),
            Ok(ProcNetDevStatus::new(vec![create_device_status(5000, 2500, true)])),
        ]);
        let mut monitor = NetworkMonitor::new(reader, &make_config(1));

        let start = Instant::now();
        _ = monitor.poll_at(start);
        let result = monitor.poll_at(start + Duration::from_secs(2)).unwrap();
        assert_eq!((result.rx_bytes, result.tx_bytes), (2000, 1000));

        let result = monitor.poll_at(start + Duration::from_millis(2500)).unwrap();
        assert_eq!((result.rx_bytes, result.tx_bytes), (2000, 1000));
    }

    #[test]
    fn uses_the_network_max_samples() {
        let mut config = AppConfiguration::default();
        config.network.max_samples = 7;
        config.memory.max_samples = 1;

        let reader = MockProcNetDevReader::new(vec![]);
        let monitor = NetworkMonitor::new(reader, &config);

        assert_eq!(monitor.max_samples, 7);
    }

    #[test]
    fn reports_each_physical_interface() {
        let sample1 = ProcNetDevStatus::new(vec![
            named_device_status("wlan0", 1000, 100, true),
            named_device_status("eth0", 5000, 500, true),
            named_device_status("tun0", 9000, 900, false),
        ]);
        let sample2 = ProcNetDevStatus::new(vec![
            named_device_status("wlan0", 3000, 300, true),
            named_device_status("eth0", 6000, 600, true),
            named_device_status("tun0", 19000, 1900, false),
        ]);

        let reader = MockProcNetDevReader::new(vec![Ok(sample1), Ok(sample2)]);
        let mut monitor = NetworkMonitor::new(reader, &make_config(1));

        _ = monitor.poll_next_second();
        let result = monitor.poll_next_second().unwrap();

        let names: Vec<&str> = result.interfaces.iter().map(|interface| interface.name.as_str()).collect();
        assert_eq!(names, ["eth0", "wlan0"]);
        assert_eq!((result.interfaces[0].rx_bytes, result.interfaces[0].tx_bytes), (1000, 100));
        assert_eq!((result.interfaces[1].rx_bytes, result.interfaces[1].tx_bytes), (2000, 200));
        assert_eq!((result.rx_bytes, result.tx_bytes), (3000, 300));
    }

    #[test]
    fn lists_connected_interfaces_first() {
        let mut disconnected = named_device_status("eth0", 0, 0, true);
        disconnected.is_up = false;
        let status = ProcNetDevStatus::new(vec![disconnected, named_device_status("wlan0", 0, 0, true)]);

        let reader = MockProcNetDevReader::new(vec![Ok(status)]);
        let mut monitor = NetworkMonitor::new(reader, &make_config(1));

        let names: Vec<String> = monitor.poll_next_second().unwrap().interfaces.into_iter().map(|i| i.name).collect();
        assert_eq!(names, ["wlan0", "eth0"]);
    }

    /// Unplugging a USB adapter removes its totals, which mustn't make the totals go backwards.
    #[test]
    fn removed_interface_is_dropped() {
        let sample1 = ProcNetDevStatus::new(vec![
            named_device_status("eth0", 1000, 1000, true),
            named_device_status("usb0", 50000, 50000, true),
        ]);
        let sample2 = ProcNetDevStatus::new(vec![named_device_status("eth0", 2000, 1500, true)]);

        let reader = MockProcNetDevReader::new(vec![Ok(sample1), Ok(sample2)]);
        let mut monitor = NetworkMonitor::new(reader, &make_config(1));

        _ = monitor.poll_next_second();
        let result = monitor.poll_next_second().unwrap();

        assert_eq!((result.rx_bytes, result.tx_bytes), (1000, 500));
        assert_eq!(result.interfaces.len(), 1);
        assert!(!monitor.interfaces.contains_key("usb0"));
    }

    #[test]
    fn counter_reset_reads_as_zero() {
        let sample1 = ProcNetDevStatus::new(vec![named_device_status("eth0", 5000, 5000, true)]);
        let sample2 = ProcNetDevStatus::new(vec![named_device_status("eth0", 100, 100, true)]);

        let reader = MockProcNetDevReader::new(vec![Ok(sample1), Ok(sample2)]);
        let mut monitor = NetworkMonitor::new(reader, &make_config(1));

        _ = monitor.poll_next_second();
        let result = monitor.poll_next_second().unwrap();

        assert_eq!((result.rx_bytes, result.tx_bytes), (0, 0));
    }

    #[test]
    fn error_is_propagated() {
        let reader = MockProcNetDevReader::new(vec![Err("boom".to_string())]);
        let mut monitor = NetworkMonitor::new(reader, &make_config(1));

        let err = monitor.poll_next_second().unwrap_err();
        assert_eq!(err, "boom");
    }
}
