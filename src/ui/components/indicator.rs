use crate::app::Message;
use crate::ui::components::indicator_label::{indicator_label, IndicatorLabelProps};
use crate::ui::components::indicator_value::{indicator_value, IndicatorValueProps};
use crate::ui::components::svg_icon::{svg_icon, SvgIconProps};
use cosmic::iced::{Alignment, Color};
use cosmic::iced_widget::Row;
use cosmic::widget::Column;
use cosmic::Element;

pub struct IndicatorValueItem<'a> {
    pub text: String,
    pub icon: Option<SvgIconProps<'a>>,
}

pub struct IndicatorProps<'a> {
    pub label_text: Option<String>,
    pub label_colour: Color,
    pub font_size: u16,
    pub value_width: Option<f32>,
    pub horizontal: bool,
    pub spacing: u16,
    pub icon_spacing: u16,
    pub values: Vec<IndicatorValueItem<'a>>,
}

pub fn indicator<'a>(core: &'a cosmic::Core, props: IndicatorProps<'a>) -> Option<Element<'a, Message>> {
    let label = indicator_label(
        core,
        IndicatorLabelProps {
            text: props.label_text,
            font_size: props.font_size,
            colour: props.label_colour,
        },
    );

    let mut content: Vec<Element<Message>> = Vec::new();
    if let Some(label_el) = label {
        content.push(label_el);
    }

    let mut value_rows: Vec<Element<Message>> = Vec::new();
    for item in props.values.into_iter() {
        let mut row = Row::new().spacing(props.icon_spacing).align_y(Alignment::Center);

        if let Some(value_el) = indicator_value(
            core,
            IndicatorValueProps {
                text: item.text,
                font_size: props.font_size,
                width: props.value_width,
                horizontal: props.horizontal,
            },
        ) {
            row = row.push(value_el);
        }

        if let Some(icon_props) = item.icon {
            if let Some(icon_el) = svg_icon(icon_props) {
                row = row.push(icon_el);
            }
        }

        value_rows.push(row.into());
    }

    if props.horizontal {
        let mut values_container = Row::new().align_y(Alignment::Center);
        for v in value_rows {
            values_container = values_container.spacing(props.spacing).push(v);
        }
        content.push(values_container.into());

        Some(
            Row::from_vec(content)
                .spacing(props.spacing)
                .align_y(Alignment::Center)
                .into(),
        )
    } else {
        // For vertical orientation stack value pairs in a column
        let mut values_container = Column::new().align_x(Alignment::Center);
        for v in value_rows {
            values_container = values_container.push(v);
        }
        content.push(values_container.into());

        Some(Column::from_vec(content).align_x(Alignment::Center).into())
    }
}
