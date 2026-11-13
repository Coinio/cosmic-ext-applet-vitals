use crate::sensors::sensor_traits::SensorReader;
use crate::sensors::storage_reader::{MountUsage, StorageDisk, StorageReading, UnmountedVolume};
use log::info;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct StorageStats {
    /// The available space across the configured mounts.
    pub available_bytes: u64,
    /// The total space across the configured mounts.
    pub total_bytes: u64,
    /// The counted mounts, in mount table order. A filesystem mounted in more than one place is listed at each.
    pub mounts: Vec<MountUsage>,
    /// The volumes that aren't mounted, which have no free space to count. Only found by a detailed poll.
    pub unmounted: Vec<UnmountedVolume>,
    /// The disks the mounts and unmounted volumes are on. Only found by a detailed poll.
    pub disks: Vec<StorageDisk>,
}

/// The mounts and unmounted volumes on one disk, or the mounts that aren't on a single disk.
#[derive(Debug, PartialEq, Eq)]
pub struct DiskGroup<'a> {
    /// The disk, or `None` for the mounts that aren't on a single disk, e.g. network shares.
    pub disk: Option<&'a StorageDisk>,
    pub mounts: Vec<&'a MountUsage>,
    pub unmounted: Vec<&'a UnmountedVolume>,
}

impl StorageStats {
    /// The used space across the configured mounts, which counts the blocks reserved for root as used.
    pub fn used_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.available_bytes)
    }

    /// The mounts and unmounted volumes grouped by disk. The disks with mounts come first, in the order of their first
    /// mount, so the disk with the root filesystem is usually first. Then the disks with only unmounted volumes, by
    /// name, and last the mounts that aren't on a single disk, e.g. network shares.
    pub fn by_disk(&self) -> Vec<DiskGroup<'_>> {
        let mut groups: Vec<DiskGroup<'_>> = Vec::new();
        let mut elsewhere = DiskGroup::new(None);

        for mount in &self.mounts {
            match self.disk(mount.disk.as_deref()) {
                Some(disk) => DiskGroup::of(&mut groups, disk).mounts.push(mount),
                None => elsewhere.mounts.push(mount),
            }
        }
        for volume in &self.unmounted {
            match self.disk(Some(&volume.disk)) {
                Some(disk) => DiskGroup::of(&mut groups, disk).unmounted.push(volume),
                None => elsewhere.unmounted.push(volume),
            }
        }

        if !elsewhere.mounts.is_empty() || !elsewhere.unmounted.is_empty() {
            groups.push(elsewhere);
        }
        groups
    }

    fn disk(&self, name: Option<&str>) -> Option<&StorageDisk> {
        self.disks.iter().find(|disk| Some(disk.name.as_str()) == name)
    }
}

impl<'a> DiskGroup<'a> {
    fn new(disk: Option<&'a StorageDisk>) -> Self {
        Self {
            disk,
            mounts: Vec::new(),
            unmounted: Vec::new(),
        }
    }

    /// The disk's group in `groups`, added at the end when it isn't there yet.
    fn of<'g>(groups: &'g mut Vec<DiskGroup<'a>>, disk: &'a StorageDisk) -> &'g mut DiskGroup<'a> {
        let index = match groups.iter().position(|group| group.disk == Some(disk)) {
            Some(index) => index,
            None => {
                groups.push(DiskGroup::new(Some(disk)));
                groups.len() - 1
            }
        };
        &mut groups[index]
    }
}

/// Totals the space across the configured mounts. Storage space doesn't need smoothing, so there are no samples.
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

    /// Poll the space on the counted mounts, and when `detailed`, e.g. while the detail popup is wanted, the disk each
    /// is on and the volumes that aren't mounted too.
    pub fn poll(&mut self, detailed: bool) -> Result<StorageStats, String> {
        let StorageReading {
            mounts,
            unmounted,
            disks,
        } = if detailed {
            self.sensor_reader.read_detailed()?
        } else {
            self.sensor_reader.read()?
        };

        let mounts: Vec<MountUsage> = mounts
            .into_iter()
            .filter(|mount| self.counted_mount_points.contains(&mount.mount_point))
            .collect();

        let mut counted: HashSet<String> = HashSet::new();
        let mut stats = StorageStats::default();

        for mount in &mounts {
            // Bind mounts and btrfs subvolumes are the same filesystem mounted more than once, so only count it once.
            if !counted.insert(filesystem_key(mount)) {
                continue;
            }

            stats.available_bytes = stats.available_bytes.saturating_add(mount.available_bytes);
            stats.total_bytes = stats.total_bytes.saturating_add(mount.total_bytes);
        }

        stats.mounts = mounts;
        stats.unmounted = unmounted;
        stats.disks = disks;
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
                ..StorageReading::default()
            })
        }
    }

    fn usage(source: &str, mount_point: &str, total_bytes: u64, available_bytes: u64) -> MountUsage {
        MountUsage {
            source: source.to_string(),
            mount_point: mount_point.to_string(),
            total_bytes,
            available_bytes,
            disk: None,
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
        assert_eq!(stats.used_bytes(), 1100);
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
    fn only_lists_and_counts_the_configured_mounts() {
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
        assert_eq!(stats.mounts, vec![mounts[0].clone(), mounts[2].clone()]);
    }

    #[test]
    fn no_mounts_gives_zero() {
        let reader = MockStorageReader::new(vec![Ok(Vec::new())]);
        let mut monitor = StorageMonitor::new(reader, mount_points(&["/"]));

        let stats = monitor.poll(true).unwrap();

        assert_eq!(stats.total_bytes, 0);
        assert_eq!(stats.available_bytes, 0);
    }

    fn disk(name: &str) -> StorageDisk {
        StorageDisk {
            name: name.to_string(),
            model: None,
            size_bytes: 0,
        }
    }

    fn on_disk(mount: MountUsage, disk: &str) -> MountUsage {
        MountUsage {
            disk: Some(disk.to_string()),
            ..mount
        }
    }

    fn unmounted(name: &str, disk: &str) -> UnmountedVolume {
        UnmountedVolume {
            name: name.to_string(),
            disk: disk.to_string(),
            label: None,
            size_bytes: 0,
            is_locked: false,
        }
    }

    #[test]
    fn groups_by_disk_in_the_order_of_their_first_mount() {
        let stats = StorageStats {
            mounts: vec![
                on_disk(usage("/dev/dm-1", "/", 0, 0), "nvme0n1"),
                on_disk(usage("/dev/sdb1", "/data", 0, 0), "sdb"),
                usage("server:/export", "/mnt/share", 0, 0),
                on_disk(usage("/dev/nvme0n1p1", "/boot/efi", 0, 0), "nvme0n1"),
            ],
            unmounted: vec![unmounted("sda1", "sda"), unmounted("sdb2", "sdb")],
            disks: vec![disk("nvme0n1"), disk("sda"), disk("sdb")],
            ..StorageStats::default()
        };

        let groups = stats.by_disk();
        let summary: Vec<(Option<&str>, Vec<&str>, Vec<&str>)> = groups
            .iter()
            .map(|group| {
                (
                    group.disk.map(|disk| disk.name.as_str()),
                    group.mounts.iter().map(|mount| mount.mount_point.as_str()).collect(),
                    group.unmounted.iter().map(|volume| volume.name.as_str()).collect(),
                )
            })
            .collect();

        assert_eq!(
            summary,
            vec![
                (Some("nvme0n1"), vec!["/", "/boot/efi"], vec![]),
                (Some("sdb"), vec!["/data"], vec!["sdb2"]),
                (Some("sda"), vec![], vec!["sda1"]),
                (None, vec!["/mnt/share"], vec![]),
            ]
        );
    }

    #[test]
    fn mounts_without_known_disks_are_one_group() {
        // A poll that wasn't detailed doesn't find the disks.
        let stats = StorageStats {
            mounts: vec![usage("/dev/sda1", "/", 0, 0), usage("/dev/sdb1", "/data", 0, 0)],
            ..StorageStats::default()
        };

        let groups = stats.by_disk();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].disk, None);
        assert_eq!(groups[0].mounts.len(), 2);
    }

    #[test]
    fn error_is_propagated() {
        let reader = MockStorageReader::new(vec![Err("boom".to_string())]);
        let mut monitor = StorageMonitor::new(reader, mount_points(&["/"]));

        assert_eq!(monitor.poll(true).unwrap_err(), "boom");
    }
}
