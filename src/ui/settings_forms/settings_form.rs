use crate::settings_app::{Message, SettingsApp};
use crate::core::app_colours::{AppColour, BRIGHT_RED, EXT_WARM_GREY};
use crate::core::settings::{SettingsForm, SettingsFormInputType};
use crate::ui::components::form_checkbox_input::{form_checkbox_input, FormCheckboxInputProps};
use crate::ui::components::form_font_dropdown_input::{form_font_dropdown_input, FormFontDropdownInputProps};
use crate::ui::components::form_mount_list_input::{form_mount_list_input, FormMountListInputProps};
use crate::ui::components::form_spin_button_input::{form_spin_button_input, FormSpinButtonInputProps};
use crate::ui::components::form_text_input::{form_text_input, FormTextInputProps};
use crate::ui::components::form_theme_colour_picker_input::{
    form_theme_colour_picker_input, FormThemeColourPickerInputProps,
};
use cosmic::iced::Color;
use cosmic::iced::widget::{container, Container};
use cosmic::{widget, Theme};

impl SettingsForm {
    pub fn draw<'a>(&'a self, app_state: &'a SettingsApp) -> Container<'a, Message, Theme> {
        let theme = app_state.core().system_theme().cosmic();
        let mut column = widget::list_column().divider_padding(2);

        column = column.add(widget::text::title4(self.title.clone()));

        for (form_value_key, settings_form_item) in self.values.iter() {
            match settings_form_item.input_type {
                SettingsFormInputType::String => {
                    let red_color: Color = AppColour::Palette(BRIGHT_RED).resolve(theme);
                    let helper_color: Color = AppColour::Palette(EXT_WARM_GREY).resolve(theme);

                    let el = form_text_input(FormTextInputProps {
                        settings_window_id: self.settings_window_id,
                        label: settings_form_item.label.clone(),
                        value: &settings_form_item.value,
                        form_value_key,
                        validator: settings_form_item.validator,
                        error_color: red_color,
                        helper_text: settings_form_item.helper_text.clone(),
                        helper_text_color: helper_color,
                    });

                    column = column.add(el);
                }
                SettingsFormInputType::CheckBox => {
                    let converted_value = settings_form_item.value.parse::<bool>().unwrap_or(false);
                    let helper_color: Color = AppColour::Palette(EXT_WARM_GREY).resolve(theme);

                    let el = form_checkbox_input(FormCheckboxInputProps {
                        settings_window_id: self.settings_window_id,
                        label: settings_form_item.label.clone(),
                        value: converted_value,
                        form_value_key,
                        helper_text: settings_form_item.helper_text.clone(),
                        helper_text_color: helper_color,
                    });

                    column = column.add(el);
                }
                SettingsFormInputType::ColourPicker => {
                    let helper_color: Color = AppColour::Palette(EXT_WARM_GREY).resolve(theme);

                    let el = form_theme_colour_picker_input(FormThemeColourPickerInputProps {
                        settings_window_id: self.settings_window_id,
                        label: settings_form_item.label.clone(),
                        value: &settings_form_item.value,
                        form_value_key,
                        helper_text: settings_form_item.helper_text.clone(),
                        helper_text_color: helper_color,
                        custom_picker: app_state.custom_colour_picker(self.settings_window_id, form_value_key),
                    });

                    column = column.add(el);
                }
                SettingsFormInputType::FontPicker => {
                    let helper_color: Color = AppColour::Palette(EXT_WARM_GREY).resolve(theme);

                    let el = form_font_dropdown_input(FormFontDropdownInputProps {
                        settings_window_id: self.settings_window_id,
                        label: settings_form_item.label.clone(),
                        value: &settings_form_item.value,
                        form_value_key,
                        options: app_state.font_options(),
                        helper_text: settings_form_item.helper_text.clone(),
                        helper_text_color: helper_color,
                    });

                    column = column.add(el);
                }
                SettingsFormInputType::SpinButton { min, max } => {
                    let helper_color: Color = AppColour::Palette(EXT_WARM_GREY).resolve(theme);

                    let el = form_spin_button_input(FormSpinButtonInputProps {
                        settings_window_id: self.settings_window_id,
                        label: settings_form_item.label.clone(),
                        value: settings_form_item.value.trim().parse::<u16>().unwrap_or(min),
                        min,
                        max,
                        form_value_key,
                        helper_text: settings_form_item.helper_text.clone(),
                        helper_text_color: helper_color,
                    });

                    column = column.add(el);
                }
                SettingsFormInputType::MountList => {
                    let helper_color: Color = AppColour::Palette(EXT_WARM_GREY).resolve(theme);

                    let rows = form_mount_list_input(FormMountListInputProps {
                        settings_window_id: self.settings_window_id,
                        label: settings_form_item.label.clone(),
                        value: &settings_form_item.value,
                        form_value_key,
                        options: app_state.mount_options(),
                        helper_text: settings_form_item.helper_text.clone(),
                        helper_text_color: helper_color,
                    });

                    for row in rows {
                        column = column.add(row);
                    }
                }
            };
        }

        container(column).padding(2)
    }
}
