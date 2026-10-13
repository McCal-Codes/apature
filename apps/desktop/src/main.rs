//! Desktop contact sheet. `photo-workflow [folder]`.

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
            .with_title("Photo Workflow"),
        ..Default::default()
    };
    eframe::run_native(
        "photo-workflow",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, folder)))),
    )
}
