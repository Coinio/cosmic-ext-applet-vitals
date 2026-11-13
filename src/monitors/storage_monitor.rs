use crate::sensors::sensor_traits::SensorReader;
use crate::sensors::storage_reader::{MountUsage, StorageReading, UnmountedVolume};
use log::info;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct StorageStats {
    /// The available space across the configured mounts.
    pub available_bytes: u64,
    /// The total space across the configured mounts.
    pub total_bytes: u64,
    /// Every mounted filesystem, including those not counted in the totals, in mount table order. Only the counted
    /// ones unless the poll was detailed.
    pub mounts: Vec<MountUsage>,
    /// The volumes that aren't mounted, which have no free space to count. Only found by a detailed poll.
    pub unmounted: Vec<UnmountedVolume>,
}

/// Totals the space across the configured mounts. Free space doesn't need smoothing, so there are no samples.
pub struct StorageMonitor<S: SensorReader<Output = StorageReading>> {
    sensor_reader: S,
    /// The mount points counted in the totals.
    counted_mount_points: Vec<String>,
}

impl<S: SensorReader<Output = StorageReading>> StorageMonitor<S> {
    pub fn new(sensor_reader: S, counted_mount_points: Vec<String>) -> Self {
        info!("Creating new storage monitor");
        Self {
            sensor_reader,
            counted_mount_points,
        }
    }

    /// Poll the space on the counted mounts, and when `detailed`, e.g. while the detail popup is wanted, every mount
    /// and the volumes that aren't mounted too.
    pub fn poll(&mut self, detailed: bool) -> Result<StorageStats, String> {
        let StorageReading { mounts, unmounted } = if detailed {
            self.sensor_reader.read_detailed()?
        } else {
            self.sensor_reader.read()?
        };

        let mut counted: HashSet<String> = HashSet::new();
        let mut stats = StorageStats::default();

        for mount in mounts.iter().filter(|mount| self.counted_mount_points.contains(&mount.mount_point)) {
            // Bind mounts and btrfs subvolumes are the same filesystem mounted more than once, so only count it once.
            if !counted.insert(filesystem_key(mount)) {
                continue;
            }

            stats.available_bytes = stats.available_bytes.saturating_add(mount.available_bytes);
            stats.total_bytes = stats.total_bytes.saturating_add(mount.total_bytes);
        }

        stats.mounts = mounts;
        stats.unmounted = unmounted;
        Ok(stats)
    }
}

/// Identifies the filesystem behind a mount. Block devices identify it, but other sources, e.g. `none`, can be shared
/// by unrelated filesystems, so those fall back to the mount point.
fn filesystem_key(mount: &MountUsage) -> String {
    if mount.source.starts_with("/dev/") {
        mount.source.clone()
    } else {
        mount.mount_point.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct MockStorageReader {
        readings: Vec<Result<Vec<MountUsage>, String>>,
        index: Cell<usize>,
    }

    impl MockStorageReader {
        fn new(readings: Vec<Result<Vec<MountUsage>, String>>) -> Self {
            Self {
                readings,
                index: Cell::new(0),
            }
        }
    }

    impl SensorReader for MockStorageReader {
        type Output = StorageReading;

        fn read(&self) -> Result<Self::Output, String> {
            let index = self.index.get();
            self.index.set(index + 1);
            self.readings[index].clone().map(|mounts| StorageReading {
                mounts,
                unmounted: Vec::new(),
            })
        }
    }

    fn usage(source: &str, mount_point: &str, total_bytes: u64, available_bytes: u64) -> MountUsage {
        MountUsage {
            source: source.to_string(),
            mount_point: mount_point.to_string(),
            total_bytes,
            available_bytes,
        }
    }

    fn mount_points(mount_points: &[&str]) -> Vec<String> {
        mount_points.iter().map(|mount_point| mount_point.to_string()).collect()
    }

    #[test]
    fn sums_separate_filesystems() {
        let reader = MockStorageReader::new(vec![Ok(vec![
            usage("/dev/nvme0n1p2", "/", 1000, 400),
            usage("/dev/sda1", "/data", 2000, 1500),
        ])]);
        let mut monitor = StorageMonitor::new(reader, mount_points(&["/", "/data"]));

        let stats = monitor.poll(true).unwrap();

        assert_eq!(stats.total_bytes, 3000);
        assert_eq!(stats.available_bytes, 1900);
    }

    #[test]
    fn counts_a_filesystem_mounted_twice_once() {
        let reader = MockStorageReader::new(vec![Ok(vec![
            usage("/dev/nvme0n1p2", "/", 1000, 400),
            usage("/dev/nvme0n1p2", "/home", 1000, 400),
        ])]);
        let mut monitor = StorageMonitor::new(reader, mount_points(&["/", "/home"]));

        let stats = monitor.poll(true).unwrap();

        assert_eq!(stats.total_bytes, 1000);
        assert_eq!(stats.available_bytes, 400);
    }

    #[test]
    fn non_device_sources_are_counted_per_mount_point() {
        let reader = MockStorageReader::new(vec![Ok(vec![
            usage("none", "/mnt/a", 1000, 100),
            usage("none", "/mnt/b", 500, 50),
        ])]);
        let mut monitor = StorageMonitor::new(reader, mount_points(&["/mnt/a", "/mnt/b"]));

        let stats = monitor.poll(true).unwrap();

        assert_eq!(stats.total_bytes, 1500);
        assert_eq!(stats.available_bytes, 150);
    }

    #[test]
    fn lists_every_mount_but_only_counts_the_configured_ones() {
        let mounts = vec![
            usage("/dev/nvme0n1p2", "/", 1000, 400),
            usage("/dev/nvme0n1p1", "/boot/efi", 100, 90),
            usage("/dev/sda1", "/data", 2000, 1500),
        ];
        let reader = MockStorageReader::new(vec![Ok(mounts.clone())]);
        let mut monitor = StorageMonitor::new(reader, mount_points(&["/", "/data"]));

        let stats = monitor.poll(true).unwrap();

        assert_eq!(stats.total_bytes, 3000);
        assert_eq!(stats.available_bytes, 1900);
        assert_eq!(stats.mounts, mounts);
    }

    #[test]
    fn no_mounts_gives_zero() {
        let reader = MockStorageReader::new(vec![Ok(Vec::new())]);
        let mut monitor = StorageMonitor::new(reader, mount_points(&["/"]));

        let stats = monitor.poll(true).unwrap();

        assert_eq!(stats.total_bytes, 0);
        assert_eq!(stats.available_bytes, 0);
    }

    #[test]
    fn error_is_propagated() {
        let reader = MockStorageReader::new(vec![Err("boom".to_string())]);
        let mut monitor = StorageMonitor::new(reader, mount_points(&["/"]));

        assert_eq!(monitor.poll(true).unwrap_err(), "boom");
    }
}
