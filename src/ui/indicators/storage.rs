use crate::app::Message;
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_colours::AppColour;
use crate::core::app_icons::{AppIcons, STORAGE_ICON};
use crate::fl;
use crate::monitors::storage_monitor::StorageStats;
use crate::sensors::storage_reader::{MountUsage, UnmountedVolume};
use crate::ui::components::detail_popup::{
    card, card_with_heading, detail_popup, heading_row, labelled_bar, popup_header, used_fraction, PopupStyle,
};
use cosmic::iced::widget::Column;
use crate::ui::components::indicator::{IndicatorContent, IndicatorIcon, IndicatorLine};
use crate::ui::indicators::format_size_bytes;
use cosmic::Element;

impl StorageStats {
    pub fn content(&self, configuration: &AppConfiguration, horizontal: bool) -> Option<IndicatorContent> {
        let storage = &configuration.storage;
        if storage.hide_indicator {
            return None;
        }

        let available = format_size_bytes(self.available_bytes, configuration);
        let total = format_size_bytes(self.total_bytes, configuration);

        // There's room for "available/total" on a single line in a horizontal panel, but not in a vertical one.
        let lines = if horizontal {
            vec![IndicatorLine::text(format!("{available}/{total}"))]
        } else {
            vec![IndicatorLine::text(available), IndicatorLine::text(total)]
        };

        let max_label_text = match (configuration.general.use_iec_units, horizontal) {
            (true, true) => "99.9TiB/99.9TiB",
            (true, false) => "99.9TiB",
            (false, true) => "99.9TB/99.9TB",
            (false, false) => "99.9TB",
        };

        Some(IndicatorContent {
            icon: (!storage.hide_icon).then(|| IndicatorIcon::new(STORAGE_ICON, storage.icon_colour.as_deref())),
            lines,
            max_label_text,
        })
    }

    /// The popup shown when hovering over the indicator, which lists every mount, not only the ones in the totals,
    /// followed by the volumes that aren't mounted.
    pub fn detail<'a>(
        &'a self,
        configuration: &AppConfiguration,
        app_icons: &'a AppIcons,
        style: PopupStyle,
    ) -> Element<'a, Message> {
        let colour = configuration.storage.icon_colour.as_deref().and_then(AppColour::parse);
        let mut cards = Vec::new();

        if !self.mounts.is_empty() {
            let mounts = self
                .mounts
                .iter()
                .map(|mount| mount_detail(style, mount, colour, configuration));
            cards.push(card(Column::with_children(mounts).spacing(12)));
        }

        if !self.unmounted.is_empty() {
            let unmounted = self
                .unmounted
                .iter()
                .map(|volume| unmounted_detail(style, volume, configuration));
            cards.push(card_with_heading(style, None, unmounted));
        }

        let summary = fl!(
            "popup-storage-space",
            available = format_size_bytes(self.available_bytes, configuration),
            total = format_size_bytes(self.total_bytes, configuration)
        );
        let header = popup_header(
            style,
            app_icons.get(STORAGE_ICON),
            colour,
            fl!("popup-storage-title"),
            Some(style.summary(summary, colour).into()),
        );

        detail_popup(header, cards)
    }
}

/// A mount point and its free space, above a bar showing how much of it is used. Blocks reserved for root count as
/// used, matching the free space shown.
fn mount_detail<'a>(
    style: PopupStyle,
    mount: &'a MountUsage,
    bar_colour: Option<AppColour>,
    configuration: &AppConfiguration,
) -> Element<'a, Message> {
    let space = fl!(
        "popup-storage-space",
        available = format_size_bytes(mount.available_bytes, configuration),
        total = format_size_bytes(mount.total_bytes, configuration)
    );
    let used_bytes = mount.total_bytes.saturating_sub(mount.available_bytes);

    labelled_bar(
        style,
        mount.mount_point.as_str(),
        space,
        used_fraction(used_bytes, mount.total_bytes),
        bar_colour,
    )
}

/// A volume that isn't mounted, by its label and device, with its size and why there's no free space shown.
fn unmounted_detail<'a>(
    style: PopupStyle,
    volume: &'a UnmountedVolume,
    configuration: &AppConfiguration,
) -> Element<'a, Message> {
    let name = match &volume.label {
        Some(label) => format!("{label} ({})", volume.name),
        None => volume.name.clone(),
    };
    let size = format_size_bytes(volume.size_bytes, configuration);
    let caption = if volume.is_locked {
        fl!("popup-storage-locked", size = size)
    } else {
        fl!("popup-storage-unmounted", size = size)
    };

    heading_row(style, name, Some(caption))
}
