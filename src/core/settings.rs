use std::time::Duration;
use cosmic::iced::window;
use indexmap::IndexMap;
use crate::configuration::app_configuration::{CPU_SETTINGS_WINDOW_ID, DISK_SETTINGS_WINDOW_ID, GENERAL_SETTINGS_WINDOW_ID, HIDE_INDICATOR_SETTING_KEY, HIDE_ICON_SETTING_KEY, ICON_COLOUR_SETTING_KEY, DOWNLOAD_COLOUR_SETTING_KEY, UPLOAD_COLOUR_SETTING_KEY, READ_COLOUR_SETTING_KEY, WRITE_COLOUR_SETTING_KEY, MAX_SAMPLES_SETTING_KEY, MEMORY_SETTINGS_WINDOW_ID, NETWORK_SETTINGS_WINDOW_ID, UPDATE_INTERVAL_SETTING_KEY, FIX_INDICATOR_SIZE_SETTING_KEY, FONT_SETTING_KEY, FONT_SIZE_SETTING_KEY, STORAGE_MOUNTS_SETTING_KEY, STORAGE_SETTINGS_WINDOW_ID, CPU_COMBINE_CORES_SETTING_KEY, CPU_TEMPERATURE_SENSOR_SETTING_KEY, CPU_FAN_SENSOR_SETTING_KEY, CPU_USE_FAHRENHEIT_SETTING_KEY, GPU_DEVICE_SETTING_KEY, GPU_SETTINGS_WINDOW_ID, GPU_SHOW_VRAM_SETTING_KEY};
use crate::configuration::cpu::CpuConfiguration;
use crate::configuration::disk::DiskConfiguration;
use crate::configuration::gpu::{device_to_form_value, GpuConfiguration};
use crate::configuration::general::{
    FontSizeSlot, GeneralConfiguration, INDICATOR_FONT_SIZE_MAXIMUM, INDICATOR_FONT_SIZE_MINIMUM,
};
use crate::configuration::memory::MemoryConfiguration;
use crate::configuration::network::NetworkConfiguration;
use crate::configuration::storage::{mounts_to_form_value, StorageConfiguration};
use crate::configuration::validation::ConfigurationValidation;
use crate::fl;
use crate::sensors::hwmon_reader::{HwmonSensorId, HwmonSensorKind};

#[derive(Debug, Clone)]
pub enum SettingsFormEvent {
    StringFieldUpdated(SettingsFormEventValue),
    CheckBoxUpdated(SettingsFormEventValue),
}

#[derive(Debug, Clone)]
pub enum SettingsFormInputType {
    String,
    CheckBox,
    ColourPicker,
    /// The value is an encoded `IndicatorFont`.
    FontPicker,
    /// The value is an integer within the inclusive bounds.
    SpinButton { min: u16, max: u16 },
    /// A checkbox per mount point. The value is the encoded list of included mount points, see
    /// [`mounts_to_form_value`].
    MountList,
    /// A dropdown of the sensors of the kind. The value is an encoded optional `HwmonSensorId`, see
    /// [`HwmonSensorId::to_form_value`], where empty picks the sensor automatically.
    SensorPicker(HwmonSensorKind),
    /// A dropdown of the GPUs. The value is a GPU's PCI address, where empty picks the GPU automatically.
    GpuPicker,
}

#[derive(Debug, Clone)]
pub struct SettingsFormEventValue {
    pub settings_window_id: window::Id,
    pub form_value_key: &'static str,
    pub value: String,
}

pub struct SettingsFormItem {
    pub label: String,
    pub value: String,
    pub input_type: SettingsFormInputType,
    pub validator: Option<fn(&str) -> Result<(), String>>,
    pub helper_text: Option<String>,
}

pub struct SettingsForm {
    pub settings_window_id: window::Id,
    pub title: String,
    pub values: IndexMap<&'static str, SettingsFormItem>,
}

impl From<&CpuConfiguration> for SettingsForm {
    fn from(config: &CpuConfiguration) -> SettingsForm {
        let mut values = build_shared_settings(
            config.hide_indicator,
            Some((config.hide_icon, config.icon_colour.clone())),
            Vec::new(),
            config.update_interval,
            Some(config.max_samples),
        );
        values.insert(
            CPU_COMBINE_CORES_SETTING_KEY,
            checkbox_setting(
                fl!("settings-cpu-combine-cores"),
                config.combine_cores,
                Some(fl!("settings-cpu-combine-cores-helper")),
            ),
        );
        values.insert(
            CPU_TEMPERATURE_SENSOR_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-cpu-temperature-sensor"),
                value: HwmonSensorId::to_form_value(config.temperature_sensor.as_ref()),
                input_type: SettingsFormInputType::SensorPicker(HwmonSensorKind::Temperature),
                validator: None,
                helper_text: Some(fl!("settings-cpu-temperature-sensor-helper")),
            },
        );
        values.insert(
            CPU_USE_FAHRENHEIT_SETTING_KEY,
            checkbox_setting(fl!("settings-cpu-use-fahrenheit"), config.use_fahrenheit, None),
        );
        values.insert(
            CPU_FAN_SENSOR_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-cpu-fan-sensor"),
                value: HwmonSensorId::to_form_value(config.fan_sensor.as_ref()),
                input_type: SettingsFormInputType::SensorPicker(HwmonSensorKind::Fan),
                validator: None,
                helper_text: Some(fl!("settings-cpu-fan-sensor-helper")),
            },
        );

        SettingsForm {
            settings_window_id: CPU_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-cpu-title"),
            values,
        }
    }
}

impl From<&MemoryConfiguration> for SettingsForm {
    fn from(config: &MemoryConfiguration) -> SettingsForm {
        let values = build_shared_settings(
            config.hide_indicator,
            Some((config.hide_icon, config.icon_colour.clone())),
            Vec::new(),
            config.update_interval,
            Some(config.max_samples),
        );
        SettingsForm {
            settings_window_id: MEMORY_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-memory-title"),
            values,
        }
    }
}

impl From<&GpuConfiguration> for SettingsForm {
    fn from(config: &GpuConfiguration) -> SettingsForm {
        let mut values = build_shared_settings(
            config.hide_indicator,
            Some((config.hide_icon, config.icon_colour.clone())),
            Vec::new(),
            config.update_interval,
            Some(config.max_samples),
        );
        values.insert(
            GPU_DEVICE_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-gpu-device"),
                value: device_to_form_value(config.device.as_deref()),
                input_type: SettingsFormInputType::GpuPicker,
                validator: None,
                helper_text: Some(fl!("settings-gpu-device-helper")),
            },
        );
        values.insert(
            GPU_SHOW_VRAM_SETTING_KEY,
            checkbox_setting(fl!("settings-gpu-show-vram"), config.show_vram, None),
        );

        SettingsForm {
            settings_window_id: GPU_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-gpu-title"),
            values,
        }
    }
}

impl From<&NetworkConfiguration> for SettingsForm {
    fn from(config: &NetworkConfiguration) -> SettingsForm {
        let values = build_shared_settings(
            config.hide_indicator,
            None,
            vec![
                colour_setting(DOWNLOAD_COLOUR_SETTING_KEY, fl!("settings-download-colour"), &config.download_colour),
                colour_setting(UPLOAD_COLOUR_SETTING_KEY, fl!("settings-upload-colour"), &config.upload_colour),
            ],
            config.update_interval,
            Some(config.max_samples),
        );

        SettingsForm {
            settings_window_id: NETWORK_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-network-title"),
            values,
        }
    }
}

impl From<&DiskConfiguration> for SettingsForm {
    fn from(config: &DiskConfiguration) -> SettingsForm {
        let values = build_shared_settings(
            config.hide_indicator,
            None,
            vec![
                colour_setting(READ_COLOUR_SETTING_KEY, fl!("settings-read-colour"), &config.read_colour),
                colour_setting(WRITE_COLOUR_SETTING_KEY, fl!("settings-write-colour"), &config.write_colour),
            ],
            config.update_interval,
            Some(config.max_samples),
        );
        SettingsForm {
            settings_window_id: DISK_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-disk-title"),
            values,
        }
    }
}

impl From<&StorageConfiguration> for SettingsForm {
    fn from(config: &StorageConfiguration) -> SettingsForm {
        let mut values = build_shared_settings(
            config.hide_indicator,
            Some((config.hide_icon, config.icon_colour.clone())),
            Vec::new(),
            config.update_interval,
            None,
        );
        values.insert(
            STORAGE_MOUNTS_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-storage-mounts"),
                value: mounts_to_form_value(&config.mounts),
                input_type: SettingsFormInputType::MountList,
                validator: None,
                helper_text: Some(fl!("settings-storage-mounts-helper")),
            },
        );

        SettingsForm {
            settings_window_id: STORAGE_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-storage-title"),
            values,
        }
    }
}

impl SettingsForm {
    /// The font size field is only shown when the applet is in a named panel/dock size, and edits that size only.
    pub fn from_general(config: &GeneralConfiguration, font_size_slot: Option<FontSizeSlot>) -> SettingsForm {
        let mut values: IndexMap<&'static str, SettingsFormItem> = IndexMap::new();
        values.insert(
            FONT_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-font"),
                value: config.font.to_form_value(),
                input_type: SettingsFormInputType::FontPicker,
                validator: None,
                helper_text: Some(fl!("settings-font-helper")),
            },
        );
        if let Some(slot) = font_size_slot {
            values.insert(
                FONT_SIZE_SETTING_KEY,
                SettingsFormItem {
                    label: fl!("settings-font-size"),
                    value: config.font_size(slot).to_string(),
                    input_type: SettingsFormInputType::SpinButton {
                        min: INDICATOR_FONT_SIZE_MINIMUM,
                        max: INDICATOR_FONT_SIZE_MAXIMUM,
                    },
                    validator: None,
                    helper_text: Some(fl!("settings-font-size-helper", size = slot.size.label())),
                },
            );
        }
        values.insert(
            FIX_INDICATOR_SIZE_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-fix-indicator-size"),
                value: config.fix_indicator_size.to_string(),
                input_type: SettingsFormInputType::CheckBox,
                validator: Some(ConfigurationValidation::is_valid_boolean),
                helper_text: Some(fl!("settings-fix-indicator-size-helper")),
            },
        );
        values.insert(
            crate::configuration::app_configuration::USE_IEC_UNITS_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-use-iec-units"),
                value: config.use_iec_units.to_string(),
                input_type: SettingsFormInputType::CheckBox,
                validator: Some(ConfigurationValidation::is_valid_boolean),
                helper_text: Some(fl!("settings-use-iec-units-helper")),
            },
        );
        values.insert(
            crate::configuration::app_configuration::SHOW_INDICATOR_SEPARATORS_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-show-indicator-separators"),
                value: config.show_indicator_separators.to_string(),
                input_type: SettingsFormInputType::CheckBox,
                validator: Some(ConfigurationValidation::is_valid_boolean),
                helper_text: None,
            },
        );

        SettingsForm {
            settings_window_id: GENERAL_SETTINGS_WINDOW_ID.clone(),
            title: fl!("settings-general-title"),
            values,
        }
    }
}

// This is used to build the shared settings between all the settings forms.
/// Extend the map returned by this with any sensor-specific settings if required.
/// `icon` is the indicator's (hide icon, icon colour) settings, for indicators that have an icon.
/// `colours` are any further colour settings, shown after the icon settings.
fn build_shared_settings(
    hide_indicator: bool,
    icon: Option<(bool, Option<String>)>,
    colours: Vec<(&'static str, SettingsFormItem)>,
    update_interval: Duration,
    max_samples: Option<usize>,
) -> IndexMap<&'static str, SettingsFormItem> {
    let mut settings = IndexMap::from([(
        HIDE_INDICATOR_SETTING_KEY,
        SettingsFormItem {
            label: fl!("settings-hide-indicator"),
            value: hide_indicator.to_string(),
            input_type: SettingsFormInputType::CheckBox,
            validator: Some(ConfigurationValidation::is_valid_boolean),
            helper_text: None,
        },
    )]);

    if let Some((hide_icon, icon_colour)) = icon {
        settings.insert(
            HIDE_ICON_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-hide-icon"),
                value: hide_icon.to_string(),
                input_type: SettingsFormInputType::CheckBox,
                validator: Some(ConfigurationValidation::is_valid_boolean),
                helper_text: None,
            },
        );
        settings.insert(
            ICON_COLOUR_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-icon-colour"),
                value: icon_colour.unwrap_or_default(),
                input_type: SettingsFormInputType::ColourPicker,
                validator: None,
                helper_text: None,
            },
        );
    }

    settings.extend(colours);

    settings.insert(
        UPDATE_INTERVAL_SETTING_KEY,
        SettingsFormItem {
            label: fl!("settings-update-interval"),
            value: update_interval.as_millis().to_string(),
            input_type: SettingsFormInputType::String,
            validator: Some(ConfigurationValidation::is_valid_interval),
            helper_text: None,
        },
    );

    if let Some(max_samples) = max_samples {
        settings.insert(
            MAX_SAMPLES_SETTING_KEY,
            SettingsFormItem {
                label: fl!("settings-max-samples"),
                value: max_samples.to_string(),
                input_type: SettingsFormInputType::String,
                validator: Some(ConfigurationValidation::is_valid_max_samples),
                helper_text: None,
            },
        );
    }

    settings
}

fn checkbox_setting(label: String, value: bool, helper_text: Option<String>) -> SettingsFormItem {
    SettingsFormItem {
        label,
        value: value.to_string(),
        input_type: SettingsFormInputType::CheckBox,
        validator: Some(ConfigurationValidation::is_valid_boolean),
        helper_text,
    }
}

fn colour_setting(key: &'static str, label: String, colour: &Option<String>) -> (&'static str, SettingsFormItem) {
    (
        key,
        SettingsFormItem {
            label,
            value: colour.clone().unwrap_or_default(),
            input_type: SettingsFormInputType::ColourPicker,
            validator: None,
            helper_text: None,
        },
    )
}
