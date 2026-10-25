//! What's New: a few plain lines about the newest version, shown once after
//! an update and from Help › What's New in Apature…. The words live in
//! `assets/whats-new.md` (newest section first, one line per change); the
//! changelog stays the long version.

use egui::RichText;

use super::tokens::{Theme, S1, S2, S3};
use crate::app::App;

const NOTES: &str = include_str!("../../assets/whats-new.md");

/// The newest section of `notes`: its version and its lines.
pub fn newest(notes: &str) -> (String, Vec<String>) {
    let mut version = String::new();
    let mut items = Vec::new();
    let mut in_section = false;
    for line in notes.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            if in_section {
                break;
            }
            in_section = true;
            version = h.trim().to_string();
        } else if in_section {
            if let Some(item) = line.strip_prefix("- ") {
                items.push(item.trim().to_string());
            }
        }
    }
    (version, items)
}

pub fn show(app: &mut App, ctx: &egui::Context, t: &Theme) {
    if !app.whats_new_open {
        return;
    }
    let (version, items) = newest(NOTES);
    let mut close = false;
    let modal = egui::Modal::new(egui::Id::new("whats-new")).show(ctx, |ui| {
        ui.set_width(420.0);
        ui.spacing_mut().item_spacing.y = S2;
        super::eyebrow(ui, t, "What’s new");
        ui.label(RichText::new(format!("Apature {version}")).size(20.0).strong().color(t.text));
        ui.add_space(S1);
        for item in &items {
            ui.horizontal_top(|ui| {
                ui.label(RichText::new("•").color(t.accent));
                ui.add(egui::Label::new(RichText::new(item).size(12.5).color(t.text2)).wrap());
            });
        }
        ui.add_space(S3);
        ui.horizontal(|ui| {
            ui.hyperlink_to("All changes", "https://github.com/McCal-Codes/apature/blob/main/CHANGELOG.md");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new(RichText::new("Got it").strong())).clicked() {
                    close = true;
                }
            });
        });
    });
    if close || modal.should_close() {
        app.whats_new_open = false;
        app.prefs.last_seen_version = env!("CARGO_PKG_VERSION").to_string();
        app.prefs_dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_section_only() {
        let (v, items) = newest("## 0.1.2

- One
- Two

## 0.1.1

- Old
");
        assert_eq!(v, "0.1.2");
        assert_eq!(items, vec!["One", "Two"]);
    }

    #[test]
    fn the_shipped_notes_are_short() {
        let (v, items) = newest(NOTES);
        assert!(!v.is_empty());
        assert!((1..=6).contains(&items.len()), "{} lines", items.len());
        assert!(items.iter().all(|l| l.len() <= 80), "one short line each");
    }
}
