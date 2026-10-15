//! Ingest workspace: source, destinations, naming, the plan, then progress
//! as a list that never hides a failure.

use egui::{Color32, CornerRadius, RichText};

use apature_ingest::FileFilter;

use super::tokens::{Theme, S1, S2, S3, S4, S5};
use crate::app::App;
use crate::commands::Command;
use crate::ingest_job::FileStatus;

fn labeled(ui: &mut egui::Ui, t: &Theme, label: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.label(
        RichText::new(label.to_uppercase())
            .monospace()
            .size(9.5)
            .color(t.muted),
    );
    add(ui);
    ui.add_space(S1);
}

fn human_bytes(b: u64) -> String {
    const GB: f64 = 1e9;
    const MB: f64 = 1e6;
    let f = b as f64;
    if f >= GB {
        format!("{:.1} GB", f / GB)
    } else {
        format!("{:.0} MB", f / MB)
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui, t: &Theme) {
    let mut cmds: Vec<Command> = Vec::new();
    let mut opts_changed = false;
    let running = app.ingest.running.is_some()
        && app
            .ingest
            .running
            .as_ref()
            .is_some_and(|r| r.done.is_none());

    egui::Frame::NONE.fill(t.photo).inner_margin(egui::Margin::same(S5 as i8)).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = S2;
        ui.add_enabled_ui(!running, |ui| {
            ui.columns(2, |cols| {
                // ---- Source ----
                let ui = &mut cols[0];
                egui::Frame::NONE.fill(t.panel).stroke(egui::Stroke::new(1.0, t.line)).corner_radius(CornerRadius::same(8)).inner_margin(egui::Margin::same(S4 as i8)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    super::eyebrow(ui, t, "Source");
                    ui.add_space(S2);
                    ui.horizontal_wrapped(|ui| {
                        for (name, path) in app.volumes.clone() {
                            let on = app.ingest.opts.source == path;
                            if ui.selectable_label(on, format!("▣ {name}")).clicked() {
                                app.ingest.opts.source = path;
                                opts_changed = true;
                            }
                        }
                        if ui.button("Folder…").clicked() {
                            cmds.push(Command::IngestChooseSource);
                        }
                    });
                    let src = app.ingest.opts.source.display().to_string();
                    ui.label(RichText::new(if src.is_empty() { "no source chosen".into() } else { src }).monospace().size(10.5).color(t.text2));
                    ui.add_space(S2);
                    if ui.checkbox(&mut app.ingest.opts.include_subfolders, "Include subfolders (DCIM/100NIKON…)").changed() {
                        opts_changed = true;
                    }
                    if ui.checkbox(&mut app.ingest.opts.only_new, "Only photos not ingested before").changed() {
                        opts_changed = true;
                    }
                    let mut suffix = app.ingest.opts.on_collision == apature_ingest::CollisionPolicy::Suffix;
                    if ui.checkbox(&mut suffix, "Add -1, -2… when a name is taken (never overwrite)").changed() {
                        app.ingest.opts.on_collision = if suffix { apature_ingest::CollisionPolicy::Suffix } else { apature_ingest::CollisionPolicy::Refuse };
                        opts_changed = true;
                    }
                    if ui.checkbox(&mut app.prefs.eject_after_ingest, "Eject the card when every file has verified").changed() {
                        app.prefs_dirty = true;
                    }
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("FILES").monospace().size(9.5).color(t.muted));
                        for (f, label) in [(FileFilter::All, "all"), (FileFilter::RawOnly, "RAW only"), (FileFilter::JpegOnly, "JPEG only")] {
                            if ui.selectable_label(app.ingest.opts.filter == f, label).clicked() {
                                app.ingest.opts.filter = f;
                                opts_changed = true;
                            }
                        }
                    });
                });

                // ---- Destinations ----
                let ui = &mut cols[1];
                egui::Frame::NONE.fill(t.panel).stroke(egui::Stroke::new(1.0, t.line)).corner_radius(CornerRadius::same(8)).inner_margin(egui::Margin::same(S4 as i8)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    super::eyebrow(ui, t, "Destinations");
                    ui.add_space(S2);
                    labeled(ui, t, "Primary", |ui| {
                        ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Choose…").clicked() {
                                cmds.push(Command::IngestChoosePrimary);
                            }
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                let p = app.ingest.opts.primary.display().to_string();
                                ui.add(egui::Label::new(RichText::new(if p.is_empty() { "choose a folder".into() } else { p }).monospace().size(10.5).color(t.text2)).truncate());
                                });
                            });
                        });
                    });
                    labeled(ui, t, "Secondary (backup)", |ui| {
                        ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if app.ingest.opts.secondary.is_some() && ui.button("×").on_hover_text("No secondary").clicked() {
                                app.ingest.opts.secondary = None;
                                opts_changed = true;
                            }
                            if ui.button("Choose…").clicked() {
                                cmds.push(Command::IngestChooseSecondary);
                            }
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                let p = app.ingest.opts.secondary.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "none".into());
                                ui.add(egui::Label::new(RichText::new(p).monospace().size(10.5).color(t.text2)).truncate());
                                });
                            });
                        });
                    });
                    labeled(ui, t, "Folder template", |ui| {
                        if ui.add(egui::TextEdit::singleline(&mut app.ingest.opts.folder_template).desired_width(f32::INFINITY).hint_text("{date}-{headline}  or  {year}/{month}/{day}  (empty = none)")).changed() {
                            opts_changed = true;
                        }
                    });
                    labeled(ui, t, "Rename template", |ui| {
                        if ui.add(egui::TextEdit::singleline(&mut app.ingest.opts.rename_template).desired_width(f32::INFINITY).hint_text("{headline}-{seqn}  (empty = keep names)")).changed() {
                            opts_changed = true;
                        }
                    });
                    labeled(ui, t, "Headline / slug", |ui| {
                            let mut h = app.ingest.opts.headline.clone().unwrap_or_default();
                            if ui.add(egui::TextEdit::singleline(&mut h).desired_width(f32::INFINITY)).changed() {
                                app.ingest.opts.headline = if h.trim().is_empty() { None } else { Some(h) };
                                opts_changed = true;
                            }
                        });
                    labeled(ui, t, "Seq. start", |ui| {
                            if ui.add(egui::DragValue::new(&mut app.ingest.opts.seq_start).range(1..=99999)).changed() {
                                opts_changed = true;
                            }
                        });
                    if app.assignment.is_some() {
                        ui.checkbox(&mut app.ingest.apply_pad, "Apply the loaded assignment pad to the copies");
                    }
                    ui.add(egui::Label::new(RichText::new("tokens: {filename} {ext} {frame} {seqn} {seqn:4} {headline} {date} {yymmdd} {year} {month} {day} {time}").monospace().size(9.5).color(t.muted)).wrap());
                });
            });
        });

        ui.add_space(S3);
        // ---- Plan ----
        egui::Frame::NONE.fill(t.panel).stroke(egui::Stroke::new(1.0, t.line)).corner_radius(CornerRadius::same(8)).inner_margin(egui::Margin::same(S4 as i8)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            super::eyebrow(ui, t, "Plan");
            ui.add_space(S2);
            if app.ingest.planning {
                ui.label(RichText::new("reading the card…").size(12.0).color(t.muted));
            }
            if let Some(err) = &app.ingest.plan_error {
                ui.label(RichText::new(err).size(12.0).color(t.reject));
            }
            match &app.ingest.plan {
                None if !app.ingest.planning && app.ingest.plan_error.is_none() => {
                    ui.label(RichText::new("Choose a source and a primary destination.").size(12.0).color(t.muted));
                }
                Some(plan) => {
                    // The plan card fades in when the plan lands (160 ms).
                    let k = super::motion::progress(app.ingest.plan_since, app.anim(0.16));
                    ui.set_opacity(k);
                    let dests = 1 + app.ingest.opts.secondary.is_some() as usize;
                    let mut summary = format!("{} files in {} photos · {} · {} destination{}", plan.entries.len(), plan.items, human_bytes(plan.total_bytes), dests, if dests == 1 { "" } else { "s" });
                    if !plan.skipped.is_empty() {
                        summary.push_str(&format!(" · {} skipped", plan.skipped.len()));
                    }
                    if plan.suffixed > 0 {
                        summary.push_str(&format!(" · {} renamed with a suffix", plan.suffixed));
                    }
                    ui.label(RichText::new(summary).size(13.0).color(t.text));
                    if let Some(first) = plan.entries.first() {
                        ui.label(RichText::new(format!("first copy: {}", first.primary.display())).monospace().size(10.5).color(t.text2));
                    }
                    for c in plan.collisions.iter().take(8) {
                        ui.label(RichText::new(format!("✕ {} — {}", c.destination.display(), c.reason)).monospace().size(10.5).color(t.reject));
                    }
                    if plan.collisions.len() > 8 {
                        ui.label(RichText::new(format!("… and {} more collisions", plan.collisions.len() - 8)).size(10.5).color(t.reject));
                    }
                    ui.add_space(S2);
                    ui.horizontal(|ui| {
                        let can = plan.can_run() && !running;
                        let start = ui.add_enabled(can, egui::Button::new(RichText::new("Start ingest").strong())).on_disabled_hover_text(if plan.collisions.is_empty() { "Nothing to copy" } else { "Resolve the collisions first" });
                        if start.clicked() {
                            cmds.push(Command::IngestStart);
                        }
                        if running && ui.button("Cancel after this file").clicked() {
                            cmds.push(Command::IngestCancel);
                        }
                    });
                }
                _ => {}
            }
        });

        // ---- Progress ----
        if let (Some(r), Some(plan)) = (&app.ingest.running, &app.ingest.plan) {
            ui.add_space(S3);
            // Finished: the card's border flashes accent and settles (600 ms).
            let flash = 1.0 - super::motion::progress(r.done_since, app.anim(0.6));
            let border = super::motion::lerp_color(t.line, t.accent, flash);
            egui::Frame::NONE.fill(t.panel).stroke(egui::Stroke::new(1.0, border)).corner_radius(CornerRadius::same(8)).inner_margin(egui::Margin::same(S4 as i8)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                super::eyebrow(ui, t, "Progress");
                let total = plan.entries.len().max(1);
                let done = r.statuses.iter().filter(|s| matches!(s, FileStatus::Verified | FileStatus::Failed)).count();
                // The bar follows bytes, not files, so one big NEF does not stall it.
                let dests = 1 + plan.entries.first().is_some_and(|e| e.secondary.is_some()) as u64;
                let frac = if r.done.is_some() {
                    1.0
                } else {
                    (r.bytes_done as f64 / (plan.total_bytes * dests).max(1) as f64) as f32
                };
                let frac = ui.ctx().animate_value_with_time(egui::Id::new("ingest_progress"), frac, app.anim(0.16));
                let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 6.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, CornerRadius::same(3), t.raised);
                ui.painter().rect_filled(egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * frac, 6.0)), CornerRadius::same(3), t.accent);
                let failed = r.statuses.iter().filter(|s| **s == FileStatus::Failed).count();
                ui.label(RichText::new(format!("{done} of {total} files · {failed} failed")).size(12.0).color(t.text2));
                if let Some(rep) = &r.done {
                    let ok = rep.all_verified(plan);
                    let msg = if rep.cancelled {
                        format!("Cancelled after {} files.", rep.outcomes.len())
                    } else if ok {
                        format!("Every file verified at every destination · {} in {:.0} s", human_bytes(rep.bytes_copied), rep.elapsed_secs)
                    } else {
                        format!("{} file{} failed. The card has not been changed; fix the destination and ingest again (only new photos will copy).", rep.files_failed(), if rep.files_failed() == 1 { "" } else { "s" })
                    };
                    ui.label(RichText::new(msg).size(13.0).color(if ok { t.text } else { t.reject }));
                    if let Some((files, failed)) = r.pad {
                        ui.label(RichText::new(format!("assignment pad applied → {files} files{}", if failed > 0 { format!(", {failed} failed") } else { String::new() })).size(11.0).color(t.text2));
                    }
                    ui.horizontal(|ui| {
                        if ok && ui.button("Open the ingested folder").clicked() {
                            cmds.push(Command::IngestOpenResult);
                        }
                        if ok && app.ingest.opts.source.starts_with("/Volumes") && ui.button("Eject the card").clicked() {
                            cmds.push(Command::IngestEject);
                        }
                    });
                }
                ui.add_space(S2);
                egui::ScrollArea::vertical().max_height(260.0).auto_shrink([false, true]).show(ui, |ui| {
                    let green = Color32::from_rgb(0x5F, 0xBF, 0x8A);
                    for (i, e) in plan.entries.iter().enumerate() {
                        // Rows arrive staggered when the run starts; the ✓ fades
                        // from the copying colour to green when a file verifies.
                        let row_k = super::motion::stagger(Some(r.started), i, 0.015, app.anim(0.16), 20);
                        let ok = ui.ctx().animate_bool_with_time(
                            egui::Id::new(("ingest-ok", i)),
                            r.statuses[i] == FileStatus::Verified,
                            app.anim(0.16),
                        );
                        let (glyph, color) = match r.statuses[i] {
                            FileStatus::Pending => ("·", t.muted),
                            FileStatus::Copying => ("→", t.accent),
                            FileStatus::Verified => ("✓", super::motion::lerp_color(t.accent, green, ok)),
                            FileStatus::Failed => ("✕", t.reject),
                        };
                        let color = color.gamma_multiply(row_k);
                        let name = e.source.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                        let line = match &r.errors[i] {
                            Some(err) => format!("{glyph} {name}   {err}"),
                            None => format!("{glyph} {name}"),
                        };
                        ui.label(RichText::new(line).monospace().size(10.5).color(color));
                    }
                });
            });
        }
    });

    if opts_changed {
        app.ingest_options_changed();
    }
    for c in cmds {
        app.dispatch(ui.ctx(), c);
    }
}
