use crate::sensors::proc_mounts_reader::{storage_mounts, ProcMountsReader};
use crate::sensors::sensor_traits::SensorReader;
use log::warn;

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

/// Reads the space on every mounted storage filesystem, in mount table order. Only mounted filesystems are read, so
/// an unplugged drive doesn't report the space of the filesystem its mount point directory is on.
pub struct StorageReader;

impl SensorReader for StorageReader {
    type Output = Vec<MountUsage>;

    fn read(&self) -> Result<Self::Output, String> {
        let mounts = storage_mounts(&ProcMountsReader.read()?);

        let usage = mounts
            .into_iter()
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

        Ok(usage)
    }
}
