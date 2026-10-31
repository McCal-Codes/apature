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
            ui.horizontal(|ui| {
                if ui.button("What’s new").clicked() {
                    app.whats_new_open = true;
                    app.about_open = false;
                }
                if ui.button("Close").clicked() {
                    app.about_open = false;
                }
            });
        });
    });
    if modal.should_close() {
        app.about_open = false;
    }
}

/// Copy or Move Photos: choose, watch, read the result. Three phases in one
/// modal so the answer to "did it all verify?" is on the same screen.
fn transfer_dialog(app: &mut App, ctx: &egui::Context, t: &Theme) -> Vec<Command> {
    use apature_ingest::transfer::Mode;
    let mut cmds: Vec<Command> = Vec::new();
    let Some(d) = app.transfer_dialog.as_mut() else {
        return cmds;
    };
    let n = d.targets.len();
    let plural = if n == 1 { "" } else { "s" };
    let phase = match d.running.as_ref() {
        None => 0,
        Some(r) if !r.finished => 1,
        Some(_) => 2,
    };
    let modal = egui::Modal::new(egui::Id::new("transfer-dialog")).show(ctx, |ui| {
        ui.set_width(520.0);
        ui.spacing_mut().item_spacing.y = S2;
        match phase {
            0 => {
                ui.label(RichText::new(format!("Copy or move {n} photo{plural}")).size(16.0).strong());
                ui.label(RichText::new("Every file of a photo goes together (RAW, JPEG and the XMP sidecar). Each one is checked by SHA-256 and nothing is overwritten.").size(11.5).color(t.text2));
                ui.add_space(S2);
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut d.mode, Mode::Copy, "Copy");
                    ui.selectable_value(&mut d.mode, Mode::Move, "Move");
                });
                ui.label(
                    RichText::new(if d.mode == Mode::Move {
                        "Move: the originals go to the Trash only after every file of that photo is verified at the destination. ⌘Z brings them back."
                    } else {
                        "Copy: the originals stay where they are."
                    })
                    .size(11.5)
                    .color(t.text2),
                );
                ui.add_space(S2);
                ui.label(RichText::new("TO").monospace().size(9.5).color(t.muted));
                ui.horizontal(|ui| {
                    let text = d
                        .dest
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "No folder chosen".into());
                    ui.add(egui::Label::new(RichText::new(text).monospace().size(11.0).color(t.text)).truncate());
                    if ui.button("Choose…").clicked() {
                        cmds.push(Command::TransferChooseDest);
                    }
                });
                if d.card_present {
                    ui.checkbox(&mut d.use_card, "If the cloud drive has not downloaded a file, read it from the camera card");
                }
                if let Some(e) = &d.error {
                    ui.label(RichText::new(e).size(12.0).color(t.reject));
                }
                ui.add_space(S3);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let verb = if d.mode == Mode::Move { "Move" } else { "Copy" };
                        if ui.add(egui::Button::new(RichText::new(format!("{verb} {n} photo{plural}")).strong())).clicked() {
                            cmds.push(Command::TransferStart);
                        }
                        if ui.button("Cancel").clicked() {
                            cmds.push(Command::CloseDialog);
                        }
                    });
                });
            }
            1 => {
                let r = d.running.as_ref().unwrap();
                ui.label(RichText::new(format!("{} {n} photo{plural}…", if r.mode == Mode::Move { "Moving" } else { "Copying" })).size(16.0).strong());
                let frac = if r.total_files == 0 { 0.0 } else { r.done_files as f32 / r.total_files as f32 };
                ui.add(egui::ProgressBar::new(frac).text(format!("{} of {} files", r.done_files, r.total_files)));
                ui.label(RichText::new(&r.current).monospace().size(10.5).color(t.text2));
                ui.add_space(S3);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Stop after this photo").clicked() {
                            cmds.push(Command::CloseDialog);
                        }
                    });
                });
            }
            _ => {
                let r = d.running.as_ref().unwrap();
                let rep = r.report.as_ref();
                let photos = rep.map(|x| x.photos.len()).unwrap_or(0);
                let failed = rep.map(|x| x.failed()).unwrap_or(0);
                let trashed = rep.map(|x| x.trashed()).unwrap_or(0);
                let copied = rep.map(|x| x.files_copied()).unwrap_or(0);
                let there = rep.map(|x| x.already_there()).unwrap_or(0);
                let title = if failed == 0 { "Done" } else { "Done, with problems" };
                ui.label(RichText::new(title).size(16.0).strong().color(if failed == 0 { t.text } else { t.reject }));
                ui.label(RichText::new(format!("{photos} photo{} · {copied} file{} copied and verified · {:.1} MB{}", if photos == 1 { "" } else { "s" }, if copied == 1 { "" } else { "s" }, r.bytes as f64 / 1e6, if there > 0 { format!(" · {there} already there, identical") } else { String::new() })).size(12.0).color(t.text2));
                if r.mode == Mode::Move {
                    ui.label(RichText::new(format!("{trashed} photo{} sent to the Trash. ⌘Z puts them back.", if trashed == 1 { "" } else { "s" })).size(12.0).color(t.text2));
                }
                if failed > 0 {
                    ui.label(RichText::new(format!("{failed} photo{} did not transfer; their originals were left alone:", if failed == 1 { "" } else { "s" })).size(12.0).color(t.reject));
                    egui::ScrollArea::vertical().max_height(140.0).auto_shrink([false, true]).show(ui, |ui| {
                        for (name, why) in r.failed_files.iter().take(30) {
                            ui.label(RichText::new(format!("✕ {name}  {why}")).monospace().size(10.5).color(t.text2));
                        }
                    });
                }
                if rep.is_some_and(|x| x.photos.len() < n) {
                    ui.label(RichText::new(format!("Stopped early: {} photo{} were not touched.", n - photos, if n - photos == 1 { "" } else { "s" })).size(12.0).color(t.text2));
                }
                ui.add_space(S3);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(RichText::new("Done").strong())).clicked() {
                            cmds.push(Command::CloseDialog);
                        }
                        if ui.button(format!("Show in {}", crate::platform::Platform::file_manager())).clicked() {
                            cmds.push(Command::TransferOpenDest);
                        }
                    });
                });
            }
        }
    });
    if modal.should_close() && phase != 1 {
        cmds.push(Command::CloseDialog);
    }
    cmds
}

pub fn show(app: &mut App, ctx: &egui::Context, t: &Theme) {
    about(app, ctx, t);
    let mut cmds: Vec<Command> = transfer_dialog(app, ctx, t);

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
                ui.label(RichText::new("NUMBERS").monospace().size(9.5).color(t.muted));
                if ui.small_button("Keep each photo's number").on_hover_text("Use {oldseq}: the number already in a name like …Highlands0049-103026, so only the rest of the name changes").clicked() {
                    d.template = keep_numbers(&d.template);
                    changed = true;
                }
                if ui.small_button("Count from the start number").clicked() {
                    d.template = count_numbers(&d.template);
                    changed = true;
                }
            });
            if app.assignment.is_some() {
                ui.checkbox(&mut d.set_headline, "Also set the headline, title and job ID to the assignment's slug");
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

/// `{seqn}` / `{seqn:4}` / `{frame}` → `{oldseq}`: keep the number each photo already has.
pub fn keep_numbers(template: &str) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let after = &rest[i..];
        let Some(j) = after.find('}') else {
            out.push_str(after);
            return out;
        };
        let token = &after[..=j];
        if token == "{seqn}" || token.starts_with("{seqn:") || token == "{frame}" {
            out.push_str("{oldseq}");
        } else {
            out.push_str(token);
        }
        rest = &after[j + 1..];
    }
    out.push_str(rest);
    out
}

/// `{oldseq}` → `{seqn:4}`: number the photos from the start number again.
pub fn count_numbers(template: &str) -> String {
    template.replace("{oldseq}", "{seqn:4}")
}

#[cfg(test)]
mod rename_tests {
    use super::*;

    #[test]
    fn the_number_part_of_a_template_switches_both_ways() {
        let t = "BUTLR-{headline}{seqn:4}-{iptcmonth0}{iptcday0}{iptcyear2}";
        assert_eq!(
            keep_numbers(t),
            "BUTLR-{headline}{oldseq}-{iptcmonth0}{iptcday0}{iptcyear2}"
        );
        assert_eq!(keep_numbers("{headline}{seqn}"), "{headline}{oldseq}");
        assert_eq!(keep_numbers("{frame}-x"), "{oldseq}-x");
        assert_eq!(count_numbers("A{oldseq}"), "A{seqn:4}");
        assert_eq!(keep_numbers("no tokens {unclosed"), "no tokens {unclosed");
    }
}
