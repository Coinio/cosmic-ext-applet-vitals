use crate::app::Message;
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_colours::AppColour;
use crate::core::app_icons::{AppIcons, GPU_ICON};
use crate::fl;
use crate::monitors::gpu_monitor::{GpuDeviceStats, GpuState, GpuStats};
use crate::ui::components::detail_popup::{
    card, detail_popup, heading_row, inline_bar, labelled_bar, popup_header, stat_tiles, used_fraction, PopupStyle,
    ROW_SPACING,
};
use crate::ui::components::indicator::{IndicatorContent, IndicatorIcon, IndicatorLine};
use crate::ui::indicators::cpu::format_temperature;
use crate::ui::indicators::format_size_bytes;
use cosmic::iced::widget::Column;
use cosmic::Element;

/// The width of the "LOAD" label before each GPU's usage bar.
const LOAD_LABEL_WIDTH: f32 = 34.0;
/// Shown in place of a value the GPU didn't report, e.g. while it's asleep.
const NO_VALUE: &str = "–";

impl GpuStats {
    /// The panel's GPU's usage, and optionally its VRAM used. There's no indicator without a supported GPU.
    pub fn content(&self, configuration: &AppConfiguration) -> Option<IndicatorContent> {
        let gpu_configuration = &configuration.gpu;
        if gpu_configuration.hide_indicator {
            return None;
        }
        let gpu = self.panel_gpu()?;

        let mut lines = vec![IndicatorLine::text(usage_value(gpu))];
        if gpu_configuration.show_vram {
            lines.push(IndicatorLine::text(
                gpu.vram_used_bytes
                    .map_or_else(|| NO_VALUE.to_string(), |used| format_size_bytes(used, configuration)),
            ));
        }

        let max_label_text = match (gpu_configuration.show_vram, configuration.general.use_iec_units) {
            (false, _) => "99.9%",
            (true, true) => "99.9GiB",
            (true, false) => "99.9GB",
        };

        Some(IndicatorContent {
            icon: (!gpu_configuration.hide_icon)
                .then(|| IndicatorIcon::new(GPU_ICON, gpu_configuration.icon_colour.as_deref())),
            lines,
            max_label_text,
        })
    }

    /// The popup shown when hovering over the indicator, with a card for every GPU, not only the panel's.
    pub fn detail<'a>(
        &'a self,
        configuration: &AppConfiguration,
        app_icons: &'a AppIcons,
        style: PopupStyle,
    ) -> Element<'a, Message> {
        let colour = configuration.gpu.icon_colour.as_deref().and_then(AppColour::parse);

        let cards = self
            .gpus
            .iter()
            .map(|gpu| card(gpu_detail(style, gpu, configuration, colour)))
            .collect::<Vec<_>>();

        let header = popup_header(
            style,
            app_icons.get(GPU_ICON),
            colour,
            fl!("popup-gpu-title"),
            self.panel_gpu().map(|gpu| style.summary(usage_value(gpu), colour).into()),
        );

        detail_popup(header, cards)
    }
}

/// The usage as the indicator shows it. A GPU that's asleep isn't doing anything.
fn usage_value(gpu: &GpuDeviceStats) -> String {
    match (gpu.state, gpu.usage_percent) {
        (GpuState::Asleep, _) => "0%".to_string(),
        (_, Some(percent)) if percent >= 100.0 => "100%".to_string(),
        (_, Some(percent)) => format!("{percent:.1}%"),
        (_, None) => NO_VALUE.to_string(),
    }
}

/// The GPU's name, and its state when it isn't being read, above its usage, VRAM, temperature and power.
fn gpu_detail<'a>(
    style: PopupStyle,
    gpu: &'a GpuDeviceStats,
    configuration: &AppConfiguration,
    colour: Option<AppColour>,
) -> Element<'a, Message> {
    let caption = match gpu.state {
        GpuState::Active => gpu.pci_address.clone(),
        GpuState::Idle => fl!("popup-gpu-idle"),
        GpuState::Asleep => fl!("popup-gpu-asleep"),
        GpuState::Unavailable => fl!("popup-gpu-unavailable"),
    };

    let mut column = Column::new()
        .spacing(ROW_SPACING)
        .push(heading_row(style, gpu.name.as_str(), Some(caption)));

    if gpu.state == GpuState::Asleep {
        return column.into();
    }

    if let Some(usage) = gpu.usage_percent {
        column = column.push(inline_bar(
            style,
            style.section_heading(fl!("popup-gpu-load")),
            LOAD_LABEL_WIDTH,
            Some(usage),
            (usage.clamp(0.0, 100.0) / 100.0) as f32,
            colour,
        ));
    }

    if let (Some(used), Some(total)) = (gpu.vram_used_bytes, gpu.vram_total_bytes) {
        column = column.push(labelled_bar(
            style,
            fl!("popup-gpu-vram"),
            fl!(
                "popup-gpu-vram-used-of",
                used = format_size_bytes(used, configuration),
                total = format_size_bytes(total, configuration)
            ),
            used_fraction(used, total),
            colour,
        ));
    }

    let readings: Vec<(String, String)> = [
        gpu.temperature_celsius.map(|celsius| {
            (
                fl!("popup-gpu-temperature"),
                format_temperature(celsius, configuration.cpu.use_fahrenheit),
            )
        }),
        gpu.power_watts
            .map(|watts| (fl!("popup-gpu-power"), fl!("popup-gpu-power-watts", watts = format!("{watts:.0}")))),
    ]
    .into_iter()
    .flatten()
    .collect();

    if !readings.is_empty() {
        column = column.push(stat_tiles(style, readings));
    }

    column.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(gpu: GpuDeviceStats) -> GpuStats {
        GpuStats {
            gpus: vec![gpu],
            panel: Some(0),
        }
    }

    fn texts(content: &IndicatorContent) -> Vec<&str> {
        content.lines.iter().map(|line| line.text.as_str()).collect()
    }

    #[test]
    fn shows_the_usage_and_vram_of_the_panel_gpu() {
        let configuration = AppConfiguration::default();
        let gpu = GpuDeviceStats {
            usage_percent: Some(37.25),
            vram_used_bytes: Some(3_200_000_000),
            vram_total_bytes: Some(8_000_000_000),
            ..GpuDeviceStats::default()
        };

        let content = stats(gpu).content(&configuration).unwrap();

        assert_eq!(texts(&content), vec!["37.2%", "3.2GB"]);
    }

    #[test]
    fn the_vram_can_be_hidden() {
        let mut configuration = AppConfiguration::default();
        configuration.gpu.show_vram = false;
        let gpu = GpuDeviceStats {
            usage_percent: Some(100.0),
            ..GpuDeviceStats::default()
        };

        let content = stats(gpu).content(&configuration).unwrap();

        assert_eq!(texts(&content), vec!["100%"]);
        assert_eq!(content.max_label_text, "99.9%");
    }

    #[test]
    fn a_sleeping_gpu_is_idle_without_vram() {
        let gpu = GpuDeviceStats {
            state: GpuState::Asleep,
            vram_total_bytes: Some(8_000_000_000),
            ..GpuDeviceStats::default()
        };

        let content = stats(gpu).content(&AppConfiguration::default()).unwrap();

        assert_eq!(texts(&content), vec!["0%", NO_VALUE]);
    }

    #[test]
    fn there_is_no_indicator_without_a_gpu() {
        assert!(GpuStats::default().content(&AppConfiguration::default()).is_none());
    }
}
