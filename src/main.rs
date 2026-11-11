#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod audio;
mod editor;
mod flowmodoro;
mod i18n;
mod impulse;
mod ramdump;
mod storage;
mod theme;
mod tray;
mod ui;
pub mod speedbump;
pub mod dopamine;

use libadwaita::prelude::*;
use libadwaita::Application;
use ui::TetherUi;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--speedbump" {
        speedbump::run_speedbump_dialog(&args[2..]);
        return;
    }

    let app = Application::builder()
        .application_id("org.tether.Tether")
        .build();

    app.connect_activate(|app| {
        let _ui = TetherUi::build(app);
    });

    app.run();
}
