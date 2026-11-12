use crate::app::Message;
use crate::configuration::app_configuration::AppConfiguration;
use crate::core::app_colours::AppColour;
use crate::core::app_icons::MEMORY_ICON;
use crate::fl;
use crate::monitors::memory_monitor::MemoryStats;
use crate::ui::components::detail_popup::{detail_popup, detail_rows, labelled_bar, used_fraction};
use crate::ui::components::indicator::{IndicatorContent, IndicatorIcon, IndicatorLine};
use crate::ui::indicators::format_size_bytes;
use cosmic::Element;

const BYTES_PER_KIB: u64 = 1024;

impl MemoryStats {
    pub fn content(&self, configuration: &AppConfiguration) -> Option<IndicatorContent> {
        let memory = &configuration.memory;
        if memory.hide_indicator {
            return None;
        }

        let use_iec_units = configuration.general.use_iec_units;

        Some(IndicatorContent {
            icon: (!memory.hide_icon).then(|| IndicatorIcon::new(MEMORY_ICON, memory.icon_colour.as_deref())),
            lines: vec![IndicatorLine::text(self.value(use_iec_units))],
            max_label_text: if use_iec_units { "99.9GiB" } else { "99.9GB" },
        })
    }

    /// The popup shown when hovering over the indicator: the memory used, as the indicator shows it, what the kernel
    /// is using as cache and what is available, followed by the swap used when there is swap.
    pub fn detail(&self, configuration: &AppConfiguration) -> Element<'_, Message> {
        let bar_colour = configuration.memory.icon_colour.as_deref().and_then(AppColour::parse);
        let size = |kib: u64| format_size_bytes(kib.saturating_mul(BYTES_PER_KIB), configuration);
        let used_of = |used_kib: u64, total_kib: u64| {
            fl!("popup-memory-used-of", used = size(used_kib), total = size(total_kib))
        };

        let mut sections = vec![
            labelled_bar(
                fl!("popup-memory-used"),
                used_of(self.used_kib, self.total_kib),
                used_fraction(self.used_kib, self.total_kib),
                bar_colour,
            ),
            detail_rows([
                (fl!("popup-memory-cached"), size(self.cached_kib)),
                (fl!("popup-memory-available"), size(self.total_kib.saturating_sub(self.used_kib))),
            ]),
        ];

        if self.swap_total_kib > 0 {
            sections.push(labelled_bar(
                fl!("popup-memory-swap"),
                used_of(self.swap_used_kib, self.swap_total_kib),
                used_fraction(self.swap_used_kib, self.swap_total_kib),
                bar_colour,
            ));
        }

        detail_popup(fl!("popup-memory-title"), sections)
    }

    fn value(&self, use_iec_units: bool) -> String {
        if use_iec_units {
            let used_gib = self.used_kib as f64 / 1024.0 / 1024.0;
            if used_gib > 99.9 {
                format!("{:.0}GiB", used_gib.round())
            } else {
                format!("{:.1}GiB", used_gib)
            }
        } else {
            let used_gb = self.used_kib as f64 * 1024.0 / 1_000_000_000.0;
            if used_gb > 99.9 {
                format!("{:.0}GB", used_gb.round())
            } else {
                format!("{:.1}GB", used_gb)
            }
        }
    }
}
