//! Reads the temperature and fan sensors the kernel's hardware monitoring (hwmon) drivers expose in sysfs. This is the
//! same data `sensors` from lm-sensors shows, without needing libsensors.
//!
//! Each `/sys/class/hwmon/hwmonN` directory is a chip, e.g. `k10temp` for an AMD CPU, with one `tempX_input` file per
//! temperature in millidegrees Celsius, and one `fanX_input` file per fan in RPM. An optional `tempX_label` or
//! `fanX_label` file names the channel.

use crate::sensors::proc_file;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Where the kernel lists the hwmon chips.
pub const HWMON_ROOT: &str = "/sys/class/hwmon";

/// The chips that measure the CPU's own temperature, in order of preference, with the channel labels to prefer on
/// each, as label prefixes in order. A chip without a preferred label uses its first channel.
const CPU_TEMPERATURE_CHIPS: &[(&str, &[&str])] = &[
    // AMD. Older Ryzen CPUs report Tctl with an offset that Tdie doesn't have. Newer ones only have Tctl.
    ("k10temp", &["Tdie", "Tctl"]),
    ("zenpower", &["Tdie", "Tctl"]),
    // Intel. Each core has a temperature too, the package is the hottest of them.
    ("coretemp", &["Package id"]),
    // ARM SoCs, e.g. the Raspberry Pi, name the CPU thermal zone in several ways.
    ("cpu_thermal", &[]),
    ("cpu-thermal", &[]),
    ("cpu0_thermal", &[]),
    ("soc_thermal", &[]),
];

/// Fan labels that name the CPU fan, compared in lower case.
const CPU_FAN_LABELS: &[&str] = &["cpu", "processor"];

/// Graphics cards' chips, whose fans are never the CPU fan.
const GPU_CHIPS: &[&str] = &["amdgpu", "radeon", "nouveau", "i915", "xe"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwmonSensorKind {
    Temperature,
    Fan,
}

impl HwmonSensorKind {
    fn file_prefix(self) -> &'static str {
        match self {
            HwmonSensorKind::Temperature => "temp",
            HwmonSensorKind::Fan => "fan",
        }
    }
}

/// Identifies a sensor across reboots. The `hwmonN` numbers depend on the order the drivers load in, so they can't.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HwmonSensorId {
    /// The chip's driver name, e.g. `k10temp`.
    pub chip: String,
    /// The device the chip belongs to, e.g. its PCI address `0000:00:18.3`, which tells apart identical chips. Empty
    /// when it has none.
    pub device: String,
    /// The channel on the chip, e.g. `temp1`.
    pub channel: String,
}

/// Separates the parts of a sensor id in a settings form value. Sysfs names can't contain a NUL, so it can't clash.
const FORM_VALUE_SEPARATOR: char = '\0';

impl HwmonSensorId {
    /// Encode an optional sensor id as a single settings form value, where `None` is the empty string.
    pub fn to_form_value(id: Option<&HwmonSensorId>) -> String {
        match id {
            None => String::new(),
            Some(id) => {
                [id.chip.as_str(), id.device.as_str(), id.channel.as_str()].join(&FORM_VALUE_SEPARATOR.to_string())
            }
        }
    }

    /// Decode an optional sensor id from a settings form value.
    pub fn from_form_value(value: &str) -> Option<HwmonSensorId> {
        let mut parts = value.split(FORM_VALUE_SEPARATOR);
        let (Some(chip), Some(device), Some(channel), None) = (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return None;
        };

        Some(HwmonSensorId {
            chip: chip.to_string(),
            device: device.to_string(),
            channel: channel.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HwmonSensor {
    pub id: HwmonSensorId,
    /// The channel's name from the driver, e.g. `Tctl` or `CPU Fan`, if it has one.
    pub label: Option<String>,
    /// The file the reading is read from.
    pub input_path: PathBuf,
}

impl HwmonSensor {
    /// The sensor's name for the settings, e.g. `k10temp: Tctl`.
    pub fn name(&self) -> String {
        format!(
            "{}: {}",
            self.id.chip,
            self.label.as_deref().unwrap_or(&self.id.channel)
        )
    }
}

/// Every sensor of `kind` under the hwmon `root`, sorted by chip, device and channel so they're listed the same way
/// on every boot.
pub fn discover_sensors(root: &Path, kind: HwmonSensorKind) -> Vec<HwmonSensor> {
    let Ok(chips) = fs::read_dir(root) else {
        return Vec::new();
    };

    let prefix = kind.file_prefix();
    let mut sensors = Vec::new();

    for chip in chips.flatten() {
        let chip_path = chip.path();
        let Ok(name) = fs::read_to_string(chip_path.join("name")) else {
            continue;
        };
        let device = fs::read_link(chip_path.join("device"))
            .ok()
            .and_then(|device| device.file_name().map(|name| name.to_string_lossy().into_owned()))
            .unwrap_or_default();

        let Ok(files) = fs::read_dir(&chip_path) else {
            continue;
        };

        for file in files.flatten() {
            let file_name = file.file_name();
            let Some(channel) = file_name
                .to_str()
                .and_then(|name| name.strip_suffix("_input"))
                .filter(|channel| is_channel(channel, prefix))
            else {
                continue;
            };

            let label = fs::read_to_string(chip_path.join(format!("{channel}_label")))
                .ok()
                .map(|label| label.trim().to_string())
                .filter(|label| !label.is_empty());

            sensors.push(HwmonSensor {
                id: HwmonSensorId {
                    chip: name.trim().to_string(),
                    device: device.clone(),
                    channel: channel.to_string(),
                },
                label,
                input_path: file.path(),
            });
        }
    }

    sensors.sort_by(|a, b| {
        (&a.id.chip, &a.id.device, channel_number(&a.id.channel)).cmp(&(
            &b.id.chip,
            &b.id.device,
            channel_number(&b.id.channel),
        ))
    });
    sensors
}

/// Whether `channel` is e.g. `temp1` for the `temp` prefix.
fn is_channel(channel: &str, prefix: &str) -> bool {
    channel
        .strip_prefix(prefix)
        .is_some_and(|number| !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()))
}

/// The channel's number, so `temp10` sorts after `temp9`.
fn channel_number(channel: &str) -> u32 {
    channel
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .parse()
        .unwrap_or(u32::MAX)
}

/// The sensor with `id`, or when there's no id, the one picked automatically for `kind`.
pub fn find_sensor(sensors: &[HwmonSensor], kind: HwmonSensorKind, id: Option<&HwmonSensorId>) -> Option<HwmonSensor> {
    match id {
        Some(id) => sensors.iter().find(|sensor| &sensor.id == id).cloned(),
        None => match kind {
            HwmonSensorKind::Temperature => automatic_cpu_temperature(sensors),
            HwmonSensorKind::Fan => automatic_cpu_fan(sensors),
        }
        .cloned(),
    }
}

/// The CPU's own temperature sensor, if the CPU has a known one.
pub fn automatic_cpu_temperature(sensors: &[HwmonSensor]) -> Option<&HwmonSensor> {
    CPU_TEMPERATURE_CHIPS.iter().find_map(|(chip, preferred_labels)| {
        let channels: Vec<&HwmonSensor> = sensors.iter().filter(|sensor| sensor.id.chip == *chip).collect();

        preferred_labels
            .iter()
            .find_map(|preferred| {
                channels.iter().find(|sensor| {
                    sensor
                        .label
                        .as_deref()
                        .is_some_and(|label| label.starts_with(preferred))
                })
            })
            .or_else(|| channels.first())
            .copied()
    })
}

/// The fan labelled as the CPU fan, or failing that, the only fan that isn't a graphics card's. A desktop board
/// usually has several unlabelled fans, which only the user knows the CPU fan among.
pub fn automatic_cpu_fan(sensors: &[HwmonSensor]) -> Option<&HwmonSensor> {
    let labelled = sensors.iter().find(|sensor| {
        sensor.label.as_deref().is_some_and(|label| {
            let label = label.to_lowercase();
            CPU_FAN_LABELS.iter().any(|cpu_label| label.contains(cpu_label))
        })
    });

    labelled.or_else(|| {
        let mut fans = sensors
            .iter()
            .filter(|sensor| !GPU_CHIPS.contains(&sensor.id.chip.as_str()));
        match (fans.next(), fans.next()) {
            (Some(only), None) => Some(only),
            _ => None,
        }
    })
}

/// The temperature in degrees Celsius.
pub fn read_temperature(sensor: &HwmonSensor) -> Option<f64> {
    read_integer(&sensor.input_path).map(|millidegrees| millidegrees as f64 / 1000.0)
}

/// The fan speed in RPM.
pub fn read_fan_speed(sensor: &HwmonSensor) -> Option<u32> {
    read_integer(&sensor.input_path).and_then(|rpm| u32::try_from(rpm).ok())
}

/// Some drivers fail the read, e.g. with `ENODATA`, while a sensor isn't connected.
fn read_integer(path: &Path) -> Option<i64> {
    proc_file::read_value(path)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    /// Builds a fake `/sys/class/hwmon` with chips that link to their devices, as the real one does.
    pub struct FakeHwmon {
        pub root: tempfile::TempDir,
        chips: usize,
    }

    impl FakeHwmon {
        pub fn new() -> Self {
            FakeHwmon {
                root: tempfile::tempdir().unwrap(),
                chips: 0,
            }
        }

        /// Add a chip with `channels` of (channel, label, value). Returns its directory.
        pub fn chip(&mut self, name: &str, device: &str, channels: &[(&str, Option<&str>, &str)]) -> PathBuf {
            let path = self.root.path().join(format!("hwmon{}", self.chips));
            self.chips += 1;
            fs::create_dir_all(&path).unwrap();
            fs::write(path.join("name"), format!("{name}\n")).unwrap();

            if !device.is_empty() {
                let device_path = self.root.path().join("devices").join(device);
                fs::create_dir_all(&device_path).unwrap();
                symlink(&device_path, path.join("device")).unwrap();
            }

            for (channel, label, value) in channels {
                fs::write(path.join(format!("{channel}_input")), format!("{value}\n")).unwrap();
                if let Some(label) = label {
                    fs::write(path.join(format!("{channel}_label")), format!("{label}\n")).unwrap();
                }
            }
            path
        }

        pub fn path(&self) -> &Path {
            self.root.path()
        }
    }

    fn id(chip: &str, device: &str, channel: &str) -> HwmonSensorId {
        HwmonSensorId {
            chip: chip.to_string(),
            device: device.to_string(),
            channel: channel.to_string(),
        }
    }

    fn channels(sensors: &[HwmonSensor]) -> Vec<String> {
        sensors
            .iter()
            .map(|sensor| format!("{}/{}", sensor.id.chip, sensor.id.channel))
            .collect()
    }

    #[test]
    fn discovers_the_sensors_of_each_kind() {
        let mut hwmon = FakeHwmon::new();
        hwmon.chip(
            "nct6683",
            "nct6683.2592",
            &[
                ("fan1", Some("CPU Fan"), "1200"),
                ("fan2", None, "0"),
                ("temp1", None, "35000"),
            ],
        );
        hwmon.chip(
            "k10temp",
            "0000:00:18.3",
            &[("temp1", Some("Tctl"), "45625"), ("temp3", Some("Tccd1"), "39125")],
        );

        let temperatures = discover_sensors(hwmon.path(), HwmonSensorKind::Temperature);
        let fans = discover_sensors(hwmon.path(), HwmonSensorKind::Fan);

        assert_eq!(
            channels(&temperatures),
            vec!["k10temp/temp1", "k10temp/temp3", "nct6683/temp1"]
        );
        assert_eq!(channels(&fans), vec!["nct6683/fan1", "nct6683/fan2"]);

        assert_eq!(temperatures[0].id, id("k10temp", "0000:00:18.3", "temp1"));
        assert_eq!(temperatures[0].label.as_deref(), Some("Tctl"));
        assert_eq!(temperatures[0].name(), "k10temp: Tctl");
        assert_eq!(fans[1].name(), "nct6683: fan2");
    }

    #[test]
    fn ignores_files_that_arent_inputs() {
        let mut hwmon = FakeHwmon::new();
        let chip = hwmon.chip("k10temp", "", &[("temp1", None, "45000")]);
        fs::write(chip.join("temp1_max"), "70000").unwrap();
        fs::write(chip.join("temp_input"), "1").unwrap();
        fs::write(chip.join("pwm1"), "128").unwrap();

        let temperatures = discover_sensors(hwmon.path(), HwmonSensorKind::Temperature);

        assert_eq!(channels(&temperatures), vec!["k10temp/temp1"]);
        assert_eq!(temperatures[0].id.device, "");
    }

    #[test]
    fn sorts_channels_by_number() {
        let mut hwmon = FakeHwmon::new();
        hwmon.chip(
            "nct6775",
            "",
            &[("fan10", None, "1"), ("fan9", None, "1"), ("fan1", None, "1")],
        );

        let fans = discover_sensors(hwmon.path(), HwmonSensorKind::Fan);

        assert_eq!(channels(&fans), vec!["nct6775/fan1", "nct6775/fan9", "nct6775/fan10"]);
    }

    #[test]
    fn a_missing_root_has_no_sensors() {
        assert!(discover_sensors(Path::new("/nonexistent/hwmon"), HwmonSensorKind::Fan).is_empty());
    }

    #[test]
    fn reads_temperatures_and_fan_speeds() {
        let mut hwmon = FakeHwmon::new();
        hwmon.chip("it87", "", &[("temp1", None, "45625"), ("fan1", None, "1350")]);

        let temperature = &discover_sensors(hwmon.path(), HwmonSensorKind::Temperature)[0];
        let fan = &discover_sensors(hwmon.path(), HwmonSensorKind::Fan)[0];

        assert_eq!(read_temperature(temperature), Some(45.625));
        assert_eq!(read_fan_speed(fan), Some(1350));
    }

    #[test]
    fn a_failed_read_has_no_value() {
        let sensor = HwmonSensor {
            id: id("it87", "", "fan1"),
            label: None,
            input_path: PathBuf::from("/nonexistent/fan1_input"),
        };
        assert_eq!(read_fan_speed(&sensor), None);
    }

    fn sensor(chip: &str, channel: &str, label: Option<&str>) -> HwmonSensor {
        HwmonSensor {
            id: id(chip, "", channel),
            label: label.map(str::to_string),
            input_path: PathBuf::new(),
        }
    }

    #[test]
    fn automatic_temperature_prefers_the_cpu_chip_and_label() {
        let sensors = vec![
            sensor("acpitz", "temp1", None),
            sensor("k10temp", "temp1", Some("Tctl")),
            sensor("k10temp", "temp2", Some("Tdie")),
            sensor("nvme", "temp1", Some("Composite")),
        ];
        assert_eq!(automatic_cpu_temperature(&sensors).unwrap().id.channel, "temp2");

        let sensors = vec![
            sensor("k10temp", "temp1", Some("Tctl")),
            sensor("k10temp", "temp3", Some("Tccd1")),
        ];
        assert_eq!(automatic_cpu_temperature(&sensors).unwrap().id.channel, "temp1");

        let sensors = vec![
            sensor("coretemp", "temp2", Some("Core 0")),
            sensor("coretemp", "temp1", Some("Package id 0")),
        ];
        assert_eq!(automatic_cpu_temperature(&sensors).unwrap().id.channel, "temp1");

        let sensors = vec![sensor("cpu_thermal", "temp1", None)];
        assert_eq!(automatic_cpu_temperature(&sensors).unwrap().id.chip, "cpu_thermal");
    }

    #[test]
    fn automatic_temperature_ignores_other_chips() {
        let sensors = vec![
            sensor("nvme", "temp1", Some("Composite")),
            sensor("amdgpu", "temp1", Some("edge")),
        ];
        assert!(automatic_cpu_temperature(&sensors).is_none());
    }

    #[test]
    fn automatic_fan_prefers_a_cpu_label() {
        let sensors = vec![
            sensor("nct6683", "fan1", Some("Pump Fan")),
            sensor("nct6683", "fan2", Some("CPU Fan")),
            sensor("nct6683", "fan3", Some("System Fan #1")),
        ];
        assert_eq!(automatic_cpu_fan(&sensors).unwrap().id.channel, "fan2");

        let sensors = vec![
            sensor("dell_smm", "fan1", Some("Processor Fan")),
            sensor("dell_smm", "fan2", None),
        ];
        assert_eq!(automatic_cpu_fan(&sensors).unwrap().id.channel, "fan1");
    }

    #[test]
    fn automatic_fan_uses_the_only_fan() {
        let sensors = vec![sensor("thinkpad", "fan1", None), sensor("amdgpu", "fan1", None)];
        assert_eq!(automatic_cpu_fan(&sensors).unwrap().id.chip, "thinkpad");
    }

    #[test]
    fn automatic_fan_isnt_a_graphics_card_or_a_guess() {
        assert!(automatic_cpu_fan(&[sensor("amdgpu", "fan1", None)]).is_none());
        assert!(automatic_cpu_fan(&[sensor("nct6775", "fan1", None), sensor("nct6775", "fan2", None)]).is_none());
        assert!(automatic_cpu_fan(&[]).is_none());
    }

    #[test]
    fn finds_a_configured_sensor_or_none() {
        let sensors = vec![sensor("nct6775", "fan1", None), sensor("nct6775", "fan2", None)];

        let configured = id("nct6775", "", "fan2");
        assert_eq!(
            find_sensor(&sensors, HwmonSensorKind::Fan, Some(&configured))
                .unwrap()
                .id,
            configured
        );

        let missing = id("nct6775", "", "fan7");
        assert!(find_sensor(&sensors, HwmonSensorKind::Fan, Some(&missing)).is_none());
    }

    #[test]
    fn sensor_ids_round_trip_through_the_form_value() {
        let sensor_id = id("k10temp", "0000:00:18.3", "temp1");

        let value = HwmonSensorId::to_form_value(Some(&sensor_id));
        assert_eq!(HwmonSensorId::from_form_value(&value), Some(sensor_id));

        assert_eq!(HwmonSensorId::to_form_value(None), "");
        assert_eq!(HwmonSensorId::from_form_value(""), None);
    }

    #[test]
    fn sensor_ids_without_a_device_round_trip() {
        let sensor_id = id("thinkpad", "", "fan1");
        let value = HwmonSensorId::to_form_value(Some(&sensor_id));
        assert_eq!(HwmonSensorId::from_form_value(&value), Some(sensor_id));
    }
}
