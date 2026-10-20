//! The shell. Each module draws one region and talks to the app only
//! through `App` state and `Command`s; no module owns business logic.

pub mod caption;
pub mod dialogs;
pub mod export;
pub mod filmstrip;
pub mod icons;
pub mod ingest;
pub mod inspector;
pub mod motion;
pub mod preview;
pub mod settings;
pub mod sheet;
pub mod sidebar;
pub mod status;
pub mod tabs;
pub mod toasts;
pub mod tokens;
pub mod toolbar;

use apature_core::Rating;

pub fn rating_text(r: Rating) -> String {
    if r.is_rejected() {
        "✕".into()
    } else {
        "★".repeat(r.stars() as usize)
    }
}

pub fn eyebrow(ui: &mut egui::Ui, t: &tokens::Theme, text: &str) {
    ui.label(
        egui::RichText::new(text.to_uppercase())
            .size(10.0)
            .strong()
            .color(t.muted),
    );
}

pub fn mono(text: impl Into<String>, size: f32, color: egui::Color32) -> egui::RichText {
    egui::RichText::new(text)
        .monospace()
        .size(size)
        .color(color)
}
