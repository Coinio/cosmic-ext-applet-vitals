use crate::app::{AppState, Message};
use crate::ui::display_item::DisplayItem;
use cosmic::iced::alignment::Vertical;
use cosmic::iced_widget::container;
use cosmic::widget::{button, row, Button};
use cosmic::Element;

pub struct IndicatorsUI;

impl IndicatorsUI {
    pub fn content<'a>(
        app_state: &'a AppState,
        display_item: &'a impl DisplayItem,
    ) -> Button<'a, Message> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        let padding = core.applet.suggested_padding(false);
        let label_color = display_item.label_color(&configuration);

        let label_container = container(
            core.applet
                .text(display_item.label(&configuration))
                .class(cosmic::theme::Text::from(label_color))
                .font(cosmic::iced::Font {
                    weight: cosmic::iced::font::Weight::Medium,
                    ..Default::default()
                }),
        )
        .padding([0, padding]);

        let text_container = container(core.applet.text(display_item.text(&configuration)).font(
            cosmic::iced::Font {
                weight: cosmic::iced::font::Weight::Medium,
                ..Default::default()
            },
        ))
        .padding([0, padding]);

        let content = vec![Element::new(label_container), Element::new(text_container)];

        button::custom(Element::from(
            row::with_children(content).align_y(Vertical::Center),
        ))
        .on_press(Message::ToggleSettingsPopup(
            display_item.settings_window_id(),
        ))
        .class(cosmic::theme::Button::AppletMenu)
    }
}
