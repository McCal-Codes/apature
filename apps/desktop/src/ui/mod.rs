//! The shell. Each module draws one region and talks to the app only
//! through `App` state and `Command`s; no module owns business logic.

pub mod assignments;
pub mod caption;
pub mod dialogs;
pub mod export;
pub mod filmstrip;
pub mod icons;
pub mod infobar;
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
pub mod welcome;
pub mod whats_new;

use apature_core::Rating;

pub fn rating_text(r: Rating) -> String {
    if r.is_rejected() {
        "✕".into()
    } else {
        "★".repeat(r.stars() as usize)
    }
}

/// A quiet card: eyebrow title, panel fill, hairline, rounded. The grouping
/// unit every workspace and Settings share (see docs/design/UI_LINEAGE.md).
pub fn card<R>(
    ui: &mut egui::Ui,
    t: &tokens::Theme,
    title: &str,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.add_space(tokens::S2);
    eyebrow(ui, t, title);
    ui.add_space(tokens::S1);
    egui::Frame::NONE
        .fill(t.raised)
        .stroke(egui::Stroke::new(1.0, t.line))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(tokens::S3 as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
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
