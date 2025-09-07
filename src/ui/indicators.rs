use crate::app::{AppState, Message};
use crate::ui::display_item::DisplayItem;
use crate::ui::icons::{APP_LOGO_ICON, ICONS};
use cosmic::applet::cosmic_panel_config::PanelSize;
use cosmic::iced::{Alignment, Color};
use cosmic::iced_widget::Row;
use cosmic::theme::Svg;
use cosmic::widget;
use cosmic::widget::{row, svg, Column};
use cosmic::Element;
use std::rc::Rc;

const DEFAULT_INDICATOR_FONT_SIZE: u16 = 12;
const DEFAULT_INDICATOR_ICON_SIZE: u16 = 16;

pub const DEFAULT_INDICATOR_SPACING: u16 = 16;

pub struct IndicatorsUI;

impl IndicatorsUI {
    pub fn content<'a>(
        app_state: &'a AppState,
        display_item: &'a impl DisplayItem,
        horizontal: bool,
    ) -> Option<Element<'a, Message>> {
        let core = app_state.core();
        let configuration = app_state.app_configuration();

        if display_item.is_hidden(&configuration) {
            return None;
        }

        let mut content: Vec<Element<Message>> = Vec::new();

        let display_item_color = display_item.label_icon_color(app_state);

        match display_item.label_icon(app_state).clone() {
            None => {}
            Some(handle) => {
                let label_icon = widget::icon::icon(handle.clone())
                    .size(Self::label_icon_size(app_state))
                    .class(Svg::Custom {
                        0: Rc::new(move |t| {
                            let mut style = svg::Catalog::style(t, &Svg::default(), svg::Status::Idle);

                            style.color = Some(Color::from_rgba(
                                display_item_color.red,
                                display_item_color.green,
                                display_item_color.blue,
                                display_item_color.alpha,
                            ));
                            style
                        }),
                    });

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
                .spacing(app_state.core().applet.suggested_padding(false))
                .align_y(Alignment::Center)
                .into()
        } else {
            Column::from_vec(content).align_x(Alignment::Center).into()
        };

        Some(Element::from(row))
    }

    pub fn no_indicators_content(app_state: &AppState) -> Element<Message> {
        let handle = ICONS.get(APP_LOGO_ICON);

        match handle {
            None => Element::new(row()),
            Some(handle) => Element::from(widget::icon::icon(handle.clone()).size(Self::label_icon_size(app_state))),
        }
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
