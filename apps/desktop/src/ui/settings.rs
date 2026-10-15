//! Settings, in its own window (⌘,), as macOS expects.

use egui::{Color32, RichText};

use apature_core::ColorClassScheme;

use super::tokens::{Theme, S2, S3, S4};
use crate::app::App;
use crate::commands::Command;
use crate::prefs::Appearance;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    General,
    Classes,
    Metadata,
    Ingest,
    Accessibility,
    Cache,
    Shortcuts,
}

const PAGES: [(Page, &str); 7] = [
    (Page::General, "General"),
    (Page::Classes, "Color classes"),
    (Page::Metadata, "Metadata"),
    (Page::Ingest, "Ingest"),
    (Page::Accessibility, "Accessibility"),
    (Page::Cache, "Cache"),
    (Page::Shortcuts, "Shortcuts"),
];

fn heading(ui: &mut egui::Ui, t: &Theme, text: &str) {
    ui.label(RichText::new(text).size(15.0).strong().color(t.text));
    ui.add_space(S2);
}

fn hint(ui: &mut egui::Ui, t: &Theme, text: &str) {
    ui.label(RichText::new(text).size(11.0).color(t.muted));
}

pub fn show(app: &mut App, ctx: &egui::Context, t: &Theme) {
    if !app.settings_open {
        return;
    }
    let mut cmds: Vec<Command> = Vec::new();
    let mut close = false;
    let t = *t;
    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("apature-settings"),
        egui::ViewportBuilder::default().with_title("Apature Settings").with_inner_size([680.0, 560.0]).with_min_inner_size([560.0, 420.0]),
        |ctx, _class| {
            if ctx.input(|i| i.viewport().close_requested()) {
                close = true;
            }
            // Screenshot support for this window (see main.rs --settings).
            if let Some(path) = app.screenshot_path() {
                let shot = ctx.input(|i| i.events.iter().find_map(|e| match e { egui::Event::Screenshot { image, .. } => Some(image.clone()), _ => None }));
                if let Some(img) = shot {
                    let [w, h] = img.size;
                    if let Some(buf) = image::RgbaImage::from_raw(w as u32, h as u32, img.as_raw().to_vec()) {
                        let _ = buf.save(&path);
                    }
                    ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
                }
            }
            egui::SidePanel::left("settings_nav")
                .exact_width(150.0)
                .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::same(S3 as i8)))
                .show(ctx, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    for (page, label) in PAGES {
                        let on = app.settings_page == page;
                        let r = ui.add_sized([ui.available_width(), 26.0], egui::Button::new(RichText::new(label).color(if on { t.accent } else { t.text2 })).fill(if on { t.accent_wash } else { Color32::TRANSPARENT }).frame(false));
                        if r.clicked() {
                            app.settings_page = page;
                        }
                    }
                });
            egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.window).inner_margin(egui::Margin::same(S4 as i8))).show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = S2;
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match app.settings_page {
                    Page::General => {
                        heading(ui, &t, "General");
                        ui.label(RichText::new("Appearance").size(12.0).color(t.text2));
                        ui.horizontal(|ui| {
                            for (a, label) in [(Appearance::System, "System"), (Appearance::Dark, "Apature Dark"), (Appearance::Light, "Apature Light")] {
                                if super::icons::chip(ui, &t, label, app.prefs.appearance == a).clicked() {
                                    app.prefs.appearance = a;
                                    app.prefs_dirty = true;
                                }
                            }
                        });
                        hint(ui, &t, "System follows System Settings › Appearance.");
                        ui.add_space(S3);
                        if ui.checkbox(&mut app.prefs.advance_after_mark, "Advance to the next photo after marking one").changed() {
                            app.prefs_dirty = true;
                        }
                        if ui.checkbox(&mut app.prefs.open_after_ingest, "Open the ingested folder when an ingest finishes").changed() {
                            app.prefs_dirty = true;
                        }
                        ui.add_space(S3);
                        ui.label(RichText::new("Default thumbnail size").size(12.0).color(t.text2));
                        let mut cell = app.prefs.cell;
                        if ui.add(egui::Slider::new(&mut cell, 80.0..=600.0).suffix(" pt")).changed() {
                            app.set_cell(cell);
                        }
                    }
                    Page::Classes => {
                        heading(ui, &t, "Color classes");
                        hint(ui, &t, "Names are written into xmp:Label, so they should match your Photo Mechanic preferences for labels to round-trip. The numeric class is always written too.");
                        ui.add_space(S2);
                        let mut changed = false;
                        egui::Grid::new("classes").num_columns(3).spacing([S3, S2]).show(ui, |ui| {
                            for i in 0..8 {
                                ui.label(RichText::new(format!("{}", i + 1)).monospace().color(t.muted));
                                let mut rgb = app.prefs.scheme.colors[i];
                                if ui.color_edit_button_srgb(&mut rgb).changed() {
                                    app.prefs.scheme.colors[i] = rgb;
                                    changed = true;
                                }
                                if ui.add(egui::TextEdit::singleline(&mut app.prefs.scheme.names[i]).desired_width(220.0)).changed() {
                                    changed = true;
                                }
                                ui.end_row();
                            }
                        });
                        ui.add_space(S2);
                        if ui.button("Reset to Photo Mechanic names and colors").clicked() {
                            app.prefs.scheme = ColorClassScheme::default();
                            changed = true;
                        }
                        if changed {
                            app.scheme_changed();
                        }
                    }
                    Page::Metadata => {
                        heading(ui, &t, "Metadata");
                        ui.label(RichText::new("Assignment file").size(12.0).color(t.text2));
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new(RichText::new(app.prefs.assignment_file.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "none".into())).monospace().size(10.5).color(t.text2)).truncate());
                            if ui.button("Choose…").clicked() {
                                cmds.push(Command::LoadAssignment);
                            }
                            if app.prefs.assignment_file.is_some() && ui.button("Clear").clicked() {
                                app.prefs.assignment_file = None;
                                app.assignment = None;
                                app.prefs_dirty = true;
                            }
                        });
                        ui.label(RichText::new("Code replacement file").size(12.0).color(t.text2));
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new(RichText::new(app.prefs.codes_file.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "none".into())).monospace().size(10.5).color(t.text2)).truncate());
                            if ui.button("Choose…").clicked() {
                                cmds.push(Command::LoadCodes);
                            }
                            if app.prefs.codes_file.is_some() && ui.button("Clear").clicked() {
                                app.prefs.codes_file = None;
                                app.prefs_dirty = true;
                            }
                        });
                        ui.add_space(S2);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Code delimiter").size(12.0).color(t.text2));
                            let mut d = app.prefs.code_delimiter.to_string();
                            if ui.add(egui::TextEdit::singleline(&mut d).desired_width(28.0).char_limit(1)).changed() {
                                if let Some(c) = d.chars().next() {
                                    app.prefs.code_delimiter = c;
                                    app.prefs_dirty = true;
                                }
                            }
                            hint(ui, &t, "Type \\cf\\ (or =cf= in Photo Mechanic's default) to expand a code.");
                        });
                        hint(ui, &t, &format!("{} codes loaded.", app.codes.codes.len()));
                    }
                    Page::Ingest => {
                        heading(ui, &t, "Ingest defaults");
                        let mut o = app.prefs.ingest.clone().unwrap_or_else(crate::prefs::default_ingest_options);
                        let mut changed = false;
                        if ui.checkbox(&mut app.prefs.seq_continue, "Keep {seqn} counting across ingests with the same headline").changed() {
                            app.prefs_dirty = true;
                        }
                        if !app.prefs.seq_counters.is_empty() {
                            hint(ui, &t, &format!("{} headline counter(s) remembered.", app.prefs.seq_counters.len()));
                            if ui.small_button("Forget all counters").clicked() {
                                app.prefs.seq_counters.clear();
                                app.prefs_dirty = true;
                            }
                        }
                        ui.add_space(S3);
                        ui.label(RichText::new("Folder template").size(12.0).color(t.text2));
                        changed |= ui.add(egui::TextEdit::singleline(&mut o.folder_template).desired_width(f32::INFINITY)).changed();
                        ui.label(RichText::new("Rename template").size(12.0).color(t.text2));
                        changed |= ui.add(egui::TextEdit::singleline(&mut o.rename_template).desired_width(f32::INFINITY)).changed();
                        changed |= ui.checkbox(&mut o.only_new, "Only photos not ingested before").changed();
                        if ui.button("Reset to defaults  ({year4}/{month0} {monthname}/{mnn3} {day0} {headline} · {headline}{seqn:3}-{month0}{day0}{yr2})").clicked() {
                            let keep = (o.source.clone(), o.primary.clone(), o.secondary.clone());
                            o = apature_ingest::Options { source: keep.0, primary: keep.1, secondary: keep.2, ..crate::prefs::default_ingest_options() };
                            changed = true;
                        }
                        changed |= ui.checkbox(&mut o.include_subfolders, "Include subfolders").changed();
                        changed |= ui.checkbox(&mut o.preserve_folders, "Keep the card's folders").changed();
                        changed |= ui.checkbox(&mut o.copy_companions, "Copy voice memos and sidecars with their photos").changed();
                        if ui.checkbox(&mut app.prefs.open_ingest_on_card, "When a camera card mounts, open Ingest with it as the source").changed() {
                            app.prefs_dirty = true;
                        }
                        hint(ui, &t, "tokens: {filename} {ext} {frame} {seqn} {seqn:4} {headline} {date} {yymmdd} {year} {month} {day} {time}");
                        if changed {
                            app.prefs.ingest = Some(o.clone());
                            app.ingest.opts = o;
                            app.prefs_dirty = true;
                        }
                    }
                    Page::Accessibility => {
                        heading(ui, &t, "Accessibility");
                        ui.label(RichText::new("Text and control size").size(12.0).color(t.text2));
                        let mut pct = (app.prefs.ui_scale * 100.0).round();
                        if ui.add(egui::Slider::new(&mut pct, 80.0..=180.0).step_by(5.0).suffix(" %")).changed() {
                            app.prefs.ui_scale = pct / 100.0;
                            app.prefs_dirty = true;
                        }
                        hint(ui, &t, "Scales every panel, label and control. ⌘+ and ⌘− do the same while the app is open; 100 % is the default.");
                        ui.add_space(S3);
                        ui.label(RichText::new("Motion").size(12.0).color(t.text2));
                        ui.horizontal(|ui| {
                            for (m, label) in [(crate::prefs::Motion::System, "Follow macOS Reduce Motion"), (crate::prefs::Motion::Reduce, "Reduce"), (crate::prefs::Motion::Full, "Full")] {
                                if super::icons::chip(ui, &t, label, app.prefs.motion == m).clicked() {
                                    app.prefs.motion = m;
                                    app.prefs_dirty = true;
                                }
                            }
                        });
                        hint(ui, &t, "Reduce turns fades, slides and the ingest stagger into instant changes. Nothing in Apature loops or autoplays either way.");
                        ui.add_space(S3);
                        if ui.checkbox(&mut app.prefs.high_contrast, "Higher contrast text and borders").changed() {
                            app.prefs_dirty = true;
                        }
                        hint(ui, &t, "Lifts the quiet grey labels and strengthens panel borders in both appearances.");
                        ui.add_space(S3);
                        hint(ui, &t, "Keyboard: every workspace is reachable with ⌘⇧1–5, the sheet with arrows, marks with the number keys (see Shortcuts). Screen reader labels are still being filled in (see docs/ACCESSIBILITY.md).");
                    }
                    Page::Cache => {
                        heading(ui, &t, "Thumbnail cache");
                        let dir = app.cache_dir();
                        ui.add(egui::Label::new(RichText::new(dir.display().to_string()).monospace().size(10.5).color(t.text2)).truncate());
                        ui.horizontal(|ui| {
                            if ui.button("Calculate size").clicked() {
                                app.cache_size = Some(dir_size(&dir));
                            }
                            if let Some((files, bytes)) = app.cache_size {
                                ui.label(RichText::new(format!("{files} thumbnails · {:.0} MB", bytes as f64 / 1e6)).size(12.0).color(t.text2));
                            }
                        });
                        if ui.button("Clear thumbnail cache").on_hover_text("Thumbnails are regenerated from the photos as needed").clicked() {
                            cmds.push(Command::ClearThumbCache);
                        }
                        hint(ui, &t, "Originals are never touched; the cache only holds 320 px JPEG thumbnails.");
                    }
                    Page::Shortcuts => {
                        heading(ui, &t, "Shortcuts");
                        ui.label(RichText::new("Number keys 1–8").size(12.0).color(t.text2));
                        ui.horizontal_wrapped(|ui| {
                            for (m, label) in [
                                (crate::prefs::NumberKeys::Rating, "Stars: 1–5 rate, 0 clears"),
                                (crate::prefs::NumberKeys::ColorClass, "Colour class: 1–8, 0 clears (Photo Mechanic)"),
                            ] {
                                if super::icons::chip(ui, &t, label, app.prefs.number_keys == m).clicked() {
                                    app.prefs.number_keys = m;
                                    app.prefs_dirty = true;
                                }
                            }
                        });
                        hint(ui, &t, match app.prefs.number_keys {
                            crate::prefs::NumberKeys::Rating => "⌘1–⌘8 set the colour class, ⌘0 clears it.",
                            crate::prefs::NumberKeys::ColorClass => "⌘0–⌘5 set the rating.",
                        });
                        ui.add_space(S3);
                        hint(ui, &t, "Editable profiles arrive later.");
                        ui.add_space(S2);
                        let (num_keys, num_row, cmd_keys, cmd_row): (&str, &str, &str, &str) = match app.prefs.number_keys {
                            crate::prefs::NumberKeys::Rating => ("1–5 · 0", "Rating · clear", "⌘1–⌘8 · ⌘0", "Colour class · clear"),
                            crate::prefs::NumberKeys::ColorClass => ("1–8 · 0", "Colour class · clear", "⌘0–⌘5", "Rating · clear"),
                        };
                        egui::Grid::new("keys").num_columns(2).spacing([S4, S2]).striped(true).show(ui, |ui| {
                            ui.label(RichText::new(num_keys).monospace().size(11.5).color(t.text));
                            ui.label(RichText::new(num_row).size(12.0).color(t.text2));
                            ui.end_row();
                            ui.label(RichText::new(cmd_keys).monospace().size(11.5).color(t.text));
                            ui.label(RichText::new(cmd_row).size(12.0).color(t.text2));
                            ui.end_row();
                            for (k, v) in SHORTCUTS {
                                ui.label(RichText::new(*k).monospace().size(11.5).color(t.text));
                                ui.label(RichText::new(*v).size(12.0).color(t.text2));
                                ui.end_row();
                            }
                        });
                    }
                });
            });
        },
    );
    if close {
        app.settings_open = false;
    }
    for c in cmds {
        app.dispatch(ctx, c);
    }
}

const SHORTCUTS: &[(&str, &str)] = &[
    (
        "← → ↑ ↓",
        "Move focus (⇧ extends the selection, ⌘ keeps it)",
    ),
    ("X · T", "Reject · toggle tag"),
    ("⏎ / Space · Esc", "Preview · back"),
    ("⌘A · ⌘⇧A", "Select all · none"),
    ("⌘Z · ⌘⇧Z", "Undo · redo"),
    ("⌘⇧1 … ⌘⇧5", "Sheet · Preview · Caption · Ingest · Export"),
    ("⌘E", "Export…"),
    ("⌘= · ⌘-", "Larger · smaller thumbnails"),
    ("⌃⌘S · ⌥⌘I · ⇧F", "Sidebar · inspector · filmstrip"),
    ("⌥⌘F", "Clear filters"),
    ("⌘O / ⌘T · ⌘W", "Open folder in a new tab · close tab"),
    ("⌘⇧] · ⌘⇧[", "Next · previous tab"),
    ("⌘⏎", "Save caption and advance"),
    ("⌘,", "Settings"),
];

fn dir_size(dir: &std::path::Path) -> (usize, u64) {
    let mut files = 0;
    let mut bytes = 0;
    for e in walkdir::WalkDir::new(dir).into_iter().flatten() {
        if e.file_type().is_file() {
            files += 1;
            bytes += e.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    (files, bytes)
}
