// SPDX-License-Identifier: GPL-3.0-only

use crate::app::AppState;

mod app;
mod core;
mod ui;
mod sensors;
mod monitors;
mod configuration;

fn main() -> cosmic::iced::Result {
    colog::init();

    cosmic::applet::run::<AppState>(())
}
