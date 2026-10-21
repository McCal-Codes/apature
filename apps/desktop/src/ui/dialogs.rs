//! Modal dialogs for file operations: Rename Photos and Move to Trash.
//! Both show exactly what will happen before anything moves.

use egui::RichText;

use super::tokens::{Theme, S1, S2, S3, S4};
use crate::app::App;
use crate::commands::Command;

const THIRD_PARTY: &str = include_str!("../../../../docs/THIRD_PARTY.md");

/// About: name, version and build, what it is, where it lives, and the
/// third-party notices the licences require us to show.
fn about(app: &mut App, ctx: &egui::Context, t: &Theme) {
    if !app.about_open {
        return;
    }
    let modal = egui::Modal::new(egui::Id::new("about")).show(ctx, |ui| {
        ui.set_width(520.0);
        ui.vertical_centered(|ui| {
            ui.add_space(S3);
            ui.label(RichText::new("Apature").size(22.0).strong().color(t.text));
            ui.label(
                RichText::new(format!(
                    "Version {} · build {}",
                    env!("CARGO_PKG_VERSION"),
                    option_env!("APATURE_BUILD").unwrap_or("dev")
                ))
                .monospace()
                .size(11.0)
                .color(t.text2),
            );
            ui.add_space(S2);
            ui.label(RichText::new("Keyboard-first contact sheet, culling, metadata and delivery for photojournalists.").size(12.5).color(t.text2));
            ui.add_space(S2);
            ui.horizontal(|ui| {
                ui.hyperlink_to("Apature on GitHub", "https://github.com/McCal-Codes/apature");
                ui.label(RichText::new("·  © 2026 Caleb McCartney  ·  MIT or Apache-2.0").size(11.0).color(t.muted));
            });
        });
        ui.add_space(S3);
        super::eyebrow(ui, t, "Third-party notices");
        ui.add_space(S1);
        egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
            // The notices file is a Markdown table; show it as plain lines.
            let plain: String = THIRD_PARTY
                .lines()
                .filter(|l| !l.starts_with("|---"))
                .map(|l| l.trim_start_matches('#').replace(" | ", "   ").trim_matches(|c| c == '|' || c == ' ').to_string())
                .collect::<Vec<_>>()
                .join("\n");
            ui.add(egui::Label::new(RichText::new(plain).monospace().size(10.0).color(t.text2)).wrap());
        });
        ui.add_space(S3);
        ui.vertical_centered(|ui| {
            if ui.button("Close").clicked() {
                app.about_open = false;
            }
        });
    });
    if modal.should_close() {
        app.about_open = false;
    }
}

pub fn show(app: &mut App, ctx: &egui::Context, t: &Theme) {
    about(app, ctx, t);
    let mut cmds: Vec<Command> = Vec::new();

    if let Some(d) = app.rename_dialog.as_mut() {
        let n = d.targets.len();
        let mut changed = false;
        let modal = egui::Modal::new(egui::Id::new("rename-dialog")).show(ctx, |ui| {
            ui.set_width(560.0);
            ui.spacing_mut().item_spacing.y = S2;
            ui.label(RichText::new(format!("Rename {n} photo{}", if n == 1 { "" } else { "s" })).size(16.0).strong());
            ui.label(RichText::new("Every file of a photo moves together: RAW, JPEG and the XMP sidecar. Nothing is overwritten.").size(11.5).color(t.text2));
            ui.add_space(S2);
            ui.label(RichText::new("NEW NAME").monospace().size(9.5).color(t.muted));
            let r = ui.add(egui::TextEdit::singleline(&mut d.template).desired_width(f32::INFINITY).hint_text("100726-SPO-BGLF-{seqn:3}   tokens: {filename} {frame} {seqn} {headline} {date} {yymmdd} {year} {month} {day} {time}"));
            changed |= r.changed();
            if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && d.plan.as_ref().is_some_and(|p| p.can_run()) {
                cmds.push(Command::RenameApply);
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new("SEQUENCE STARTS AT").monospace().size(9.5).color(t.muted));
                changed |= ui.add(egui::DragValue::new(&mut d.seq_start).range(1..=99999)).changed();
            });
            ui.add_space(S2);
            if let Some(e) = &d.error {
                ui.label(RichText::new(e).size(12.0).color(t.reject));
            }
            if let Some(plan) = &d.plan {
                let files = plan.moves.len();
                ui.label(RichText::new(format!("{files} file{} will be renamed{}", if files == 1 { "" } else { "s" }, if plan.unchanged > 0 { format!(" · {} unchanged", plan.unchanged) } else { String::new() })).size(12.0).color(t.text));
                egui::ScrollArea::vertical().max_height(180.0).auto_shrink([false, true]).show(ui, |ui| {
                    for (from, to) in plan.moves.iter().take(40) {
                        let f = from.file_name().map(|x| x.to_string_lossy().into_owned()).unwrap_or_default();
                        let g = to.file_name().map(|x| x.to_string_lossy().into_owned()).unwrap_or_default();
                        ui.label(RichText::new(format!("{f}  →  {g}")).monospace().size(10.5).color(t.text2));
                    }
                    if plan.moves.len() > 40 {
                        ui.label(RichText::new(format!("… and {} more", plan.moves.len() - 40)).size(10.5).color(t.muted));
                    }
                    for (p, why) in &plan.collisions {
                        ui.label(RichText::new(format!("✕ {}  {why}", p.file_name().map(|x| x.to_string_lossy().into_owned()).unwrap_or_default())).monospace().size(10.5).color(t.reject));
                    }
                });
            }
            ui.add_space(S3);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let can = d.plan.as_ref().is_some_and(|p| p.can_run());
                    if ui.add_enabled(can, egui::Button::new(RichText::new("Rename").strong())).clicked() {
                        cmds.push(Command::RenameApply);
                    }
                    if ui.button("Cancel").clicked() {
                        cmds.push(Command::CloseDialog);
                    }
                });
            });
        });
        if modal.should_close() {
            cmds.push(Command::CloseDialog);
        }
        if changed {
            app.rename_recompute();
        }
    }

    if let Some(targets) = app.trash_dialog.clone() {
        let n = targets.len();
        let files: usize = targets
            .iter()
            .map(|&i| apature_core::fileops::item_files(&app.items[i]).len())
            .sum();
        let modal = egui::Modal::new(egui::Id::new("trash-dialog")).show(ctx, |ui| {
            ui.set_width(420.0);
            ui.spacing_mut().item_spacing.y = S2;
            ui.label(RichText::new(format!("Move {n} photo{} to the Trash?", if n == 1 { "" } else { "s" })).size(16.0).strong());
            ui.label(RichText::new(format!("{files} file{} including RAW, JPEG and sidecars. Apature never deletes: they stay in the Trash until you empty it.", if files == 1 { "" } else { "s" })).size(11.5).color(t.text2));
            egui::ScrollArea::vertical().max_height(120.0).auto_shrink([false, true]).show(ui, |ui| {
                for &i in targets.iter().take(20) {
                    ui.label(RichText::new(app.items[i].display_name()).monospace().size(10.5).color(t.text2));
                }
                if n > 20 {
                    ui.label(RichText::new(format!("… and {} more", n - 20)).size(10.5).color(t.muted));
                }
            });
            ui.add_space(S4);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new("Move to Trash").strong().color(t.reject))).clicked() {
                        cmds.push(Command::TrashConfirm);
                    }
                    if ui.button("Cancel").clicked() {
                        cmds.push(Command::CloseDialog);
                    }
                });
            });
        });
        if modal.should_close() {
            cmds.push(Command::CloseDialog);
        }
    }

    for c in cmds {
        app.dispatch(ctx, c);
    }
}
