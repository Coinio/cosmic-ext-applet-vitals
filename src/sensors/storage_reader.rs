use crate::sensors::proc_disk_stats_reader::{
    device_name, has_media, is_disk, mounted_devices, read_disk_model, SYS_BLOCK_ROOT,
};
use crate::sensors::proc_mounts_reader::{storage_mounts, MountEntry, ProcMountsReader};
use crate::sensors::sensor_traits::SensorReader;
use log::warn;
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::Path;

/// Where udev keeps what it knows about each device, including the filesystem on it, which users can read.
const UDEV_DATA_ROOT: &str = "/run/udev/data";

const SECTOR_SIZE_BYTES: u64 = 512;

/// The space on one mounted filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountUsage {
    /// What is mounted, e.g. `/dev/nvme0n1p2`. Mounts of the same filesystem share a source.
    pub source: String,
    pub mount_point: String,
    pub total_bytes: u64,
    /// The space available to unprivileged users, which excludes the blocks reserved for root, as `df` reports.
    pub available_bytes: u64,
    /// The name of the disk the filesystem is on, e.g. `nvme0n1`, through any encryption or LVM. `None` when it isn't
    /// on a disk, e.g. a network share, when it spans several disks, or when the read wasn't detailed.
    pub disk: Option<String>,
}

impl MountUsage {
    /// The space unprivileged users can't use, which counts the blocks reserved for root as used.
    pub fn used_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.available_bytes)
    }
}

/// A partition, or a disk without partitions, with a filesystem on it that isn't mounted, e.g. a second drive, or an
/// encrypted partition that hasn't been unlocked. Its free space can't be read until it's mounted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnmountedVolume {
    /// The device name, e.g. `sda1`.
    pub name: String,
    /// The name of the disk it's on, e.g. `sda`, which is its own name when it has no partitions.
    pub disk: String,
    /// The filesystem's label, if it has one.
    pub label: Option<String>,
    pub size_bytes: u64,
    /// An encrypted volume that hasn't been unlocked.
    pub is_locked: bool,
}

/// A disk with media in it, which the mounts and unmounted volumes are on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageDisk {
    /// The kernel name, e.g. `nvme0n1`.
    pub name: String,
    /// The disk's model, e.g. "Samsung SSD 980 1TB", where the driver reports one.
    pub model: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StorageReading {
    /// The counted mounts, in mount table order.
    pub mounts: Vec<MountUsage>,
    /// The volumes that aren't mounted, by disk and then partition. Only found by a detailed read.
    pub unmounted: Vec<UnmountedVolume>,
    /// The disks, by name. Only found by a detailed read.
    pub disks: Vec<StorageDisk>,
}

/// Reads the space on the mounted storage filesystems. Only mounted filesystems are read, so an unplugged drive doesn't
/// report the space of the filesystem its mount point directory is on.
///
/// The panel and the detail popup only show the counted mount points, so only those are read. Reading the space on a
/// network mount asks its server. A detailed read, for the detail popup, also finds the disk each mount is on, and the
/// volumes that aren't mounted, which reads a few sysfs and udev files for every partition.
pub struct StorageReader {
    counted_mount_points: Vec<String>,
}

impl StorageReader {
    pub fn new(counted_mount_points: Vec<String>) -> Self {
        Self { counted_mount_points }
    }

    fn read_with(&self, detailed: bool) -> Result<StorageReading, String> {
        let all_mounts = ProcMountsReader.read()?;

        let mounts = storage_mounts(&all_mounts)
            .into_iter()
            .filter(|mount| self.counted_mount_points.contains(&mount.mount_point))
            .filter_map(|mount| match rustix::fs::statvfs(mount.mount_point.as_str()) {
                Ok(stats) => Some(MountUsage {
                    total_bytes: stats.f_blocks.saturating_mul(stats.f_frsize),
                    available_bytes: stats.f_bavail.saturating_mul(stats.f_frsize),
                    disk: None,
                    source: mount.source,
                    mount_point: mount.mount_point,
                }),
                Err(err) => {
                    warn!("Unable to read the space on {}: {}", mount.mount_point, err);
                    None
                }
            })
            .collect();

        if !detailed {
            return Ok(StorageReading {
                mounts,
                ..StorageReading::default()
            });
        }

        let sys_block_root = Path::new(SYS_BLOCK_ROOT);
        let layout = DiskLayout::read(sys_block_root);
        let mounts = layout.assign_disks(sys_block_root, mounts);
        let unmounted = unmounted_volumes(sys_block_root, Path::new(UDEV_DATA_ROOT), &layout, &all_mounts);

        Ok(StorageReading {
            mounts,
            unmounted,
            disks: layout.disks.into_iter().map(|disk| disk.disk).collect(),
        })
    }
}

impl SensorReader for StorageReader {
    type Output = StorageReading;

    fn read(&self) -> Result<Self::Output, String> {
        self.read_with(false)
    }

    fn read_detailed(&self) -> Result<Self::Output, String> {
        self.read_with(true)
    }
}

/// A disk and its partitions.
struct LaidOutDisk {
    disk: StorageDisk,
    /// The partitions' names, in order.
    partitions: Vec<String>,
}

/// The disks with media in `sys_block_root`, by name, and their partitions.
struct DiskLayout {
    disks: Vec<LaidOutDisk>,
}

impl DiskLayout {
    fn read(sys_block_root: &Path) -> Self {
        let Ok(entries) = fs::read_dir(sys_block_root) else {
            return Self { disks: Vec::new() };
        };
        let mut names: Vec<String> =
            entries.flatten().filter_map(|entry| entry.file_name().into_string().ok()).collect();
        names.retain(|disk| is_disk(sys_block_root, disk) && has_media(sys_block_root, disk));
        names.sort();

        let disks = names
            .into_iter()
            .map(|name| {
                let disk_dir = sys_block_root.join(&name);
                let mut partitions: Vec<String> = fs::read_dir(&disk_dir)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter(|entry| entry.path().join("partition").exists())
                    .filter_map(|entry| entry.file_name().into_string().ok())
                    .collect();
                partitions.sort();

                let sectors: u64 = fs::read_to_string(disk_dir.join("size"))
                    .ok()
                    .and_then(|size| size.trim().parse().ok())
                    .unwrap_or_default();

                LaidOutDisk {
                    disk: StorageDisk {
                        model: read_disk_model(sys_block_root, &name),
                        size_bytes: sectors.saturating_mul(SECTOR_SIZE_BYTES),
                        name,
                    },
                    partitions,
                }
            })
            .collect();

        Self { disks }
    }

    /// The disk the device is, or is a partition of.
    fn disk_of_device(&self, device: &str) -> Option<&str> {
        self.disks
            .iter()
            .find(|disk| disk.disk.name == device || disk.partitions.iter().any(|partition| partition == device))
            .map(|disk| disk.disk.name.as_str())
    }

    /// The disk a mount's `source` is on, following device mapper and RAID devices down to the partitions they're
    /// stacked on, e.g. an LVM volume on an encrypted partition. `None` when it isn't on a disk, or is on several.
    fn disk_of_source(&self, sys_block_root: &Path, source: &str) -> Option<String> {
        let mut pending = vec![device_name(source)?];
        let mut seen = HashSet::new();
        let mut disks = BTreeSet::new();

        while let Some(device) = pending.pop() {
            if !seen.insert(device.clone()) {
                continue;
            }
            if let Some(disk) = self.disk_of_device(&device) {
                disks.insert(disk.to_string());
                continue;
            }
            // Only device mapper and RAID devices are stacked on others, and only they are listed in `sys_block_root`.
            if let Ok(slaves) = fs::read_dir(sys_block_root.join(&device).join("slaves")) {
                pending.extend(slaves.flatten().filter_map(|slave| slave.file_name().into_string().ok()));
            }
        }

        if disks.len() == 1 {
            disks.pop_first()
        } else {
            None
        }
    }

    /// The mounts, each with the disk it's on.
    fn assign_disks(&self, sys_block_root: &Path, mounts: Vec<MountUsage>) -> Vec<MountUsage> {
        mounts
            .into_iter()
            .map(|mount| MountUsage {
                disk: self.disk_of_source(sys_block_root, &mount.source),
                ..mount
            })
            .collect()
    }
}

/// The volumes on the disks in `layout` that have a filesystem, or are encrypted, and aren't in use. One that is
/// mounted, or is under a mount, e.g. the encrypted partition under an LVM volume, is in use. So is one that something
/// is stacked on, e.g. an unlocked encrypted partition.
fn unmounted_volumes(
    sys_block_root: &Path,
    udev_data_root: &Path,
    layout: &DiskLayout,
    mounts: &[MountEntry],
) -> Vec<UnmountedVolume> {
    let in_use = mounted_devices(sys_block_root, mounts);
    let mut volumes = Vec::new();

    for LaidOutDisk { disk, partitions } in &layout.disks {
        let disk_dir = sys_block_root.join(&disk.name);

        // A disk without partitions can have a filesystem on the whole of it, e.g. some USB sticks.
        let candidates = if partitions.is_empty() {
            vec![(disk.name.clone(), disk_dir.clone())]
        } else {
            partitions.iter().map(|partition| (partition.clone(), disk_dir.join(partition))).collect()
        };

        for (name, dir) in candidates {
            if in_use.contains(&name) || has_holders(&dir) {
                continue;
            }
            if let Some(volume) = read_volume(&name, &disk.name, &dir, udev_data_root) {
                volumes.push(volume);
            }
        }
    }

    volumes
}

/// Whether something is stacked on the device, e.g. it's an unlocked encrypted partition or an LVM physical volume.
fn has_holders(device_dir: &Path) -> bool {
    fs::read_dir(device_dir.join("holders")).is_ok_and(|mut holders| holders.next().is_some())
}

/// The volume on the device in `device_dir`, if udev found a filesystem or an encrypted volume on it.
fn read_volume(name: &str, disk: &str, device_dir: &Path, udev_data_root: &Path) -> Option<UnmountedVolume> {
    let device_number = fs::read_to_string(device_dir.join("dev")).ok()?;
    let data = fs::read_to_string(udev_data_root.join(format!("b{}", device_number.trim()))).ok()?;
    let properties: HashSet<(&str, &str)> = data
        .lines()
        .filter_map(|line| line.strip_prefix("E:")?.split_once('='))
        .collect();
    let property = |key: &str| properties.iter().find(|(k, _)| *k == key).map(|(_, value)| *value);

    // Swap, RAID members and the like are "other".
    let is_locked = match property("ID_FS_USAGE")? {
        "filesystem" => false,
        "crypto" => true,
        _ => return None,
    };

    let sectors: u64 = fs::read_to_string(device_dir.join("size")).ok()?.trim().parse().ok()?;
    let label = property("ID_FS_LABEL_ENC")
        .map(decode_udev_string)
        .or_else(|| property("ID_FS_LABEL").map(str::to_string))
        .filter(|label| !label.is_empty());

    Some(UnmountedVolume {
        name: name.to_string(),
        disk: disk.to_string(),
        label,
        size_bytes: sectors.saturating_mul(SECTOR_SIZE_BYTES),
        is_locked,
    })
}

/// udev escapes characters such as spaces in its values as `\x20`.
fn decode_udev_string(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\\' && bytes.get(index + 1) == Some(&b'x') {
            if let Some(byte) = value.get(index + 2..index + 4).and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                decoded.push(byte);
                index += 4;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fake `/sys/block` and `/run/udev/data`.
    struct FakeSystem {
        root: tempfile::TempDir,
        next_minor: u32,
    }

    impl FakeSystem {
        fn new() -> Self {
            let root = tempfile::tempdir().unwrap();
            fs::create_dir_all(root.path().join("block")).unwrap();
            fs::create_dir_all(root.path().join("udev")).unwrap();
            Self { root, next_minor: 0 }
        }

        fn sys_block(&self) -> std::path::PathBuf {
            self.root.path().join("block")
        }

        fn udev(&self) -> std::path::PathBuf {
            self.root.path().join("udev")
        }

        /// A device of `sectors`, with the udev properties.
        fn add_device(&mut self, dir: &Path, sectors: u64, udev_properties: &[&str]) {
            fs::create_dir_all(dir.join("holders")).unwrap();
            fs::write(dir.join("size"), format!("{sectors}\n")).unwrap();
            self.next_minor += 1;
            let device_number = format!("8:{}", self.next_minor);
            fs::write(dir.join("dev"), format!("{device_number}\n")).unwrap();
            let data: String = udev_properties.iter().map(|property| format!("E:{property}\n")).collect();
            fs::write(self.udev().join(format!("b{device_number}")), data).unwrap();
        }

        fn add_disk(&mut self, name: &str, sectors: u64, udev_properties: &[&str]) {
            let dir = self.sys_block().join(name);
            fs::create_dir_all(dir.join("device")).unwrap();
            self.add_device(&dir, sectors, udev_properties);
        }

        fn add_partition(&mut self, disk: &str, name: &str, sectors: u64, udev_properties: &[&str]) {
            let dir = self.sys_block().join(disk).join(name);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("partition"), "1\n").unwrap();
            self.add_device(&dir, sectors, udev_properties);
        }

        fn volumes(&self, mounts: &[MountEntry]) -> Vec<UnmountedVolume> {
            let layout = DiskLayout::read(&self.sys_block());
            unmounted_volumes(&self.sys_block(), &self.udev(), &layout, mounts)
        }
    }

    const EXT4: &[&str] = &["ID_FS_TYPE=ext4", "ID_FS_USAGE=filesystem"];
    const LUKS: &[&str] = &["ID_FS_TYPE=crypto_LUKS", "ID_FS_USAGE=crypto"];
    const SWAP: &[&str] = &["ID_FS_TYPE=swap", "ID_FS_USAGE=other"];
    const PARTITION_TABLE: &[&str] = &["ID_PART_TABLE_TYPE=gpt"];

    fn mount(source: &str) -> MountEntry {
        MountEntry {
            source: source.to_string(),
            mount_point: "/".to_string(),
            fs_type: "ext4".to_string(),
        }
    }

    fn volume(name: &str, disk: &str, label: Option<&str>, sectors: u64, is_locked: bool) -> UnmountedVolume {
        UnmountedVolume {
            name: name.to_string(),
            disk: disk.to_string(),
            label: label.map(str::to_string),
            size_bytes: sectors * 512,
            is_locked,
        }
    }

    #[test]
    fn lists_the_unmounted_filesystems_and_locked_volumes() {
        let mut system = FakeSystem::new();
        system.add_disk("sda", 3000, PARTITION_TABLE);
        system.add_partition("sda", "sda1", 1000, &["ID_FS_TYPE=ext4", "ID_FS_USAGE=filesystem", "ID_FS_LABEL_ENC=My\\x20Backup"]);
        system.add_partition("sda", "sda2", 1000, LUKS);
        system.add_partition("sda", "sda3", 500, SWAP);
        system.add_partition("sda", "sda4", 500, &[]);

        assert_eq!(
            system.volumes(&[]),
            [volume("sda1", "sda", Some("My Backup"), 1000, false), volume("sda2", "sda", None, 1000, true)]
        );
    }

    #[test]
    fn leaves_out_the_volumes_in_use() {
        let mut system = FakeSystem::new();
        system.add_disk("nvme0n1", 3000, PARTITION_TABLE);
        system.add_partition("nvme0n1", "nvme0n1p1", 1000, EXT4);
        // Unlocked, but not mounted.
        system.add_partition("nvme0n1", "nvme0n1p2", 1000, LUKS);
        fs::write(system.sys_block().join("nvme0n1").join("nvme0n1p2").join("holders").join("dm-0"), "").unwrap();
        // Under the mounted LVM volume dm-2.
        system.add_partition("nvme0n1", "nvme0n1p3", 1000, LUKS);
        let slaves = system.sys_block().join("dm-2").join("slaves");
        fs::create_dir_all(&slaves).unwrap();
        fs::write(slaves.join("nvme0n1p3"), "").unwrap();

        let mounts = [mount("/dev/nonexistent/nvme0n1p1"), mount("/dev/nonexistent/dm-2")];

        assert_eq!(system.volumes(&mounts), []);
    }

    #[test]
    fn a_disk_without_partitions_can_be_a_volume() {
        let mut system = FakeSystem::new();
        system.add_disk("sdb", 2000, EXT4);
        // An empty card reader.
        system.add_disk("sdc", 0, EXT4);

        assert_eq!(system.volumes(&[]), [volume("sdb", "sdb", None, 2000, false)]);
    }

    fn usage(source: &str) -> MountUsage {
        MountUsage {
            source: source.to_string(),
            mount_point: "/".to_string(),
            total_bytes: 0,
            available_bytes: 0,
            disk: None,
        }
    }

    #[test]
    fn finds_the_disk_each_mount_is_on() {
        let mut system = FakeSystem::new();
        system.add_disk("nvme0n1", 4000, PARTITION_TABLE);
        fs::write(system.sys_block().join("nvme0n1").join("device").join("model"), "Fast  SSD \n").unwrap();
        system.add_partition("nvme0n1", "nvme0n1p1", 1000, EXT4);
        system.add_partition("nvme0n1", "nvme0n1p2", 1000, LUKS);
        system.add_disk("sda", 2000, EXT4);
        system.add_disk("sdb", 2000, PARTITION_TABLE);
        system.add_partition("sdb", "sdb1", 2000, EXT4);
        // An LVM volume on an encrypted partition.
        let add_mapper = |name: &str, slaves: &[&str]| {
            let dir = system.sys_block().join(name).join("slaves");
            fs::create_dir_all(&dir).unwrap();
            for slave in slaves {
                fs::write(dir.join(slave), "").unwrap();
            }
        };
        add_mapper("dm-0", &["nvme0n1p2"]);
        add_mapper("dm-1", &["dm-0"]);
        // An LVM volume across two disks.
        add_mapper("dm-2", &["sda", "sdb1"]);

        let layout = DiskLayout::read(&system.sys_block());
        let disks: Vec<Option<String>> = layout
            .assign_disks(
                &system.sys_block(),
                vec![
                    usage("/dev/nonexistent/dm-1"),
                    usage("/dev/nonexistent/nvme0n1p1"),
                    usage("/dev/nonexistent/sda"),
                    usage("/dev/nonexistent/dm-2"),
                    usage("server:/export"),
                ],
            )
            .into_iter()
            .map(|mount| mount.disk)
            .collect();

        let some = |disk: &str| Some(disk.to_string());
        assert_eq!(disks, [some("nvme0n1"), some("nvme0n1"), some("sda"), None, None]);

        let first = &layout.disks[0].disk;
        assert_eq!(
            first,
            &StorageDisk {
                name: "nvme0n1".to_string(),
                model: Some("Fast SSD".to_string()),
                size_bytes: 4000 * 512,
            }
        );
    }

    /// Lists the disks, and what's on them, of the machine the tests run on. Run with
    /// `cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn reads_the_disks_of_this_machine() {
        let mounts = storage_mounts(&ProcMountsReader.read().unwrap());
        let mount_points = mounts.into_iter().map(|mount| mount.mount_point).collect();
        let reading = StorageReader::new(mount_points).read_detailed().unwrap();
        for disk in &reading.disks {
            println!("{disk:?}");
        }
        for mount in &reading.mounts {
            println!("    {} on {:?} ({})", mount.mount_point, mount.disk, mount.source);
        }
        for volume in &reading.unmounted {
            println!("    {} on {} (not mounted)", volume.name, volume.disk);
        }
    }

    #[test]
    fn decodes_udev_escapes() {
        assert_eq!(decode_udev_string("My\\x20Disk"), "My Disk");
        assert_eq!(decode_udev_string("plain"), "plain");
        assert_eq!(decode_udev_string("bad\\xZZ"), "bad\\xZZ");
    }

    /// Only the counted mounts are shown, so the rest are left out of every read, and only a detailed read finds the
    /// disks. Reads this machine's mounts, which can be anything, so only checks what's left out.
    #[test]
    fn only_reads_the_counted_mounts() {
        let reader = StorageReader::new(vec!["/".to_string()]);

        let reading = reader.read().unwrap();
        assert!(reading.mounts.iter().all(|mount| mount.mount_point == "/"));
        assert!(reading.unmounted.is_empty());
        assert!(reading.disks.is_empty());

        let detailed = reader.read_detailed().unwrap();
        assert!(detailed.mounts.iter().all(|mount| mount.mount_point == "/"));
        // Every disk a mount is on is listed.
        for disk in detailed.mounts.iter().filter_map(|mount| mount.disk.as_ref()) {
            assert!(detailed.disks.iter().any(|listed| &listed.name == disk), "{disk}");
        }
    }
}
