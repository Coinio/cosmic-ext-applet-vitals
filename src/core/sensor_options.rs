use crate::fl;
use crate::sensors::hwmon_reader::{
    discover_sensors, find_sensor, read_fan_speed, read_temperature, HwmonSensor, HwmonSensorId, HwmonSensorKind,
    HWMON_ROOT,
};
use crate::ui::indicators::cpu::{format_fan_speed, format_temperature};
use std::path::Path;
use std::sync::Arc;

/// The sensors that can be chosen for a CPU temperature or fan setting, for a dropdown. The first is always
/// automatic, which is the empty form value.
#[derive(Debug, Clone, Default)]
pub struct SensorOptions {
    pub labels: Vec<String>,
    /// The form values, parallel to `labels`.
    pub values: Arc<[String]>,
    /// Whether any sensor of this kind was found, so the settings can explain why there are none.
    pub has_sensors: bool,
}

impl SensorOptions {
    /// Each sensor is shown with its current reading, so they can be told apart by watching them. The configured
    /// sensor is included even if it's missing, so it still shows as selected.
    pub fn new(kind: HwmonSensorKind, configured: Option<&HwmonSensorId>, use_fahrenheit: bool) -> Self {
        let sensors = discover_sensors(Path::new(HWMON_ROOT), kind);
        Self::from_sensors(&sensors, kind, configured, |sensor| match kind {
            HwmonSensorKind::Temperature => {
                read_temperature(sensor).map(|celsius| format_temperature(celsius, use_fahrenheit))
            }
            HwmonSensorKind::Fan => read_fan_speed(sensor).map(format_fan_speed),
        })
    }

    fn from_sensors(
        sensors: &[HwmonSensor],
        kind: HwmonSensorKind,
        configured: Option<&HwmonSensorId>,
        reading: impl Fn(&HwmonSensor) -> Option<String>,
    ) -> Self {
        let automatic = match find_sensor(sensors, kind, None) {
            Some(sensor) => fl!("settings-cpu-sensor-automatic", sensor = sensor.name()),
            None => fl!("settings-cpu-sensor-automatic-none"),
        };

        let mut labels = vec![automatic];
        let mut values = vec![HwmonSensorId::to_form_value(None)];

        for sensor in sensors {
            labels.push(match reading(sensor) {
                Some(reading) => format!("{} ({reading})", sensor.name()),
                None => fl!("settings-cpu-sensor-no-reading", sensor = sensor.name()),
            });
            values.push(HwmonSensorId::to_form_value(Some(&sensor.id)));
        }

        if let Some(configured) = configured {
            if !sensors.iter().any(|sensor| &sensor.id == configured) {
                let name = format!("{}: {}", configured.chip, configured.channel);
                labels.push(fl!("settings-cpu-sensor-missing", sensor = name));
                values.push(HwmonSensorId::to_form_value(Some(configured)));
            }
        }

        SensorOptions {
            labels,
            values: values.into(),
            has_sensors: !sensors.is_empty(),
        }
    }
    #[cfg(test)]
    pub fn position(&self, form_value: &str) -> Option<usize> {
        self.values.iter().position(|value| value == form_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sensor(chip: &str, channel: &str, label: Option<&str>) -> HwmonSensor {
        HwmonSensor {
            id: HwmonSensorId {
                chip: chip.to_string(),
                device: String::new(),
                channel: channel.to_string(),
            },
            label: label.map(str::to_string),
            input_path: PathBuf::new(),
        }
    }

    #[test]
    fn starts_with_automatic_then_each_sensor() {
        let sensors = vec![
            sensor("k10temp", "temp1", Some("Tctl")),
            sensor("nvme", "temp1", Some("Composite")),
        ];

        let options = SensorOptions::from_sensors(&sensors, HwmonSensorKind::Temperature, None, |sensor| {
            (sensor.id.chip == "k10temp").then(|| "45°C".to_string())
        });

        assert_eq!(options.labels.len(), options.values.len());
        assert_eq!(options.values.len(), 3);
        assert!(options.labels[0].contains("k10temp: Tctl"));
        assert_eq!(options.position(""), Some(0));
        assert_eq!(options.labels[1], "k10temp: Tctl (45°C)");
        assert!(options.labels[2].contains("nvme: Composite"));
        assert_eq!(
            options.position(&HwmonSensorId::to_form_value(Some(&sensors[1].id))),
            Some(2)
        );
        assert!(options.has_sensors);
    }

    #[test]
    fn includes_a_configured_sensor_that_is_missing() {
        let configured = sensor("nct6775", "fan2", None).id;

        let options = SensorOptions::from_sensors(&[], HwmonSensorKind::Fan, Some(&configured), |_| None);

        let index = options
            .position(&HwmonSensorId::to_form_value(Some(&configured)))
            .expect("Configured sensor missing from options");
        assert!(options.labels[index].contains("nct6775: fan2"));
        assert!(!options.has_sensors);
    }
}
