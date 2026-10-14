//! Apature desktop. `apature [folder] [--theme dark|light] [--screenshot out.png [--frames N]]`
//!
//! The screenshot flags exist so the shell can be checked visually in both
//! appearances without a bundled app or screen-recording permission.

mod app;
mod commands;
mod filter;
mod platform;
mod prefs;
mod selection;
mod ui;
mod undo;
mod worker;

use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct Launch {
    pub folder: Option<PathBuf>,
    pub theme: Option<egui::Theme>,
    pub screenshot: Option<PathBuf>,
    pub screenshot_frame: u64,
    /// Hidden: select everything and open the filter popup, for screenshots.
    pub demo: bool,
}

fn parse_args() -> Launch {
    let mut l = Launch {
        screenshot_frame: 120,
        ..Default::default()
    };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--theme" => {
                l.theme = match args.next().as_deref() {
                    Some("light") => Some(egui::Theme::Light),
                    Some("dark") => Some(egui::Theme::Dark),
                    _ => None,
                }
            }
            "--screenshot" => l.screenshot = args.next().map(PathBuf::from),
            "--demo" => l.demo = true,
            "--frames" => {
                l.screenshot_frame = args.next().and_then(|v| v.parse().ok()).unwrap_or(120)
            }
            _ => l.folder = Some(PathBuf::from(a)),
        }
    }
    l
}

/// The app icon, decoded from the bundled PNG (see assets/icon.svg for the source).
fn icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../assets/icon-256.png");
    let img = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (width, height) = img.dimensions();
    Some(egui::IconData {
        rgba: img.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result {
    env_logger::init();
    let launch = parse_args();
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1400.0, 900.0])
        .with_min_inner_size([760.0, 480.0])
        .with_title("Apature")
        .with_app_id("dev.mccal.apature");
    if let Some(i) = icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(i));
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "dev.mccal.apature",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, launch)))),
    )
}
