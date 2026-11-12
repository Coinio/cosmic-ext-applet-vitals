use crate::sensors::proc_mounts_reader::MountEntry;
use crate::sensors::sensor_traits::SensorReader;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

const PROC_DISK_STATS_FILE: &str = "/proc/diskstats";

/// Where the kernel lists block devices, including each disk's model.
pub const SYS_BLOCK_ROOT: &str = "/sys/block";

const DISK_NAME_INDEX: usize = 2;
const SECTORS_READ_INDEX: usize = 5;
const SECTORS_WRITTEN_INDEX: usize = 9;
const IO_TIME_MS_INDEX: usize = 12;

#[derive(Clone, Debug, Default)]
pub struct ProcDiskStatsStatus {
    pub device_name: String,
    pub sectors_read: u64,
    pub sectors_written: u64,
    /// The milliseconds the device has spent with I/O in flight, which is how long it's been busy.
    pub io_time_ms: u64,
}

impl ProcDiskStatsStatus {
    #[cfg(test)]
    pub fn new(device_name: String, sectors_read: u64, sectors_written: u64) -> Self {
        Self {
            device_name,
            sectors_read,
            sectors_written,
            io_time_ms: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcDiskStats {
    pub device_statuses: Vec<ProcDiskStatsStatus>
}

impl ProcDiskStats {
    pub fn new(device_statuses: Vec<ProcDiskStatsStatus>) -> Self {
        Self {
            device_statuses
        }
    }
}

#[derive(Default, Debug)]
pub struct ProcDiskStatsReader;

impl SensorReader for ProcDiskStatsReader {
    type Output = ProcDiskStats;

    fn read(&self) -> Result<Self::Output, String> {
        let path = Path::new(PROC_DISK_STATS_FILE);

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(e) => return Err(format!("Unable to open file: {} {}", PROC_DISK_STATS_FILE, e)),
        };

        let mut contents = String::new();

        if let Err(_) = file.read_to_string(&mut contents) {
            return Err(format!("Unable to read {}", PROC_DISK_STATS_FILE));
        }

        let mut statuses = Vec::new();

        for line in contents.lines() {
            if line.trim().is_empty() { continue; }
            let device_status = self.parse_disk_stats_line(line)?;

            statuses.push(device_status);
        }

        Ok(ProcDiskStats::new(statuses))
    }
}

impl ProcDiskStatsReader {
    fn parse_disk_stats_line(&self, line: &str) -> Result<ProcDiskStatsStatus, String> {
        let values: Vec<&str> = line
            .split_whitespace()
            .collect();

        if values.len() < 20 {
            return Err(format!("Invalid file format {}", PROC_DISK_STATS_FILE));
        }

        let device_name = values[DISK_NAME_INDEX].to_string();
        let sectors_read = values[SECTORS_READ_INDEX].parse::<u64>().unwrap_or_default();
        let sectors_written = values[SECTORS_WRITTEN_INDEX].parse::<u64>().unwrap_or_default();
        let io_time_ms = values[IO_TIME_MS_INDEX].parse::<u64>().unwrap_or_default();

        Ok(ProcDiskStatsStatus {
            device_name,
            sectors_read,
            sectors_written,
            io_time_ms,
        })
    }
}

/// The disk's model, e.g. "Samsung SSD 980 1TB", where the driver reports one. SATA drivers pad the vendor and model
/// to fixed widths, so runs of spaces are collapsed.
pub fn read_disk_model(sys_block_root: &Path, device_name: &str) -> Option<String> {
    let model = fs::read_to_string(sys_block_root.join(device_name).join("device").join("model")).ok()?;
    let model = model.split_whitespace().collect::<Vec<_>>().join(" ");
    (!model.is_empty()).then_some(model)
}

/// Whether the block device is a disk, e.g. sda, nvme0n1, mmcblk0 or vda, rather than a partition or a virtual device.
/// Partitions aren't listed in `sys_block_root`, and virtual devices such as loop, zram and device mapper ones have no
/// `device` link to hardware. An eMMC's boot partitions are listed as disks of their own, but are part of the eMMC.
pub fn is_disk(sys_block_root: &Path, device_name: &str) -> bool {
    if device_name.starts_with("mmcblk") && device_name.contains("boot") {
        return false;
    }
    sys_block_root.join(device_name).join("device").exists()
}

/// Whether the disk has media in it. A card reader without a card, or an optical drive without a disc, has a size of 0.
pub fn has_media(sys_block_root: &Path, device_name: &str) -> bool {
    fs::read_to_string(sys_block_root.join(device_name).join("size"))
        .ok()
        .and_then(|size| size.trim().parse::<u64>().ok())
        .is_some_and(|sectors| sectors > 0)
}

/// The block devices that the mounts are on, and the devices those are stacked on, e.g. the encrypted partition
/// under an LVM volume. By kernel name, e.g. `nvme0n1p2` for `/dev/nvme0n1p2`, and `dm-1` for `/dev/mapper/vg-root`.
pub fn mounted_devices(sys_block_root: &Path, mounts: &[MountEntry]) -> HashSet<String> {
    let mut pending: Vec<String> = mounts
        .iter()
        .filter(|mount| mount.source.starts_with("/dev/"))
        .filter_map(|mount| {
            // `/dev/mapper` and `/dev/disk/by-*` names are links to the kernel's device node.
            let path = fs::canonicalize(&mount.source).unwrap_or_else(|_| PathBuf::from(&mount.source));
            path.file_name()?.to_str().map(str::to_string)
        })
        .collect();

    let mut devices = HashSet::new();
    while let Some(device) = pending.pop() {
        if !devices.insert(device.clone()) {
            continue;
        }
        // Only device mapper and RAID devices are stacked on others, and only they are listed in `sys_block_root`.
        if let Ok(slaves) = fs::read_dir(sys_block_root.join(&device).join("slaves")) {
            pending.extend(slaves.flatten().filter_map(|slave| slave.file_name().into_string().ok()));
        }
    }
    devices
}

/// Whether the disk, or one of its partitions, is in `devices`.
pub fn is_disk_in(sys_block_root: &Path, disk_name: &str, devices: &HashSet<String>) -> bool {
    devices.iter().any(|device| {
        // A partition's name starts with its disk's, e.g. sda1 or nvme0n1p1, and it's listed under the disk.
        device == disk_name
            || (device.starts_with(disk_name) && sys_block_root.join(disk_name).join(device).join("partition").exists())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_disk_stats_line() {
        let line = " 259       0 nvme0n1 120 5 3000 40 200 7 4000 90 0 1234 130 0 0 0 0 10 2";
        let status = ProcDiskStatsReader.parse_disk_stats_line(line).unwrap();

        assert_eq!(status.device_name, "nvme0n1");
        assert_eq!(status.sectors_read, 3000);
        assert_eq!(status.sectors_written, 4000);
        assert_eq!(status.io_time_ms, 1234);
    }

    #[test]
    fn reads_the_trimmed_model() {
        let root = tempfile::tempdir().unwrap();
        let device = root.path().join("sda").join("device");
        fs::create_dir_all(&device).unwrap();
        fs::write(device.join("model"), "WDC  WDS100T2B0A    \n").unwrap();
        fs::create_dir_all(root.path().join("nvme0n1").join("device")).unwrap();
        fs::write(root.path().join("nvme0n1").join("device").join("model"), "  \n").unwrap();

        assert_eq!(read_disk_model(root.path(), "sda").as_deref(), Some("WDC WDS100T2B0A"));
        assert_eq!(read_disk_model(root.path(), "nvme0n1"), None);
        assert_eq!(read_disk_model(root.path(), "sdb"), None);
    }

    /// A disk in a fake `/sys/block`, with partitions and a size in sectors.
    fn add_disk(root: &Path, name: &str, partitions: &[&str], size: u64) {
        let disk = root.join(name);
        fs::create_dir_all(disk.join("device")).unwrap();
        fs::write(disk.join("size"), format!("{size}\n")).unwrap();
        for partition in partitions {
            fs::create_dir_all(disk.join(partition)).unwrap();
            fs::write(disk.join(partition).join("partition"), "1\n").unwrap();
        }
    }

    /// A device mapper device in a fake `/sys/block`, stacked on `slaves`.
    fn add_mapper(root: &Path, name: &str, slaves: &[&str]) {
        let slaves_dir = root.join(name).join("slaves");
        fs::create_dir_all(&slaves_dir).unwrap();
        for slave in slaves {
            fs::write(slaves_dir.join(slave), "").unwrap();
        }
    }

    fn mount(source: &str) -> MountEntry {
        MountEntry {
            source: source.to_string(),
            mount_point: "/".to_string(),
            fs_type: "ext4".to_string(),
        }
    }

    #[test]
    fn finds_the_disks() {
        let root = tempfile::tempdir().unwrap();
        for disk in ["sda", "nvme0n1", "mmcblk0", "mmcblk0boot0", "vda"] {
            add_disk(root.path(), disk, &[], 100);
        }
        fs::create_dir_all(root.path().join("loop0")).unwrap();
        add_mapper(root.path(), "dm-0", &["sda1"]);

        for disk in ["sda", "nvme0n1", "mmcblk0", "vda"] {
            assert!(is_disk(root.path(), disk), "{disk}");
        }
        for device in ["sda1", "mmcblk0boot0", "loop0", "dm-0", "zram0"] {
            assert!(!is_disk(root.path(), device), "{device}");
        }
    }

    #[test]
    fn a_disk_without_media_has_no_size() {
        let root = tempfile::tempdir().unwrap();
        add_disk(root.path(), "sda", &[], 1000);
        add_disk(root.path(), "sdb", &[], 0);

        assert!(has_media(root.path(), "sda"));
        assert!(!has_media(root.path(), "sdb"));
        assert!(!has_media(root.path(), "sdc"));
    }

    #[test]
    fn finds_the_mounted_disks_through_stacked_devices() {
        let root = tempfile::tempdir().unwrap();
        add_disk(root.path(), "nvme0n1", &["nvme0n1p1", "nvme0n1p2", "nvme0n1p3"], 100);
        add_disk(root.path(), "sda", &["sda1"], 100);
        add_disk(root.path(), "sdb", &[], 100);
        add_disk(root.path(), "sdc", &[], 100);
        add_disk(root.path(), "sdd", &["sdd1"], 100);
        // An LVM volume on an encrypted partition.
        add_mapper(root.path(), "dm-0", &["nvme0n1p3"]);
        add_mapper(root.path(), "dm-1", &["dm-0"]);

        // The device nodes don't exist, so their names are used as they are.
        let mounts = [
            mount("/dev/nonexistent/dm-1"),
            mount("/dev/nonexistent/sda1"),
            // A filesystem on the whole disk.
            mount("/dev/nonexistent/sdb"),
            // Not a block device.
            mount("tmpfs"),
        ];
        let devices = mounted_devices(root.path(), &mounts);

        assert!(devices.contains("nvme0n1p3"));
        for disk in ["nvme0n1", "sda", "sdb"] {
            assert!(is_disk_in(root.path(), disk, &devices), "{disk}");
        }
        // sdc has no partitions, and sdd's isn't mounted.
        assert!(!is_disk_in(root.path(), "sdc", &devices));
        assert!(!is_disk_in(root.path(), "sdd", &devices));
    }
}
