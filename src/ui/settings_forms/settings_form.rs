use crate::app::{AppState, Message};
use crate::configuration::app_configuration::*;
use crate::core::app_colours::{AppColours, BRIGHT_RED};
use crate::core::settings::{SettingsForm, SettingsFormInputType};
use crate::ui::components::form_checkbox_input::{form_checkbox_input, FormCheckboxInputProps};
use crate::ui::components::form_text_input::{form_text_input, FormTextInputProps};
use crate::ui::components::form_theme_colour_picker_input::{
    form_theme_colour_picker_input, FormThemeColourPickerInputProps,
};
use cosmic::iced::Color;
use cosmic::iced_widget::{container, Container};
use cosmic::widget::settings;
use cosmic::{widget, Theme};

impl SettingsForm {
    pub fn draw(&self, app_state: &AppState) -> Container<'_, Message, Theme> {
        let mut column = widget::list_column().padding(2).spacing(0).divider_padding(2);

        let back_button = widget::button::custom(widget::icon::from_name("go-previous-symbolic").size(16).icon())
            .on_press(Message::SettingsPopupOpened(MAIN_SETTINGS_WINDOW_ID.clone()));

        column = column.add(settings::item(self.title.clone(), back_button));

        for (form_value_key, settings_form_item) in self.values.iter() {
            match settings_form_item.input_type {
                SettingsFormInputType::String => {
                    let red_color: Color = app_state
                        .app_colours()
                        .get(BRIGHT_RED)
                        .map_or(Color::WHITE, |c| Color::new(c.red, c.green, c.blue, c.alpha));

                    let el = form_text_input(FormTextInputProps {
                        settings_window_id: self.settings_window_id,
                        label: settings_form_item.label.clone(),
                        value: &settings_form_item.value,
                        form_value_key,
                        validator: settings_form_item.validator,
                        error_color: red_color,
                    });

                    column = column.add(el);
                }
                SettingsFormInputType::CheckBox => {
                    let converted_value = settings_form_item.value.parse::<bool>().unwrap_or(false);

                    let el = form_checkbox_input(FormCheckboxInputProps {
                        settings_window_id: self.settings_window_id,
                        label: settings_form_item.label.clone(),
                        value: converted_value,
                        form_value_key,
                    });

                    column = column.add(el);
                }
                SettingsFormInputType::ColourPicker => {
                    let palette = &app_state.core().system_theme().cosmic().palette;
                    let app_colours = AppColours::from(palette);

                    let el = form_theme_colour_picker_input(FormThemeColourPickerInputProps {
                        settings_window_id: self.settings_window_id,
                        label: settings_form_item.label.clone(),
                        selected_key: settings_form_item.value.clone(),
                        form_value_key,
                        app_colours,
                    });

                    column = column.add(el);
                }
            };
        }

        container(column)
    }
}
