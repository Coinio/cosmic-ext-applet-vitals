// SPDX-License-Identifier: GPL-3.0-only

use log::LevelFilter;
use crate::app::AppState;
use crate::configuration::general::FontSizeSlot;
use crate::settings_app::SettingsPage;
use systemd_journal_logger::JournalLog;

/// The variable tokio reads the number of worker threads of its default runtime from.
pub const TOKIO_WORKER_THREADS: &str = "TOKIO_WORKER_THREADS";

mod app;
mod core;
mod ui;
mod sensors;
mod monitors;
mod configuration;
mod settings_app;

fn main() -> cosmic::iced::Result {
    JournalLog::new().unwrap().install().unwrap();
    log::set_max_level(LevelFilter::Info);

    let mut args = std::env::args().skip(1);

    // The applet opens its settings by launching this binary again, see `settings_app` for why.
    if args.next().as_deref() == Some(settings_app::SETTINGS_FLAG) {
        let args: Vec<String> = args.collect();
        let font_size_slot = args.iter().find_map(|arg| FontSizeSlot::from_arg(arg));
        let page = args.iter().find_map(|arg| SettingsPage::from_arg(arg)).unwrap_or_default();
        return settings_app::run(font_size_slot, page);
    }

    // The applet's background tasks, e.g. the monitors, are small and spend nearly all their time waiting, but the
    // default async runtime starts a worker thread for every CPU, and hands the tasks between them, which wakes several
    // threads on every tick. `applet::run` doesn't use the application's executor, so the runtime is limited to one
    // worker through the variable it reads its default from. Nothing else has started a thread yet.
    std::env::set_var(TOKIO_WORKER_THREADS, "1");

    cosmic::applet::run::<AppState>(())
}
