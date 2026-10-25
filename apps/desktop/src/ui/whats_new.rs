//! What's New: the top section of the changelog, shown once after an update
//! and from Help › What's New in Apature…. The changelog file is the single
//! source: the page is whatever it says for the newest section.

use egui::RichText;

use super::tokens::{Theme, S1, S2, S3};
use crate::app::App;

const CHANGELOG: &str = include_str!("../../../../CHANGELOG.md");

/// One block of the page: a heading (Added, Changed, Fixed) and its bullets.
#[derive(Debug, PartialEq)]
pub struct Block {
    pub heading: String,
    pub items: Vec<String>,
}

/// The newest section of `changelog` as (title, blocks). Repeated headings
/// ("### Added" appears several times in a long [Unreleased]) are merged in
/// the order first seen.
pub fn newest(changelog: &str) -> (String, Vec<Block>) {
    let mut title = String::new();
    let mut blocks: Vec<Block> = Vec::new();
    let mut cur: Option<usize> = None;
    let mut in_section = false;
    for line in changelog.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            if in_section {
                break;
            }
            in_section = true;
            // "[0.1.1] - 2026-10-23" -> "0.1.1 · 2026-10-23"
            title = h.trim().replace("] - ", " · ").trim_matches(|c| c == '[' || c == ']').to_string();
        } else if !in_section {
            continue;
        } else if let Some(h) = line.strip_prefix("### ") {
            let h = h.trim();
            cur = Some(match blocks.iter().position(|b| b.heading == h) {
                Some(i) => i,
                None => {
                    blocks.push(Block { heading: h.to_string(), items: Vec::new() });
                    blocks.len() - 1
                }
            });
        } else if let Some(item) = line.strip_prefix("- ") {
            if let Some(i) = cur {
                blocks[i].items.push(plain(item));
            }
        } else if line.starts_with("  ") && !line.trim().is_empty() {
            if let Some(last) = cur.and_then(|i| blocks[i].items.last_mut()) {
                last.push(' ');
                last.push_str(&plain(line.trim()));
            }
        }
    }
    (title, blocks)
}

/// Markdown emphasis and code marks off, links reduced to their text.
fn plain(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '`' | '*' => {}
            '[' => {
                // [text](url) -> text
                let text: String = chars.by_ref().take_while(|&c| c != ']').collect();
                if chars.peek() == Some(&'(') {
                    chars.by_ref().take_while(|&c| c != ')').for_each(drop);
                }
                out.push_str(&text);
            }
            c => out.push(c),
        }
    }
    out
}

pub fn show(app: &mut App, ctx: &egui::Context, t: &Theme) {
    if !app.whats_new_open {
        return;
    }
    let (title, blocks) = newest(CHANGELOG);
    let mut close = false;
    let modal = egui::Modal::new(egui::Id::new("whats-new")).show(ctx, |ui| {
        ui.set_width(600.0);
        ui.spacing_mut().item_spacing.y = S2;
        super::eyebrow(ui, t, "What’s new");
        ui.label(RichText::new(format!("Apature {}", env!("CARGO_PKG_VERSION"))).size(20.0).strong().color(t.text));
        if !title.is_empty() {
            ui.label(RichText::new(format!("From the changelog: {title}")).monospace().size(10.5).color(t.muted));
        }
        ui.add_space(S1);
        egui::ScrollArea::vertical().max_height(380.0).auto_shrink([false, true]).show(ui, |ui| {
            for b in &blocks {
                if b.items.is_empty() {
                    continue;
                }
                ui.add_space(S2);
                ui.label(RichText::new(&b.heading).strong().size(13.0).color(t.accent));
                for item in &b.items {
                    ui.horizontal_top(|ui| {
                        ui.label(RichText::new("•").color(t.muted));
                        ui.add(egui::Label::new(RichText::new(item).size(12.0).color(t.text2)).wrap());
                    });
                }
            }
        });
        ui.add_space(S3);
        ui.horizontal(|ui| {
            ui.hyperlink_to("Full changelog", "https://github.com/McCal-Codes/apature/blob/main/CHANGELOG.md");
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
    fn newest_section_merges_repeated_headings_and_strips_marks() {
        let md = "# Changelog\n\nintro\n\n## [Unreleased]\n\n### Added\n- **One** with `code` and [a link](https://x.y).\n\n### Fixed\n- Two\n  continued here.\n\n### Added\n- Three\n\n## [0.1.1] - 2026-10-23\n\n### Added\n- Old\n";
        let (title, blocks) = newest(md);
        assert_eq!(title, "Unreleased");
        let added = blocks.iter().find(|b| b.heading == "Added").unwrap();
        assert_eq!(added.items, vec!["One with code and a link.", "Three"]);
        let fixed = blocks.iter().find(|b| b.heading == "Fixed").unwrap();
        assert_eq!(fixed.items, vec!["Two continued here."]);
        assert!(!blocks.iter().any(|b| b.items.contains(&"Old".to_string())));
    }
}
