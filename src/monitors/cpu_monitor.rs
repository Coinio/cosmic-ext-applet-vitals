use crate::configuration::app_configuration::AppConfiguration;
use crate::sensors::cpu_topology_reader::CpuTopology;
use crate::sensors::proc_stat_reader::{CpuTimes, ProcStatStatus};
use crate::sensors::sensor_traits::SensorReader;
use log::info;
use std::collections::{BTreeMap, VecDeque};

/// The usage of one CPU thread, or one physical core.
#[derive(Debug, Clone, PartialEq)]
pub struct CoreUsage {
    /// The thread's kernel number, or the physical core's number, both from 0.
    pub number: usize,
    pub usage_percent: f64,
}

#[derive(Default, Clone, Debug)]
pub struct CpuStats {
    pub cpu_usage_percent: f64,
    /// The usage of each thread, or of each physical core when they're combined, in order.
    pub cores: Vec<CoreUsage>,
}

/// Turns the times since boot into the usage since the previous reading, averaged over the recent samples.
struct UsageTracker {
    previous_idle: u64,
    previous_total: u64,
    sample_buffer: VecDeque<f64>,
    max_samples: usize,
}

impl UsageTracker {
    fn new(max_samples: usize) -> Self {
        Self {
            previous_idle: 0,
            previous_total: 0,
            sample_buffer: VecDeque::with_capacity(max_samples + 1),
            max_samples,
        }
    }

    fn update(&mut self, times: CpuTimes) -> f64 {
        let total_idle = times.idle + times.iowait;

        let idle_delta = total_idle.saturating_sub(self.previous_idle);
        let total_delta = times.total.saturating_sub(self.previous_total);

        // No time passed, e.g. the CPU was offline, so there's no usage to measure.
        let current_usage_percent = if total_delta == 0 {
            0.0
        } else {
            100.0 * (1.0 - idle_delta as f64 / total_delta as f64)
        };

        self.sample_buffer.push_back(current_usage_percent);

        self.previous_idle = total_idle;
        self.previous_total = times.total;

        if self.sample_buffer.len() > self.max_samples {
            self.sample_buffer.pop_front();
        }

        self.sample_buffer.iter().sum::<f64>() / self.sample_buffer.len() as f64
    }
}

pub struct CpuMonitor<S: SensorReader<Output = ProcStatStatus>> {
    sensor_reader: S,
    total: UsageTracker,
    /// The tracker for each thread, or each physical core, by its number.
    cores: BTreeMap<usize, UsageTracker>,
    /// Combines the threads into physical cores, when set.
    topology: Option<CpuTopology>,
    max_samples: usize,
}

impl<S: SensorReader<Output = ProcStatStatus>> CpuMonitor<S> {
    /// `topology` combines the threads into their physical cores. Without it, each thread is reported separately.
    pub fn new(sensor_reader: S, configuration: &AppConfiguration, topology: Option<CpuTopology>) -> Self {
        info!("Creating new cpu monitor {:?}", configuration.cpu);
        let max_samples = configuration.cpu.max_samples;
        Self {
            sensor_reader,
            total: UsageTracker::new(max_samples),
            cores: BTreeMap::new(),
            topology,
            max_samples,
        }
    }

    pub fn poll(&mut self) -> Result<CpuStats, String> {
        let current = self.sensor_reader.read()?;

        let cpu_usage_percent = self.total.update(current.all);

        let mut core_times: BTreeMap<usize, CpuTimes> = BTreeMap::new();
        for (cpu, times) in current.cpus {
            let core = self.core_of(cpu);
            let entry = core_times.entry(core).or_default();
            *entry = *entry + times;
        }

        // A CPU that went offline is forgotten, so it starts afresh if it comes back.
        self.cores.retain(|core, _| core_times.contains_key(core));

        let cores = core_times
            .into_iter()
            .map(|(number, times)| {
                let tracker = self
                    .cores
                    .entry(number)
                    .or_insert_with(|| UsageTracker::new(self.max_samples));
                CoreUsage {
                    number,
                    usage_percent: tracker.update(times),
                }
            })
            .collect();

        Ok(CpuStats {
            cpu_usage_percent,
            cores,
        })
    }

    /// The number the thread is reported under: its own, or its physical core's. A thread missing from the topology,
    /// e.g. one that came online since it was read, is reported on its own after the cores.
    fn core_of(&self, cpu: usize) -> usize {
        match &self.topology {
            None => cpu,
            Some(topology) => topology.core_of(cpu).unwrap_or(UNKNOWN_CORE_OFFSET + cpu),
        }
    }
}

/// Numbers threads missing from the topology after every real core.
const UNKNOWN_CORE_OFFSET: usize = 1 << 20;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct MockProcStatReader {
        // Fix set of readings to return from the reader.
        readings: Vec<Result<ProcStatStatus, String>>,
        // Tracks the index of the next reading to return.
        index: Cell<usize>,
    }

    impl MockProcStatReader {
        fn new(readings: Vec<Result<ProcStatStatus, String>>) -> Self {
            Self {
                readings,
                index: Cell::new(0),
            }
        }
    }

    impl SensorReader for MockProcStatReader {
        type Output = ProcStatStatus;

        fn read(&self) -> Result<Self::Output, String> {
            let read_index = self.index.get();
            let reading = self.readings[read_index].clone();

            self.index.set(read_index + 1);
            reading
        }
    }

    fn make_config(max_samples: usize) -> AppConfiguration {
        let mut cfg = AppConfiguration::default();
        cfg.cpu.max_samples = max_samples;
        cfg
    }

    fn with_cpus(cpus: &[(usize, CpuTimes)]) -> Result<ProcStatStatus, String> {
        let all = cpus.iter().fold(CpuTimes::default(), |sum, (_, times)| sum + *times);
        Ok(ProcStatStatus {
            all,
            cpus: cpus.to_vec(),
        })
    }

    fn usages(stats: &CpuStats) -> Vec<(usize, f64)> {
        stats
            .cores
            .iter()
            .map(|core| (core.number, core.usage_percent))
            .collect()
    }

    #[test]
    fn single_poll_computes_expected_usage() {
        let reader = MockProcStatReader::new(vec![Ok(ProcStatStatus::new(100, 50, 500))]);
        let mut monitor = CpuMonitor::new(reader, &make_config(4), None);

        let stats = monitor.poll().expect("poll should succeed");

        assert!(eq_to_three_decimal_places(stats.cpu_usage_percent, 70.000));
    }

    #[test]
    fn multiple_polls_average_and_buffering() {
        let reader = MockProcStatReader::new(vec![
            Ok(ProcStatStatus::new(100, 50, 500)),
            Ok(ProcStatStatus::new(200, 60, 700)),
        ]);
        let mut monitor = CpuMonitor::new(reader, &make_config(4), None);

        let reading1 = monitor.poll().unwrap();
        assert!(eq_to_three_decimal_places(reading1.cpu_usage_percent, 70.000));

        let reading2 = monitor.poll().unwrap();
        assert!(eq_to_three_decimal_places(reading2.cpu_usage_percent, 57.500));
    }

    #[test]
    fn sample_buffer_trims_to_max_samples() {
        let reader = MockProcStatReader::new(vec![
            Ok(ProcStatStatus::new(0, 0, 100)),
            Ok(ProcStatStatus::new(50, 0, 200)),
            Ok(ProcStatStatus::new(100, 0, 300)),
        ]);
        let mut monitor = CpuMonitor::new(reader, &make_config(2), None);

        _ = monitor.poll();
        _ = monitor.poll();

        assert!(monitor.total.sample_buffer.len() == 2);
    }

    #[test]
    fn error_is_propagated() {
        let reader = MockProcStatReader::new(vec![Err("boom".to_string())]);
        let mut monitor = CpuMonitor::new(reader, &make_config(1), None);

        let err = monitor.poll().unwrap_err();
        assert_eq!(err, "boom");
    }

    #[test]
    fn reports_each_thread() {
        let reader = MockProcStatReader::new(vec![
            with_cpus(&[(0, CpuTimes::new(0, 0, 0)), (1, CpuTimes::new(0, 0, 0))]),
            with_cpus(&[(0, CpuTimes::new(75, 0, 100)), (1, CpuTimes::new(25, 0, 100))]),
        ]);
        let mut monitor = CpuMonitor::new(reader, &make_config(1), None);

        _ = monitor.poll();
        let stats = monitor.poll().unwrap();

        assert_eq!(usages(&stats), vec![(0, 25.0), (1, 75.0)]);
        assert!(eq_to_three_decimal_places(stats.cpu_usage_percent, 50.0));
    }

    #[test]
    fn combines_threads_into_their_physical_cores() {
        // Two cores of two threads, with the second thread of each core numbered after every first thread.
        let topology = CpuTopology::from_cores([(0, 0), (1, 1), (2, 0), (3, 1)]);
        let reader = MockProcStatReader::new(vec![
            with_cpus(&[
                (0, CpuTimes::new(0, 0, 0)),
                (1, CpuTimes::new(0, 0, 0)),
                (2, CpuTimes::new(0, 0, 0)),
                (3, CpuTimes::new(0, 0, 0)),
            ]),
            with_cpus(&[
                (0, CpuTimes::new(0, 0, 100)),
                (1, CpuTimes::new(100, 0, 100)),
                (2, CpuTimes::new(50, 0, 100)),
                (3, CpuTimes::new(90, 10, 100)),
            ]),
        ]);
        let mut monitor = CpuMonitor::new(reader, &make_config(1), Some(topology));

        _ = monitor.poll();
        let stats = monitor.poll().unwrap();

        assert_eq!(usages(&stats), vec![(0, 75.0), (1, 0.0)]);
    }

    #[test]
    fn threads_missing_from_the_topology_follow_the_cores() {
        let topology = CpuTopology::from_cores([(0, 0), (1, 0)]);
        let reader = MockProcStatReader::new(vec![with_cpus(&[
            (0, CpuTimes::new(50, 0, 100)),
            (1, CpuTimes::new(50, 0, 100)),
            (2, CpuTimes::new(100, 0, 100)),
        ])]);
        let mut monitor = CpuMonitor::new(reader, &make_config(1), Some(topology));

        let stats = monitor.poll().unwrap();

        assert_eq!(stats.cores.len(), 2);
        assert_eq!(stats.cores[0].number, 0);
        assert_eq!(stats.cores[1].number, UNKNOWN_CORE_OFFSET + 2);
    }

    #[test]
    fn a_cpu_that_goes_offline_is_dropped_and_restarts() {
        let reader = MockProcStatReader::new(vec![
            with_cpus(&[(0, CpuTimes::new(0, 0, 0)), (1, CpuTimes::new(0, 0, 0))]),
            with_cpus(&[(0, CpuTimes::new(50, 0, 100))]),
            with_cpus(&[(0, CpuTimes::new(100, 0, 200)), (1, CpuTimes::new(100, 0, 400))]),
        ]);
        let mut monitor = CpuMonitor::new(reader, &make_config(1), None);

        _ = monitor.poll();
        let offline = monitor.poll().unwrap();
        assert_eq!(usages(&offline), vec![(0, 50.0)]);

        let back = monitor.poll().unwrap();
        // CPU 1's history was dropped, so it's measured from boot like a new CPU.
        assert_eq!(
            back.cores[1],
            CoreUsage {
                number: 1,
                usage_percent: 75.0
            }
        );
    }

    #[test]
    fn no_elapsed_time_is_no_usage() {
        let reader = MockProcStatReader::new(vec![
            with_cpus(&[(0, CpuTimes::new(50, 0, 100))]),
            with_cpus(&[(0, CpuTimes::new(50, 0, 100))]),
        ]);
        let mut monitor = CpuMonitor::new(reader, &make_config(1), None);

        _ = monitor.poll();
        let stats = monitor.poll().unwrap();

        assert_eq!(usages(&stats), vec![(0, 0.0)]);
        assert!(!stats.cpu_usage_percent.is_nan());
    }

    fn eq_to_three_decimal_places(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.0005
    }
}
