use crate::app::Message;
use crate::ui::components::indicator_value::{indicator_value, IndicatorValueProps};
use crate::ui::components::svg_icon::{svg_icon, SvgIconProps};
use cosmic::iced::Alignment;
use cosmic::iced::widget::Row;
use cosmic::widget::Column;
use cosmic::{widget, Element};

pub struct IndicatorValueItem<'a> {
    pub text: String,
    /// Drawn before the value, e.g. the direction of a data rate
    pub icon: Option<SvgIconProps<'a>>,
}

pub struct IndicatorProps<'a> {
    /// The icon identifying the indicator, drawn before its values
    pub icon: Option<SvgIconProps<'a>>,
    pub font_size: u16,
    pub font: &'a cosmic::font::Font,
    /// The height of the line box the value text is laid out in
    pub line_height: u16,
    /// The vertical centre of the rendered digits within the line box, which icons are aligned to. When unknown,
    /// icons are centred in the line box instead.
    pub digit_centre: Option<f32>,
    pub value_width: Option<f32>,
    pub horizontal: bool,
    pub spacing: u16,
    pub icon_spacing: u16,
    pub values: Vec<IndicatorValueItem<'a>>,
}

pub fn indicator<'a>(core: &'a cosmic::Core, props: IndicatorProps<'a>) -> Option<Element<'a, Message>> {
    let mut content: Vec<Element<Message>> = Vec::new();
    if let Some(icon_el) = props.icon.and_then(|icon| aligned_icon(icon, props.line_height, props.digit_centre)) {
        content.push(icon_el);
    }

    let mut value_rows: Vec<Element<Message>> = Vec::new();
    for item in props.values.into_iter() {
        let mut row = Row::new().spacing(props.icon_spacing).align_y(Alignment::Center);

        if let Some(icon_el) = item.icon.and_then(|icon| aligned_icon(icon, props.line_height, props.digit_centre)) {
            row = row.push(icon_el);
        }

        if let Some(value_el) = indicator_value(
            core,
            IndicatorValueProps {
                text: item.text,
                font_size: props.font_size,
                font: props.font,
                line_height: props.line_height,
                width: props.value_width,
                horizontal: props.horizontal,
            },
        ) {
            row = row.push(value_el);
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
        let mut values_container = Column::new().align_x(Alignment::Center);
        for v in value_rows {
            values_container = values_container.push(v);
        }
        content.push(values_container.into());

        Some(Column::from_vec(content).align_x(Alignment::Center).into())
    }
}

/// An icon in a box the height of the text line, so it shares the text's position within the row, offset so its
/// centre lines up with the centre of the rendered digits.
fn aligned_icon<'a>(mut icon: SvgIconProps<'a>, line_height: u16, digit_centre: Option<f32>) -> Option<Element<'a, Message>> {
    let Some(digit_centre) = digit_centre else {
        return svg_icon(icon).map(|el| widget::container(el).height(line_height).align_y(Alignment::Center).into());
    };

    // The gap above the icon, centre - size / 2, is only a whole number of pixels when twice the centre and the size
    // are both even or both odd. Shrink the icon by a pixel when they aren't, so it isn't drawn between pixels.
    let twice_centre = (digit_centre * 2.0).round() as i32;
    let mut size = icon.size as i32;
    if (twice_centre - size) % 2 != 0 {
        size -= 1;
    }
    let top = ((twice_centre - size) / 2).clamp(0, (line_height as i32 - size).max(0));
    icon.size = size.max(1) as u16;

    svg_icon(icon).map(|el| {
        widget::container(el)
            .height(line_height)
            .padding([top as u16, 0, 0, 0])
            .into()
    })
}
