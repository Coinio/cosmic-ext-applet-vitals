use crate::sensors::proc_disk_stats_reader::{has_media, is_disk, mounted_devices, SYS_BLOCK_ROOT};
use crate::sensors::proc_mounts_reader::{storage_mounts, MountEntry, ProcMountsReader};
use crate::sensors::sensor_traits::SensorReader;
use log::warn;
use std::collections::HashSet;
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
}

/// A partition, or a disk without partitions, with a filesystem on it that isn't mounted, e.g. a second drive, or an
/// encrypted partition that hasn't been unlocked. Its free space can't be read until it's mounted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnmountedVolume {
    /// The device name, e.g. `sda1`.
    pub name: String,
    /// The filesystem's label, if it has one.
    pub label: Option<String>,
    pub size_bytes: u64,
    /// An encrypted volume that hasn't been unlocked.
    pub is_locked: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StorageReading {
    /// Every mounted storage filesystem, or only the counted ones when the read isn't detailed, in mount table order.
    pub mounts: Vec<MountUsage>,
    /// The volumes that aren't mounted, by disk and then partition. Only found by a detailed read.
    pub unmounted: Vec<UnmountedVolume>,
}

/// Reads the space on the mounted storage filesystems. Only mounted filesystems are read, so an unplugged drive doesn't
/// report the space of the filesystem its mount point directory is on.
///
/// The panel only shows the space on the counted mount points, so only those are read, except by a detailed read for
/// the detail popup, which reads every mount, and finds the volumes that aren't mounted. Reading the space on a
/// network mount asks its server, and finding the volumes reads a few sysfs and udev files for every partition.
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
            .filter(|mount| detailed || self.counted_mount_points.contains(&mount.mount_point))
            .filter_map(|mount| match rustix::fs::statvfs(mount.mount_point.as_str()) {
                Ok(stats) => Some(MountUsage {
                    total_bytes: stats.f_blocks.saturating_mul(stats.f_frsize),
                    available_bytes: stats.f_bavail.saturating_mul(stats.f_frsize),
                    source: mount.source,
                    mount_point: mount.mount_point,
                }),
                Err(err) => {
                    warn!("Unable to read the space on {}: {}", mount.mount_point, err);
                    None
                }
            })
            .collect();

        let unmounted = if detailed {
            unmounted_volumes(Path::new(SYS_BLOCK_ROOT), Path::new(UDEV_DATA_ROOT), &all_mounts)
        } else {
            Vec::new()
        };

        Ok(StorageReading { mounts, unmounted })
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

/// The volumes on the disks in `sys_block_root` that have a filesystem, or are encrypted, and aren't in use. One that
/// is mounted, or is under a mount, e.g. the encrypted partition under an LVM volume, is in use. So is one that
/// something is stacked on, e.g. an unlocked encrypted partition.
fn unmounted_volumes(sys_block_root: &Path, udev_data_root: &Path, mounts: &[MountEntry]) -> Vec<UnmountedVolume> {
    let Ok(entries) = fs::read_dir(sys_block_root) else {
        return Vec::new();
    };
    let mut disks: Vec<String> = entries.flatten().filter_map(|entry| entry.file_name().into_string().ok()).collect();
    disks.retain(|disk| is_disk(sys_block_root, disk) && has_media(sys_block_root, disk));
    disks.sort();

    let in_use = mounted_devices(sys_block_root, mounts);
    let mut volumes = Vec::new();

    for disk in disks {
        let disk_dir = sys_block_root.join(&disk);
        let mut partitions: Vec<String> = fs::read_dir(&disk_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|entry| entry.path().join("partition").exists())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        partitions.sort();

        // A disk without partitions can have a filesystem on the whole of it, e.g. some USB sticks.
        let candidates = if partitions.is_empty() {
            vec![(disk.clone(), disk_dir.clone())]
        } else {
            partitions.into_iter().map(|partition| (partition.clone(), disk_dir.join(partition))).collect()
        };

        for (name, dir) in candidates {
            if in_use.contains(&name) || has_holders(&dir) {
                continue;
            }
            if let Some(volume) = read_volume(&name, &dir, udev_data_root) {
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
fn read_volume(name: &str, device_dir: &Path, udev_data_root: &Path) -> Option<UnmountedVolume> {
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
            unmounted_volumes(&self.sys_block(), &self.udev(), mounts)
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

    fn volume(name: &str, label: Option<&str>, sectors: u64, is_locked: bool) -> UnmountedVolume {
        UnmountedVolume {
            name: name.to_string(),
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
            [volume("sda1", Some("My Backup"), 1000, false), volume("sda2", None, 1000, true)]
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

        assert_eq!(system.volumes(&[]), [volume("sdb", None, 2000, false)]);
    }

    #[test]
    fn decodes_udev_escapes() {
        assert_eq!(decode_udev_string("My\\x20Disk"), "My Disk");
        assert_eq!(decode_udev_string("plain"), "plain");
        assert_eq!(decode_udev_string("bad\\xZZ"), "bad\\xZZ");
    }

    /// The panel only shows the counted mounts, so a read that isn't detailed leaves the rest out. Reads this machine's
    /// mounts, which can be anything, so only checks what's left out.
    #[test]
    fn only_a_detailed_read_reads_every_mount() {
        let reader = StorageReader::new(vec!["/".to_string()]);

        let reading = reader.read().unwrap();
        assert!(reading.mounts.iter().all(|mount| mount.mount_point == "/"));
        assert!(reading.unmounted.is_empty());

        let detailed = reader.read_detailed().unwrap();
        assert!(detailed.mounts.len() >= reading.mounts.len());
    }
}
