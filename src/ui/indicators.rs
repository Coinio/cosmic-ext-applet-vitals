use crate::app::{AppState, Message};
use crate::ui::display_item::DisplayItem;
use cosmic::applet::cosmic_panel_config::PanelSize;
use cosmic::widget;
use cosmic::Element;

const DEFAULT_INDICATOR_FONT_SIZE: u16 = 14;
const DEFAULT_INDICATOR_ICON_SIZE: u16 = 14;

pub struct IndicatorsUI;

impl IndicatorsUI {
    pub fn content<'a>(
        app_state: &'a AppState,
        display_item: &'a impl DisplayItem,
        horizontal: bool,
    ) -> Vec<Element<'a, Message>> {
        let core = app_state.core();
        let configuration = app_state.app_configuration();

        let mut content: Vec<Element<Message>> = Vec::new();

        let _label_color = display_item.label_color(&configuration);

        match display_item.label_icon(app_state).clone() {
            None => {}
            Some(handle) => {
                let label_icon = widget::icon::icon(handle.clone()).size(Self::label_icon_size(app_state));

                content.push(Element::from(label_icon));
            }
        };

        let mut value_text = core.applet.text(display_item.text(&configuration));
        if !horizontal {
            value_text = value_text.size(Self::label_text_vertical_font_size(app_state));
        }

        content.push(Element::from(value_text));

        content
    }

    fn label_icon_size(app_state: &AppState) -> u16 {
        match app_state.core().applet.size {
            cosmic::applet::Size::PanelSize(PanelSize::XS) => DEFAULT_INDICATOR_ICON_SIZE,
            cosmic::applet::Size::PanelSize(PanelSize::S) => DEFAULT_INDICATOR_ICON_SIZE + 4,
            cosmic::applet::Size::PanelSize(PanelSize::M) => DEFAULT_INDICATOR_ICON_SIZE + 8,
            cosmic::applet::Size::PanelSize(PanelSize::L) => DEFAULT_INDICATOR_ICON_SIZE + 10,
            cosmic::applet::Size::PanelSize(PanelSize::XL) => DEFAULT_INDICATOR_ICON_SIZE + 12,
            _ => DEFAULT_INDICATOR_ICON_SIZE,
        }
    }

    fn label_text_vertical_font_size(app_state: &AppState) -> u16 {
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
