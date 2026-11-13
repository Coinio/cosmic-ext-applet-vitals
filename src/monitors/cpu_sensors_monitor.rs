use crate::configuration::cpu::CpuConfiguration;
use crate::sensors::cpu_info_reader::CpuFrequencyReader;
use crate::sensors::hwmon_reader::{
    discover_sensors, find_sensor, read_fan_speed, read_temperature, HwmonSensor, HwmonSensorKind,
};
use log::{info, warn};
use std::path::Path;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CpuSensorStats {
    /// The CPU temperature in degrees Celsius, when there's a sensor for it.
    pub temperature_celsius: Option<f64>,
    /// The CPU fan speed in RPM, when there's a sensor for it.
    pub fan_rpm: Option<u32>,
    /// The current clock speed in kHz, averaged over the threads, when the CPU's frequency driver reports it.
    pub frequency_khz: Option<u64>,
}

/// Reads the CPU temperature, fan speed and clock speed. Only the two chosen sensors are read on each poll, as some
/// drivers are slow to read, or wake their device to do it.
pub struct CpuSensorsMonitor {
    temperature: Option<HwmonSensor>,
    fan: Option<HwmonSensor>,
    frequency: CpuFrequencyReader,
}

impl CpuSensorsMonitor {
    /// Find the configured sensors, or the automatic ones, once. A driver loaded later is picked up the next time
    /// the monitors start, e.g. when the configuration changes.
    pub fn new(hwmon_root: &Path, sysfs_cpu_root: &Path, configuration: &CpuConfiguration) -> Self {
        let temperature = Self::find(hwmon_root, HwmonSensorKind::Temperature, configuration);
        let fan = Self::find(hwmon_root, HwmonSensorKind::Fan, configuration);

        info!(
            "Creating new cpu sensors monitor, temperature: {:?}, fan: {:?}",
            temperature.as_ref().map(HwmonSensor::name),
            fan.as_ref().map(HwmonSensor::name)
        );

        Self {
            temperature,
            fan,
            frequency: CpuFrequencyReader::new(sysfs_cpu_root),
        }
    }

    fn find(hwmon_root: &Path, kind: HwmonSensorKind, configuration: &CpuConfiguration) -> Option<HwmonSensor> {
        let configured = match kind {
            HwmonSensorKind::Temperature => configuration.temperature_sensor.as_ref(),
            HwmonSensorKind::Fan => configuration.fan_sensor.as_ref(),
        };

        let sensor = find_sensor(&discover_sensors(hwmon_root, kind), kind, configured);
        if sensor.is_none() {
            if let Some(configured) = configured {
                warn!("The configured {kind:?} sensor {configured:?} wasn't found");
            }
        }
        sensor
    }

    pub fn poll(&self) -> CpuSensorStats {
        CpuSensorStats {
            temperature_celsius: self.temperature.as_ref().and_then(read_temperature),
            fan_rpm: self.fan.as_ref().and_then(read_fan_speed),
            frequency_khz: self.frequency.read_average_khz(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::hwmon_reader::tests::FakeHwmon;
    use crate::sensors::hwmon_reader::HwmonSensorId;
    use std::fs;

    /// The tests are about the hwmon sensors, so there are no CPUs to read the clock speed of.
    const NO_CPU_ROOT: &str = "/nonexistent/cpu";

    #[test]
    fn reads_the_automatic_sensors() {
        let mut hwmon = FakeHwmon::new();
        hwmon.chip("nvme", "nvme0", &[("temp1", Some("Composite"), "38000")]);
        hwmon.chip("k10temp", "0000:00:18.3", &[("temp1", Some("Tctl"), "52500")]);
        hwmon.chip(
            "nct6683",
            "nct6683.2592",
            &[("fan1", Some("CPU Fan"), "980"), ("fan2", None, "0")],
        );

        let monitor = CpuSensorsMonitor::new(hwmon.path(), Path::new(NO_CPU_ROOT), &CpuConfiguration::default());

        assert_eq!(
            monitor.poll(),
            CpuSensorStats {
                temperature_celsius: Some(52.5),
                fan_rpm: Some(980),
                frequency_khz: None,
            }
        );
    }

    #[test]
    fn reads_the_configured_sensors() {
        let mut hwmon = FakeHwmon::new();
        hwmon.chip(
            "k10temp",
            "0000:00:18.3",
            &[("temp1", Some("Tctl"), "52500"), ("temp3", Some("Tccd1"), "41000")],
        );
        hwmon.chip(
            "nct6775",
            "nct6775.656",
            &[("fan1", None, "700"), ("fan2", None, "1100")],
        );

        let configuration = CpuConfiguration {
            temperature_sensor: Some(HwmonSensorId {
                chip: "k10temp".to_string(),
                device: "0000:00:18.3".to_string(),
                channel: "temp3".to_string(),
            }),
            fan_sensor: Some(HwmonSensorId {
                chip: "nct6775".to_string(),
                device: "nct6775.656".to_string(),
                channel: "fan2".to_string(),
            }),
            ..CpuConfiguration::default()
        };
        let monitor = CpuSensorsMonitor::new(hwmon.path(), Path::new(NO_CPU_ROOT), &configuration);

        assert_eq!(
            monitor.poll(),
            CpuSensorStats {
                temperature_celsius: Some(41.0),
                fan_rpm: Some(1100),
                frequency_khz: None,
            }
        );
    }

    #[test]
    fn has_no_readings_without_sensors() {
        let mut hwmon = FakeHwmon::new();
        // Unlabelled fans, which could be any of the board's fans.
        hwmon.chip("nct6775", "", &[("fan1", None, "700"), ("fan2", None, "1100")]);

        let monitor = CpuSensorsMonitor::new(hwmon.path(), Path::new(NO_CPU_ROOT), &CpuConfiguration::default());

        assert_eq!(monitor.poll(), CpuSensorStats::default());
    }

    #[test]
    fn follows_the_reading_as_it_changes() {
        let mut hwmon = FakeHwmon::new();
        let chip = hwmon.chip("coretemp", "coretemp.0", &[("temp1", Some("Package id 0"), "40000")]);

        let monitor = CpuSensorsMonitor::new(hwmon.path(), Path::new(NO_CPU_ROOT), &CpuConfiguration::default());
        assert_eq!(monitor.poll().temperature_celsius, Some(40.0));

        fs::write(chip.join("temp1_input"), "61000\n").unwrap();
        assert_eq!(monitor.poll().temperature_celsius, Some(61.0));
    }
}
