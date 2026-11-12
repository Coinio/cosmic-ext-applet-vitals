use crate::app::AppState;
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_text_measurements::indicator_line_height;
use crate::core::app_colours::AppColour;
use crate::ui::components::svg_icon::SvgIconProps;

pub mod disk;
pub mod network;
pub mod cpu;
pub mod memory;
pub mod storage;

fn format_bytes_per_second(bytes_per_sec: u64, app_config: &AppConfiguration) -> String {
    if app_config.general.use_iec_units {
        let mib_per_second = bytes_per_sec as f64 / 1024.0 / 1024.0;
        if mib_per_second > 999.9 {
            let gib_per_second = bytes_per_sec as f64 / 1024.0 / 1024.0 / 1024.0;
            format!("{:.1}GiB/s", gib_per_second)
        } else if mib_per_second > 99.9 {
            format!("{:.0}MiB/s", mib_per_second.round())
        } else {
            format!("{:.1}MiB/s", mib_per_second)
        }
    } else {
        let mb_per_second = bytes_per_sec as f64 / 1_000_000.0;
        if mb_per_second > 999.9 {
            let gb_per_second = bytes_per_sec as f64 / 1_000_000_000.0;
            format!("{:.1}GB/s", gb_per_second)
        } else if mb_per_second > 99.9 {
            format!("{:.0}MB/s", mb_per_second.round())
        } else {
            format!("{:.1}MB/s", mb_per_second)
        }
    }
}
/// One of the app's icons, sized to sit alongside indicator text and tinted with `colour`. Without a colour the
/// icon keeps its own (white) fill.
fn themed_icon<'a>(app_state: &'a AppState, icon_key: &str, colour: Option<AppColour>, size: u16) -> SvgIconProps<'a> {
    SvgIconProps {
        icon: app_state.app_icons().get(icon_key),
        size,
        colour,
    }
}

/// The icon colour from a configuration value. Unrecognised values leave the icon untinted.
fn configured_colour(value: Option<&str>) -> Option<AppColour> {
    value.and_then(AppColour::parse)
}

/// The line height of the indicator text and the centre of the rendered digits within it, for aligning icons.
fn text_line_metrics(app_state: &AppState, font_size: u16) -> (u16, Option<f32>) {
    let line_height = indicator_line_height(font_size);
    let digit_centre = app_state
        .app_text_measurements()
        .digit_centre(font_size, line_height, app_state.indicator_font());
    (line_height, digit_centre)
}
