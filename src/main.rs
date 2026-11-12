// SPDX-License-Identifier: GPL-3.0-only

use log::LevelFilter;
use crate::app::AppState;
use crate::configuration::general::FontSizeSlot;
use crate::settings_app::SettingsPage;
use systemd_journal_logger::JournalLog;

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

    cosmic::applet::run::<AppState>(())
}
