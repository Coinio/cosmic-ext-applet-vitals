use crate::app::{AppState, Message};
use crate::ui::app_icons::APP_LOGO_ICON;
use crate::ui::display_item::DisplayItem;
use cosmic::applet::cosmic_panel_config::{PanelAnchor, PanelSize};
use cosmic::iced::Alignment;
use cosmic::iced_widget::Row;
use cosmic::theme::Svg;
use cosmic::widget;
use cosmic::widget::{row, svg, Column};
use cosmic::Element;
use std::rc::Rc;


const DEFAULT_INDICATOR_FONT_SIZE: u16 = 14;
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
        
        let font_size = Self::label_font_size(app_state);

        let max_text_width = app_state
            .app_text_measurements()
            .measure(display_item.max_label_text(configuration), font_size);

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

                            style.color = Some(display_item_color);
                            style
                        }),
                    });

                content.push(Element::from(label_icon));
            }
        };

        let mut value_text = core
            .applet
            .text(display_item.label(&configuration))
            .width(max_text_width.unwrap_or(1.0))
            .size(font_size);

        if horizontal {
            value_text = value_text.align_x(Alignment::Start);
        } else {
            value_text = value_text.align_x(Alignment::Center);
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

    pub fn no_indicators_content(app_state: &'_ AppState) -> Element<'_, Message> {
        let handle = app_state.app_icons().get(APP_LOGO_ICON);

        match handle {
            None => Element::new(row()),
            Some(handle) => Element::from(widget::icon::icon(handle.clone()).size(Self::label_icon_size(app_state))),
        }
    }

    fn label_font_size(app_state: &AppState) -> u16 {
        let configuration = app_state.app_configuration();
        let is_horizontal = matches!(app_state.core().applet.anchor, PanelAnchor::Top | PanelAnchor::Bottom);

        match app_state.core().applet.size {
            cosmic::applet::Size::PanelSize(PanelSize::XS) if is_horizontal => configuration.general.horizontal_font_size_xs,
            cosmic::applet::Size::PanelSize(PanelSize::XS) => configuration.general.vertical_font_size_xs,
            cosmic::applet::Size::PanelSize(PanelSize::S) if is_horizontal => configuration.general.horizontal_font_size_sm,
            cosmic::applet::Size::PanelSize(PanelSize::S) => configuration.general.vertical_font_size_sm,
            cosmic::applet::Size::PanelSize(PanelSize::M) if is_horizontal => configuration.general.horizontal_font_size_md,
            cosmic::applet::Size::PanelSize(PanelSize::M) => configuration.general.vertical_font_size_md,
            cosmic::applet::Size::PanelSize(PanelSize::L) if is_horizontal => configuration.general.horizontal_font_size_lg,
            cosmic::applet::Size::PanelSize(PanelSize::L) => configuration.general.vertical_font_size_lg,
            cosmic::applet::Size::PanelSize(PanelSize::XL) if is_horizontal => configuration.general.horizontal_font_size_xl,
            cosmic::applet::Size::PanelSize(PanelSize::XL) => configuration.general.vertical_font_size_xl,
            _ => DEFAULT_INDICATOR_FONT_SIZE,
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
}
