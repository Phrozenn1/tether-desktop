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
            .with_inner_size([460.0, 580.0])
            .with_min_inner_size([380.0, 440.0])
            .with_title("Tether — ADHD Flow & Save-State")
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "Tether",
        native_options,
        Box::new(|cc| Ok(Box::new(TetherApp::new(cc)))),
    )
}
