use crate::app::Message;
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_colours::AppColour;
use crate::core::app_icons::{AppIcons, STORAGE_ICON};
use crate::fl;
use crate::monitors::storage_monitor::{DiskGroup, StorageStats};
use crate::sensors::storage_reader::{MountUsage, StorageDisk, UnmountedVolume};
use crate::ui::components::detail_popup::{
    card, detail_popup, heading_row, labelled_bar, popup_header, used_fraction, PopupStyle, ROW_SPACING,
};
use cosmic::iced::widget::{Column, Row};
use cosmic::iced::{Alignment, Length};
use crate::ui::components::indicator::{IndicatorContent, IndicatorIcon, IndicatorLine};
use crate::ui::indicators::format_size_bytes_precise;
use cosmic::Element;

impl StorageStats {
    pub fn content(&self, configuration: &AppConfiguration, horizontal: bool) -> Option<IndicatorContent> {
        let storage = &configuration.storage;
        if storage.hide_indicator {
            return None;
        }

        // The same precision as the detail popup, which fits in the same width as the widest label below.
        let used = format_size_bytes_precise(self.used_bytes(), configuration);
        let total = format_size_bytes_precise(self.total_bytes, configuration);

        // There's room for "used/total" on a single line in a horizontal panel, but not in a vertical one.
        let lines = if horizontal {
            vec![IndicatorLine::text(format!("{used}/{total}"))]
        } else {
            vec![IndicatorLine::text(used), IndicatorLine::text(total)]
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

    /// The popup shown when hovering over the indicator, which lists the mounts in the totals, and the volumes that
    /// aren't mounted, in a card for each disk.
    pub fn detail<'a>(
        &'a self,
        configuration: &AppConfiguration,
        app_icons: &'a AppIcons,
        style: PopupStyle,
    ) -> Element<'a, Message> {
        let colour = configuration.storage.icon_colour.as_deref().and_then(AppColour::parse);
        let groups = self.by_disk();
        // The mounts that aren't on a single disk only need a heading to set them apart from the disks.
        let has_disks = groups.iter().any(|group| group.disk.is_some());
        let cards: Vec<Element<'a, Message>> = groups
            .into_iter()
            .map(|group| disk_card(style, group, has_disks, colour, configuration))
            .collect();

        let summary = fl!(
            "popup-storage-space",
            used = format_size_bytes_precise(self.used_bytes(), configuration),
            total = format_size_bytes_precise(self.total_bytes, configuration)
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

/// A disk's model, name and size, over its mounts, then its volumes that aren't mounted.
fn disk_card<'a>(
    style: PopupStyle,
    group: DiskGroup<'a>,
    has_disks: bool,
    bar_colour: Option<AppColour>,
    configuration: &AppConfiguration,
) -> Element<'a, Message> {
    let heading = match group.disk {
        Some(disk) => Some(disk_heading(style, disk, configuration)),
        None => has_disks.then(|| style.section_heading(fl!("popup-storage-other")).into()),
    };

    // Left out when empty, so they don't add spacing.
    let mounts = (!group.mounts.is_empty()).then(|| {
        let mounts = group
            .mounts
            .into_iter()
            .map(|mount| mount_detail(style, mount, bar_colour, configuration));
        Column::with_children(mounts).spacing(12)
    });
    let unmounted = (!group.unmounted.is_empty()).then(|| {
        let unmounted = group
            .unmounted
            .into_iter()
            .map(|volume| unmounted_detail(style, volume, configuration));
        Column::with_children(unmounted).spacing(ROW_SPACING)
    });

    card(
        Column::new()
            .spacing(10)
            .push_maybe(heading)
            .push_maybe(mounts)
            .push_maybe(unmounted),
    )
}

/// The disk's model, or its name when it has no model, with its name and size on the right.
fn disk_heading<'a>(style: PopupStyle, disk: &'a StorageDisk, configuration: &AppConfiguration) -> Element<'a, Message> {
    let size = format_size_bytes_precise(disk.size_bytes, configuration);
    let (title, caption) = match &disk.model {
        Some(model) => (model.as_str(), format!("{} · {size}", disk.name)),
        None => (disk.name.as_str(), size),
    };

    Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(style.section_heading(title).width(Length::Fill))
        .push(style.caption(caption))
        .into()
}

/// A mount point and its used space, above a bar showing the same.
fn mount_detail<'a>(
    style: PopupStyle,
    mount: &'a MountUsage,
    bar_colour: Option<AppColour>,
    configuration: &AppConfiguration,
) -> Element<'a, Message> {
    let used_bytes = mount.used_bytes();
    let space = fl!(
        "popup-storage-space",
        used = format_size_bytes_precise(used_bytes, configuration),
        total = format_size_bytes_precise(mount.total_bytes, configuration)
    );

    labelled_bar(
        style,
        mount.mount_point.as_str(),
        space,
        used_fraction(used_bytes, mount.total_bytes),
        bar_colour,
    )
}

/// A volume that isn't mounted, by its label and device, with its size and why there's no used space shown.
fn unmounted_detail<'a>(
    style: PopupStyle,
    volume: &'a UnmountedVolume,
    configuration: &AppConfiguration,
) -> Element<'a, Message> {
    let name = match &volume.label {
        Some(label) => format!("{label} ({})", volume.name),
        None => volume.name.clone(),
    };
    let size = format_size_bytes_precise(volume.size_bytes, configuration);
    let caption = if volume.is_locked {
        fl!("popup-storage-locked", size = size)
    } else {
        fl!("popup-storage-unmounted", size = size)
    };

    heading_row(style, name, Some(caption))
}
