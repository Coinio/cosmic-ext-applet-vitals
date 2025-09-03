use crate::app::{AppState, Message};
use crate::ui::display_item::DisplayItem;
use cosmic::applet::cosmic_panel_config::PanelSize;
use cosmic::widget;
use cosmic::Element;
use cosmic::iced::Alignment;
use cosmic::iced_widget::Row;
use cosmic::widget::Column;

const DEFAULT_INDICATOR_FONT_SIZE: u16 = 12;
const DEFAULT_INDICATOR_ICON_SIZE: u16 = 18;

pub struct IndicatorsUI;

impl IndicatorsUI {
    pub fn content<'a>(
        app_state: &'a AppState,
        display_item: &'a impl DisplayItem,
        horizontal: bool,
    ) -> Element<'a, Message> {
        let core = app_state.core();
        let configuration = app_state.app_configuration();

        let mut content: Vec<Element<Message>> = Vec::new();

        match display_item.label_icon(app_state).clone() {
            None => {}
            Some(handle) => {
                let label_icon = widget::icon::icon(handle.clone())
                    .size(Self::label_icon_size(app_state));

                content.push(Element::from(label_icon));
            }
        };

        let mut value_text = core.applet.text(display_item.text(&configuration));

        if !horizontal {
            value_text = value_text.size(Self::label_text_vertical_font_size(app_state));
        }

        content.push(Element::from(value_text));
        
        let row: Element<Message> = if horizontal {
            Row::from_vec(content)
                .align_y(Alignment::Center)
                .spacing(app_state.core().applet.suggested_padding(true))
                .into()
        } else {
            Column::from_vec(content).align_x(Alignment::Center).into()
        };

        Element::from(row)
    }

    fn label_icon_size(app_state: &AppState) -> u16 {
        match app_state.core().applet.size {
            cosmic::applet::Size::PanelSize(PanelSize::XS) => DEFAULT_INDICATOR_ICON_SIZE,
            cosmic::applet::Size::PanelSize(PanelSize::S) => DEFAULT_INDICATOR_ICON_SIZE + 3,
            cosmic::applet::Size::PanelSize(PanelSize::M) => DEFAULT_INDICATOR_ICON_SIZE + 6,
            cosmic::applet::Size::PanelSize(PanelSize::L) => DEFAULT_INDICATOR_ICON_SIZE + 9,
            cosmic::applet::Size::PanelSize(PanelSize::XL) => DEFAULT_INDICATOR_ICON_SIZE + 10,
            _ => DEFAULT_INDICATOR_ICON_SIZE,
        }
    }

    fn label_text_vertical_font_size(app_state: &AppState) -> u16 {
        match app_state.core().applet.size {
            cosmic::applet::Size::PanelSize(PanelSize::XS) => DEFAULT_INDICATOR_FONT_SIZE - 3,
            cosmic::applet::Size::PanelSize(PanelSize::S) => DEFAULT_INDICATOR_FONT_SIZE - 1,
            cosmic::applet::Size::PanelSize(PanelSize::M) => DEFAULT_INDICATOR_FONT_SIZE,
            cosmic::applet::Size::PanelSize(PanelSize::L) => DEFAULT_INDICATOR_FONT_SIZE + 1,
            cosmic::applet::Size::PanelSize(PanelSize::XL) => DEFAULT_INDICATOR_FONT_SIZE + 2,
            _ => DEFAULT_INDICATOR_FONT_SIZE,
        }
    }
}
