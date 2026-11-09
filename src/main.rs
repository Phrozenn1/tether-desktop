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

use libadwaita::prelude::*;
use libadwaita::Application;
use ui::TetherUi;

fn main() {
    let app = Application::builder()
        .application_id("org.tether.Tether")
        .build();

    app.connect_activate(|app| {
        let _ui = TetherUi::build(app);
    });

    app.run();
}
