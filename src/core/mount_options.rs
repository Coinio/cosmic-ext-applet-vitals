use crate::fl;
use crate::sensors::proc_mounts_reader::{storage_mounts, MountEntry, ProcMountsReader};
use crate::sensors::sensor_traits::SensorReader;
use log::error;

/// A mount point that can be included in the storage totals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountOption {
    pub mount_point: String,
    /// What is mounted there, e.g. `/dev/sda1 (ext4)`, or that it isn't currently mounted.
    pub description: String,
}

#[derive(Debug, Clone, Default)]
pub struct MountOptions {
    pub options: Vec<MountOption>,
}

impl MountOptions {
    /// The storage mounts, followed by any configured mount points that aren't currently mounted, so they can
    /// still be excluded.
    pub fn new(configured: &[String]) -> Self {
        let mounts = ProcMountsReader.read().unwrap_or_else(|err| {
            error!("Failed to read the mounted filesystems: {}", err);
            Vec::new()
        });

        Self::from_mounts(&mounts, configured)
    }

    fn from_mounts(mounts: &[MountEntry], configured: &[String]) -> Self {
        let mut options: Vec<MountOption> = storage_mounts(mounts)
            .into_iter()
            .map(|mount| MountOption {
                description: format!("{} ({})", mount.source, mount.fs_type),
                mount_point: mount.mount_point,
            })
            .collect();

        for mount_point in configured {
            if !options.iter().any(|option| &option.mount_point == mount_point) {
                options.push(MountOption {
                    mount_point: mount_point.clone(),
                    description: fl!("settings-storage-not-mounted"),
                });
            }
        }

        MountOptions { options }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::proc_mounts_reader::parse_mounts;

    #[test]
    fn lists_storage_mounts_then_unmounted_configured_mounts() {
        let mounts = parse_mounts(
            "proc /proc proc rw 0 0
/dev/sda2 / ext4 rw 0 0
/dev/sda1 /boot/efi vfat rw 0 0
",
        );

        let options = MountOptions::from_mounts(&mounts, &["/".to_string(), "/media/usb".to_string()]);
        let mount_points: Vec<&str> = options.options.iter().map(|o| o.mount_point.as_str()).collect();

        assert_eq!(mount_points, vec!["/", "/boot/efi", "/media/usb"]);
        assert_eq!(options.options[0].description, "/dev/sda2 (ext4)");
    }
}
