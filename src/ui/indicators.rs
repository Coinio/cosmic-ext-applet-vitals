use crate::app::{AppState, Message};
use crate::ui::display_item::DisplayItem;
use cosmic::applet::cosmic_panel_config::PanelSize;
use cosmic::{widget, Element};
use cosmic::iced::alignment::Vertical;
use cosmic::widget::{button, container, row};

const DEFAULT_INDICATOR_FONT_SIZE: u16 = 14;

pub struct IndicatorsUI;

impl IndicatorsUI {
    pub fn content<'a>(
        app_state: &'a AppState,
        display_item: &'a impl DisplayItem,
        horizontal: bool,
    ) -> Vec<Element<'a, Message>> {
        let core = app_state.core();
        let configuration = app_state.configuration();

        let mut content: Vec<Element<Message>> = Vec::new();

        let label_color = display_item.label_color(&configuration);
        let label_text = core
            .applet
            .text(display_item.label(&configuration))
            .class(cosmic::theme::Text::from(label_color));
        
        let value_text = core.applet.text(display_item.text(&configuration));
        
        let label_text = if !horizontal {
            label_text.size(Self::vertical_font_size(app_state))
        } else { 
            label_text
        };
        
        let value_text = if !horizontal {
            value_text.size(Self::vertical_font_size(app_state))
        } else { 
            value_text
        };
        
        content.push(Element::from(label_text));
        content.push(Element::from(value_text));

        content
    }

    fn horizontal_font_size() -> u16 {
        DEFAULT_INDICATOR_FONT_SIZE
    }

    fn vertical_font_size(app_state: &AppState) -> u16 {
        match app_state.core().applet.size {
            cosmic::applet::Size::PanelSize(PanelSize::XS) => DEFAULT_INDICATOR_FONT_SIZE - 4,
            cosmic::applet::Size::PanelSize(PanelSize::S) => DEFAULT_INDICATOR_FONT_SIZE - 2,
            cosmic::applet::Size::PanelSize(PanelSize::M) => DEFAULT_INDICATOR_FONT_SIZE,
            cosmic::applet::Size::PanelSize(PanelSize::L) => DEFAULT_INDICATOR_FONT_SIZE + 2,
            cosmic::applet::Size::PanelSize(PanelSize::XL) => DEFAULT_INDICATOR_FONT_SIZE + 4,
            _ => DEFAULT_INDICATOR_FONT_SIZE,
        }
    }
}
