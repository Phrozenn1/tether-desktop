mod app;
mod flowmodoro;
mod impulse;
mod ramdump;
mod storage;

use app::TetherApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("tether")
            .with_inner_size([250.0, 60.0])
            .with_min_inner_size([180.0, 50.0])
            .with_title("Tether — ADHD Flow & Save-State")
            .with_always_on_top()
            .with_decorations(false)
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_native(
        "Tether",
        native_options,
        Box::new(|cc| Ok(Box::new(TetherApp::new(cc)))),
    )
}
