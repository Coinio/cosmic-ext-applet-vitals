use crate::configuration::app_configuration::{
    HIDE_ICON_SETTING_KEY, HIDE_INDICATOR_SETTING_KEY, ICON_COLOUR_SETTING_KEY, STORAGE_MOUNTS_SETTING_KEY,
    STORAGE_SETTINGS_WINDOW_ID, UPDATE_INTERVAL_SETTING_KEY,
};
use crate::configuration::validation::ConfigurationValidation;
use crate::core::app_colours::ACCENT_ORANGE;
use crate::core::settings::SettingsForm;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// The mount point included in the totals by default.
pub const DEFAULT_STORAGE_MOUNT: &str = "/";

/// Separates the mount points in the settings form value. Paths can't contain a NUL, so it can't clash.
const MOUNTS_FORM_VALUE_SEPARATOR: char = '\0';

/// The configuration for the storage monitor
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
// Fields added in later versions are missing from older saved configurations, so they take their default.
#[serde(default)]
pub struct StorageConfiguration {
    /// Whether to hide the storage indicator from the panel
    pub hide_indicator: bool,
    /// Whether to hide only the icon for this indicator
    pub hide_icon: bool,
    /// The duration between each update interval, i.e. 5 seconds
    pub update_interval: Duration,
    /// The indicator icon colour key
    pub icon_colour: Option<String>,
    /// The mount points whose space is included in the available and total space, sorted.
    pub mounts: Vec<String>,
}

impl Default for StorageConfiguration {
    fn default() -> Self {
        StorageConfiguration {
            hide_indicator: false,
            hide_icon: false,
            update_interval: Duration::from_secs(5),
            icon_colour: Some(ACCENT_ORANGE.to_string()),
            mounts: vec![DEFAULT_STORAGE_MOUNT.to_string()],
        }
    }
}

impl StorageConfiguration {
    pub fn update(&self, settings_form: &SettingsForm) -> Self {
        if settings_form.settings_window_id != STORAGE_SETTINGS_WINDOW_ID.clone() {
            panic!("Attempted to update storage settings from a non-storage settings window.")
        }

        StorageConfiguration {
            hide_indicator: ConfigurationValidation::sanitise_boolean_input(
                form_value(settings_form, HIDE_INDICATOR_SETTING_KEY),
                self.hide_indicator,
            ),
            hide_icon: ConfigurationValidation::sanitise_boolean_input(
                form_value(settings_form, HIDE_ICON_SETTING_KEY),
                self.hide_icon,
            ),
            update_interval: ConfigurationValidation::sanitise_interval_input(
                form_value(settings_form, UPDATE_INTERVAL_SETTING_KEY),
                self.update_interval,
            ),
            icon_colour: Some(form_value(settings_form, ICON_COLOUR_SETTING_KEY)),
            mounts: mounts_from_form_value(&form_value(settings_form, STORAGE_MOUNTS_SETTING_KEY)),
        }
    }
}

/// Encode the mount points as a single settings form value.
pub fn mounts_to_form_value(mounts: &[String]) -> String {
    let mut sorted = mounts.to_vec();
    sorted.sort();
    sorted.dedup();
    sorted.join(&MOUNTS_FORM_VALUE_SEPARATOR.to_string())
}

/// Decode the mount points from a settings form value, sorted and without duplicates.
pub fn mounts_from_form_value(value: &str) -> Vec<String> {
    let mut mounts: Vec<String> = value
        .split(MOUNTS_FORM_VALUE_SEPARATOR)
        .filter(|mount| !mount.is_empty())
        .map(str::to_string)
        .collect();
    mounts.sort();
    mounts.dedup();
    mounts
}

/// The settings form value with `mount` included or excluded.
pub fn toggle_mount_in_form_value(value: &str, mount: &str, included: bool) -> String {
    let mut mounts = mounts_from_form_value(value);
    mounts.retain(|existing| existing != mount);
    if included {
        mounts.push(mount.to_string());
    }
    mounts_to_form_value(&mounts)
}

fn form_value(settings_form: &SettingsForm, key: &str) -> String {
    settings_form
        .values
        .get(key)
        .unwrap_or_else(|| panic!("{key} missing from settings form options"))
        .value
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_the_root_mount() {
        assert_eq!(StorageConfiguration::default().mounts, vec!["/".to_string()]);
    }

    #[test]
    fn loads_a_partial_configuration() {
        let config: StorageConfiguration =
            ron::from_str("(hide_indicator: true)").expect("Partial configuration should load");

        assert!(config.hide_indicator);
        assert_eq!(config.mounts, vec!["/".to_string()]);
    }

    #[test]
    fn mounts_round_trip_through_the_form_value() {
        let mounts = vec!["/".to_string(), "/home".to_string(), "/media/user/My Drive".to_string()];

        assert_eq!(mounts_from_form_value(&mounts_to_form_value(&mounts)), mounts);
        assert_eq!(mounts_from_form_value(""), Vec::<String>::new());
    }

    #[test]
    fn toggling_a_mount_includes_or_excludes_it() {
        let value = mounts_to_form_value(&["/".to_string()]);

        let included = toggle_mount_in_form_value(&value, "/data", true);
        assert_eq!(mounts_from_form_value(&included), vec!["/".to_string(), "/data".to_string()]);

        let included_twice = toggle_mount_in_form_value(&included, "/data", true);
        assert_eq!(included_twice, included);

        let excluded = toggle_mount_in_form_value(&included, "/", false);
        assert_eq!(mounts_from_form_value(&excluded), vec!["/data".to_string()]);
    }

    #[test]
    fn update_reads_the_mounts_from_the_form() {
        let configuration = StorageConfiguration::default();
        let mut form = SettingsForm::from(&configuration);

        let item = form.values.get_mut(STORAGE_MOUNTS_SETTING_KEY).expect("Mounts missing from form");
        item.value = toggle_mount_in_form_value(&item.value, "/home", true);

        let updated = configuration.update(&form);

        assert_eq!(
            updated,
            StorageConfiguration {
                mounts: vec!["/".to_string(), "/home".to_string()],
                ..configuration
            }
        );
    }
}
