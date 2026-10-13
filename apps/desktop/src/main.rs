//! Apature desktop contact sheet. `apature [folder]`.

mod app;
mod worker;

use std::path::PathBuf;

fn main() -> eframe::Result {
    env_logger::init();
    let folder = std::env::args().nth(1).map(PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([640.0, 400.0])
            .with_title("Apature"),
        ..Default::default()
    };
    eframe::run_native(
        "dev.mccal.apature",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, folder)))),
    )
}
