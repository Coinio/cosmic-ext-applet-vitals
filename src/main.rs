// SPDX-License-Identifier: GPL-3.0-only

use log::LevelFilter;
use crate::app::AppState;
use systemd_journal_logger::JournalLog;

mod app;
mod core;
mod ui;
mod sensors;
mod monitors;
mod configuration;

fn main() -> cosmic::iced::Result {
    JournalLog::new().unwrap().install().unwrap();
    log::set_max_level(LevelFilter::Info);

    cosmic::applet::run::<AppState>(())
}
