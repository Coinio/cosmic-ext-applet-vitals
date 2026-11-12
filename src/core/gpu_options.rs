use crate::configuration::gpu::device_to_form_value;
use crate::fl;
use crate::monitors::gpu_monitor::panel_gpu;
use crate::sensors::gpu_reader::{discover_gpus, GpuDevice, GpuPaths};
use std::sync::Arc;

/// The GPUs that can be chosen for the panel, for a dropdown. The first is always automatic, which is the empty form
/// value. Listing them doesn't read them, so it doesn't wake any that are asleep.
#[derive(Debug, Clone, Default)]
pub struct GpuOptions {
    pub labels: Vec<String>,
    /// The form values, parallel to `labels`.
    pub values: Arc<[String]>,
    /// Whether any supported GPU was found, so the settings can explain why there are none.
    pub has_gpus: bool,
}

impl GpuOptions {
    /// The configured GPU is included even if it's missing, so it still shows as selected.
    pub fn new(configured: Option<&str>) -> Self {
        Self::from_gpus(&discover_gpus(&GpuPaths::system()), configured)
    }

    fn from_gpus(gpus: &[GpuDevice], configured: Option<&str>) -> Self {
        let automatic = match panel_gpu(gpus, None) {
            Some(index) => fl!("settings-gpu-automatic", gpu = gpus[index].name.as_str()),
            None => fl!("settings-gpu-automatic-none"),
        };

        let mut labels = vec![automatic];
        let mut values = vec![device_to_form_value(None)];

        for gpu in gpus {
            labels.push(format!("{} ({})", gpu.name, gpu.pci_address));
            values.push(device_to_form_value(Some(&gpu.pci_address)));
        }

        if let Some(configured) = configured {
            if !gpus.iter().any(|gpu| gpu.pci_address == configured) {
                labels.push(fl!("settings-gpu-missing", gpu = configured));
                values.push(device_to_form_value(Some(configured)));
            }
        }

        GpuOptions {
            labels,
            values: values.into(),
            has_gpus: !gpus.is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::gpu_reader::GpuDriver;
    use std::path::PathBuf;

    fn gpu(pci_address: &str, driver: GpuDriver, name: &str) -> GpuDevice {
        GpuDevice {
            pci_address: pci_address.to_string(),
            driver,
            name: name.to_string(),
            device_path: PathBuf::new(),
            hwmon_path: None,
            can_sleep: false,
            autosuspend_delay: None,
            vram_total_bytes: None,
        }
    }

    #[test]
    fn starts_with_automatic_then_each_gpu() {
        let gpus = vec![
            gpu("0000:01:00.0", GpuDriver::Nvidia, "NVIDIA GeForce RTX 4060 Ti"),
            gpu("0000:10:00.0", GpuDriver::Amdgpu, "AMD Raphael"),
        ];

        let options = GpuOptions::from_gpus(&gpus, None);

        assert_eq!(options.labels.len(), options.values.len());
        assert!(options.labels[0].contains("NVIDIA GeForce RTX 4060 Ti"));
        assert_eq!(options.values[0], "");
        assert_eq!(options.labels[2], "AMD Raphael (0000:10:00.0)");
        assert_eq!(options.values[2], "0000:10:00.0");
        assert!(options.has_gpus);
    }

    #[test]
    fn includes_a_configured_gpu_that_is_missing() {
        let options = GpuOptions::from_gpus(&[], Some("0000:03:00.0"));

        assert_eq!(options.values.len(), 2);
        assert_eq!(options.values[1], "0000:03:00.0");
        assert!(options.labels[1].contains("0000:03:00.0"));
        assert!(!options.has_gpus);
    }
}
