use crate::configuration::storage::{mounts_from_form_value, toggle_mount_in_form_value};
use crate::core::mount_options::MountOptions;
use crate::core::settings::{SettingsFormEvent, SettingsFormEventValue};
use crate::settings_app::Message;
use cosmic::iced::{window, Color, Length};
use cosmic::widget::{self, settings};
use cosmic::Element;

pub struct FormMountListInputProps<'a> {
    pub settings_window_id: window::Id,
    pub label: String,
    pub value: &'a str,
    pub form_value_key: &'static str,
    pub options: &'a MountOptions,
    pub helper_text: Option<String>,
    pub helper_text_color: Color,
}

/// A heading, followed by a row with a checkbox for each mount point. Ticking a mount point includes it in the
/// storage totals. Each row is its own element, so the settings list draws dividers between them.
pub fn form_mount_list_input<'a>(props: FormMountListInputProps<'a>) -> Vec<Element<'a, Message>> {
    let FormMountListInputProps {
        settings_window_id,
        label,
        value,
        form_value_key,
        options,
        helper_text,
        helper_text_color,
    } = props;

    let included = mounts_from_form_value(value);

    let mut heading = widget::Column::new().spacing(6).push(widget::text::heading(label));
    if let Some(helper) = helper_text {
        let helper_text_widget = widget::text(helper)
            .size(12)
            .class(cosmic::theme::Text::from(helper_text_color));
        heading = heading.push(widget::container(helper_text_widget).width(Length::Fill));
    }

    let mut rows: Vec<Element<'a, Message>> = vec![heading.into()];

    for option in &options.options {
        let mount_point = option.mount_point.clone();
        // The value as it is when drawn, which is current, as the form redraws after every change.
        let current_value = value.to_string();

        let checkbox = widget::checkbox(included.contains(&option.mount_point)).on_toggle(move |is_included| {
            Message::SettingsFormUpdate(SettingsFormEvent::CheckBoxUpdated(SettingsFormEventValue {
                settings_window_id,
                form_value_key,
                value: toggle_mount_in_form_value(&current_value, &mount_point, is_included),
            }))
        });

        rows.push(
            settings::item::builder(option.mount_point.as_str())
                .description(option.description.as_str())
                .control(checkbox)
                .into(),
        );
    }

    rows
}
