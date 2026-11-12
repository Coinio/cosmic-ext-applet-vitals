use crate::app::Message;
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_colours::AppColour;
use crate::core::app_icons::CPU_ICON;
use crate::fl;
use crate::monitors::cpu_monitor::{CoreUsage, CpuStats};
use crate::monitors::cpu_sensors_monitor::CpuSensorStats;
use crate::ui::components::detail_popup::{coloured_bar_class, detail_popup};
use crate::ui::components::indicator::{IndicatorContent, IndicatorIcon, IndicatorLine};
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::widget::{progress_bar, Column, Row};
use cosmic::iced::{Alignment, Length};
use cosmic::{widget, Element};

/// The thickness of each thread's or core's usage bar.
const USAGE_BAR_GIRTH: u16 = 6;
/// The width of the thread or core number before each bar, which fits 3 digits.
const USAGE_NUMBER_WIDTH: f32 = 24.0;
/// The width of the percentage after each bar, which fits "100%".
const USAGE_PERCENT_WIDTH: f32 = 34.0;
/// The spacing between the columns of usage bars.
const USAGE_COLUMN_SPACING: u16 = 16;

/// Up to this many threads or cores are listed in a single column.
const SINGLE_COLUMN_MAXIMUM: usize = 8;
/// Up to this many are listed in two columns. Any more are listed in four, without their percentages to fit, so even a
/// CPU with a hundred or more threads fits on screen.
const TWO_COLUMN_MAXIMUM: usize = 32;

impl CpuStats {
    pub fn content(&self, configuration: &AppConfiguration) -> Option<IndicatorContent> {
        let cpu = &configuration.cpu;
        if cpu.hide_indicator {
            return None;
        }

        Some(IndicatorContent {
            icon: (!cpu.hide_icon).then(|| IndicatorIcon::new(CPU_ICON, cpu.icon_colour.as_deref())),
            lines: vec![IndicatorLine::text(self.value())],
            max_label_text: "99.9%",
        })
    }

    fn value(&self) -> String {
        if self.cpu_usage_percent >= 100.0 {
            "100%".to_string()
        } else {
            format!("{:.1}%", self.cpu_usage_percent)
        }
    }

    /// The popup shown when hovering over the indicator: the temperature and fan speed when there are sensors for
    /// them, then the usage of each thread or core.
    pub fn detail<'a>(&'a self, sensors: &CpuSensorStats, configuration: &AppConfiguration) -> Element<'a, Message> {
        let cpu = &configuration.cpu;
        let mut sections: Vec<Element<'a, Message>> = Vec::new();

        let readings: Vec<(String, String)> = [
            sensors.temperature_celsius.map(|celsius| {
                (
                    fl!("popup-cpu-temperature"),
                    format_temperature(celsius, cpu.use_fahrenheit),
                )
            }),
            sensors.fan_rpm.map(|rpm| (fl!("popup-cpu-fan"), format_fan_speed(rpm))),
        ]
        .into_iter()
        .flatten()
        .collect();

        if !readings.is_empty() {
            let rows = readings.into_iter().map(|(label, value)| {
                Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(widget::text::body(label).width(Length::Fill))
                    .push(widget::text::body(value))
                    .into()
            });
            sections.push(Column::with_children(rows).spacing(4).into());
        }

        if !self.cores.is_empty() {
            let heading = if cpu.combine_cores {
                fl!("popup-cpu-cores")
            } else {
                fl!("popup-cpu-threads")
            };
            let bar_colour = cpu.icon_colour.as_deref().and_then(AppColour::parse);

            sections.push(
                Column::new()
                    .spacing(6)
                    .push(widget::text::caption_heading(heading))
                    .push(usage_grid(&self.cores, bar_colour))
                    .into(),
            );
        }

        detail_popup(fl!("popup-cpu-title"), sections)
    }
}

/// The usage bars in columns, filled top to bottom. CPUs usually number the second thread of each core after every
/// first thread, so in two columns each core's threads are side by side.
fn usage_grid<'a>(cores: &[CoreUsage], bar_colour: Option<AppColour>) -> Element<'a, Message> {
    let columns = column_count(cores.len());
    let rows = cores.len().div_ceil(columns);
    let show_percent = cores.len() <= TWO_COLUMN_MAXIMUM;

    let grid = cores.chunks(rows).map(|chunk| {
        let bars = chunk.iter().map(|core| usage_bar(core, bar_colour, show_percent));
        Column::with_children(bars).spacing(4).width(Length::Fill).into()
    });

    Row::with_children(grid).spacing(USAGE_COLUMN_SPACING).into()
}

fn column_count(count: usize) -> usize {
    match count {
        0..=SINGLE_COLUMN_MAXIMUM => 1,
        ..=TWO_COLUMN_MAXIMUM => 2,
        _ => 4,
    }
}

/// The thread or core number, from 1, the bar, and optionally its percentage.
fn usage_bar<'a>(core: &CoreUsage, bar_colour: Option<AppColour>, show_percent: bool) -> Element<'a, Message> {
    let fraction = (core.usage_percent / 100.0).clamp(0.0, 1.0) as f32;

    let mut row = Row::new()
        .spacing(6)
        .align_y(Alignment::Center)
        .push(widget::text::caption((core.number + 1).to_string()).width(Length::Fixed(USAGE_NUMBER_WIDTH)))
        .push(
            progress_bar(0.0..=1.0, fraction)
                .girth(USAGE_BAR_GIRTH)
                .class(coloured_bar_class(bar_colour)),
        );

    if show_percent {
        row = row.push(
            widget::text::caption(format!("{:.0}%", core.usage_percent.clamp(0.0, 100.0)))
                .width(Length::Fixed(USAGE_PERCENT_WIDTH))
                .align_x(Horizontal::Right),
        );
    }

    row.into()
}

/// A temperature in whole degrees, which is as precise as a CPU temperature is useful.
pub fn format_temperature(celsius: f64, use_fahrenheit: bool) -> String {
    if use_fahrenheit {
        format!("{:.0}°F", celsius * 9.0 / 5.0 + 32.0)
    } else {
        format!("{celsius:.0}°C")
    }
}

pub fn format_fan_speed(rpm: u32) -> String {
    fl!("popup-cpu-fan-speed", rpm = rpm.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_temperatures() {
        assert_eq!(format_temperature(45.625, false), "46°C");
        assert_eq!(format_temperature(0.0, false), "0°C");
        assert_eq!(format_temperature(100.0, true), "212°F");
        assert_eq!(format_temperature(-40.0, true), "-40°F");
    }

    #[test]
    fn spreads_many_threads_over_more_columns() {
        assert_eq!(column_count(1), 1);
        assert_eq!(column_count(8), 1);
        assert_eq!(column_count(16), 2);
        assert_eq!(column_count(32), 2);
        assert_eq!(column_count(64), 4);
    }
}
