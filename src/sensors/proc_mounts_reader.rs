use crate::sensors::sensor_traits::SensorReader;
use std::fs;

const MOUNTS_FILE: &str = "/proc/self/mounts";

/// Filesystem types that don't hold user storage, i.e. kernel interfaces, RAM-backed or read-only images.
const NON_STORAGE_FS_TYPES: &[&str] = &[
    "autofs",
    "binfmt_misc",
    "bpf",
    "cgroup",
    "cgroup2",
    "configfs",
    "debugfs",
    "devpts",
    "devtmpfs",
    "efivarfs",
    "fusectl",
    "hugetlbfs",
    "mqueue",
    "nsfs",
    "overlay",
    "proc",
    "pstore",
    "ramfs",
    "rpc_pipefs",
    "securityfs",
    "selinuxfs",
    "squashfs",
    "sysfs",
    "tmpfs",
    "tracefs",
    "fuse.gvfsd-fuse",
    "fuse.portal",
];

/// A single entry from the mount table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountEntry {
    /// What is mounted, e.g. `/dev/nvme0n1p2`, or a pseudo name such as `tmpfs`
    pub source: String,
    /// Where it is mounted, e.g. `/home`
    pub mount_point: String,
    pub fs_type: String,
}

impl MountEntry {
    /// Whether the mount is a filesystem that holds storage, rather than a kernel interface or RAM-backed.
    pub fn is_storage(&self) -> bool {
        !NON_STORAGE_FS_TYPES.contains(&self.fs_type.as_str())
    }
}

/// Reads the mount table of the current process.
pub struct ProcMountsReader;

impl SensorReader for ProcMountsReader {
    type Output = Vec<MountEntry>;

    fn read(&self) -> Result<Self::Output, String> {
        let contents =
            fs::read_to_string(MOUNTS_FILE).map_err(|err| format!("Unable to read {MOUNTS_FILE}: {err}"))?;

        Ok(parse_mounts(&contents))
    }
}

/// The storage mounts, in mount table order, with each mount point listed once.
pub fn storage_mounts(mounts: &[MountEntry]) -> Vec<MountEntry> {
    let mut storage: Vec<MountEntry> = Vec::new();

    for mount in mounts.iter().filter(|mount| mount.is_storage()) {
        // A later mount on the same mount point hides the earlier one, so it's the one to report.
        storage.retain(|existing| existing.mount_point != mount.mount_point);
        storage.push(mount.clone());
    }

    storage
}

/// Parse the contents of `/proc/self/mounts`. Malformed lines are skipped.
pub fn parse_mounts(contents: &str) -> Vec<MountEntry> {
    contents
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let source = fields.next()?;
            let mount_point = fields.next()?;
            let fs_type = fields.next()?;

            Some(MountEntry {
                source: unescape(source),
                mount_point: unescape(mount_point),
                fs_type: unescape(fs_type),
            })
        })
        .collect()
}

/// The kernel escapes spaces, tabs, newlines and backslashes in the mount table as 3 digit octal, e.g. `\040`.
fn unescape(field: &str) -> String {
    let bytes = field.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'\\' && index + 3 < bytes.len() {
            let octal = &bytes[index + 1..index + 4];
            if octal.iter().all(|digit| (b'0'..=b'7').contains(digit)) {
                let value = octal.iter().fold(0u32, |value, digit| value * 8 + u32::from(digit - b'0'));
                if let Ok(value) = u8::try_from(value) {
                    result.push(value);
                    index += 4;
                    continue;
                }
            }
        }
        result.push(bytes[index]);
        index += 1;
    }

    String::from_utf8_lossy(&result).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MOUNTS: &str = "sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
udev /dev devtmpfs rw,nosuid,relatime,size=14273896k,mode=755,inode64 0 0
tmpfs /run tmpfs rw,nosuid,nodev,noexec,relatime,size=3193004k,mode=755,inode64 0 0
/dev/mapper/data-root / ext4 rw,noatime,errors=remount-ro 0 0
cgroup2 /sys/fs/cgroup cgroup2 rw,nosuid,nodev,noexec,relatime 0 0
/dev/nvme0n1p1 /boot/efi vfat rw,relatime,fmask=0077,dmask=0077 0 0
/dev/loop0 /snap/core/1 squashfs ro,nodev,relatime 0 0
/dev/sdb1 /media/user/My\\040Drive ext4 rw,relatime 0 0
portal /run/user/1000/doc fuse.portal rw,nosuid,nodev,relatime 0 0
";

    #[test]
    fn parses_mount_entries() {
        let mounts = parse_mounts(SAMPLE_MOUNTS);

        assert_eq!(mounts.len(), 10);
        assert_eq!(
            mounts[4],
            MountEntry {
                source: "/dev/mapper/data-root".to_string(),
                mount_point: "/".to_string(),
                fs_type: "ext4".to_string(),
            }
        );
    }

    #[test]
    fn unescapes_octal_sequences() {
        assert_eq!(unescape("/media/user/My\\040Drive"), "/media/user/My Drive");
        assert_eq!(unescape("/a\\011b\\134c"), "/a\tb\\c");
        assert_eq!(unescape("/trailing\\"), "/trailing\\");
        assert_eq!(unescape("/short\\04"), "/short\\04");
        assert_eq!(unescape("/not\\999octal"), "/not\\999octal");
    }

    #[test]
    fn storage_mounts_excludes_pseudo_filesystems() {
        let mounts = parse_mounts(SAMPLE_MOUNTS);
        let mount_points: Vec<String> = storage_mounts(&mounts).into_iter().map(|m| m.mount_point).collect();

        assert_eq!(mount_points, vec!["/", "/boot/efi", "/media/user/My Drive"]);
    }

    #[test]
    fn storage_mounts_keeps_the_last_mount_on_a_mount_point() {
        let mounts = parse_mounts(
            "/dev/sda1 /mnt ext4 rw 0 0
/dev/sdb1 /mnt xfs rw 0 0
",
        );

        let storage = storage_mounts(&mounts);

        assert_eq!(storage.len(), 1);
        assert_eq!(storage[0].source, "/dev/sdb1");
    }

    #[test]
    fn skips_malformed_lines() {
        let mounts = parse_mounts("\n/dev/sda1 /\n/dev/sda2 /home ext4 rw 0 0\n");

        assert_eq!(mounts.len(), 1);
        assert_eq!(mounts[0].mount_point, "/home");
    }
}
