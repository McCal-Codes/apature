//! Export workspace: which photos, where, how big, then a list that never
//! hides a failure. The desk gets JPEGs with EXIF, XMP and IPTC-IIM.
//!
//! Laid out the way Photo Mechanic's Save Photos As dialog is read, top to
//! bottom: the preset, which photos, where they go and what they are called,
//! then the image (format, size, quality), then the metadata, then where to
//! send them. One card per group; the Export button stays at the bottom.

use egui::{Color32, CornerRadius, RichText};

use super::tokens::{Theme, S1, S2, S3, S4};
use crate::app::App;
use crate::commands::Command;
use crate::export_job::Status;

/// A small uppercase label above a control inside a card.
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

/// A row with a label, a one-line description and a switch on the right,
/// the same shape Settings uses.
fn switch_row(ui: &mut egui::Ui, t: &Theme, label: &str, what: &str, on: &mut bool) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_max_width((ui.available_width() - 60.0).max(120.0));
            ui.label(RichText::new(label).size(12.5).color(t.text));
            if !what.is_empty() {
                ui.add(egui::Label::new(RichText::new(what).size(10.5).color(t.muted)).wrap());
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            changed = super::settings::toggle(ui, t, on);
        });
    });
    ui.add_space(S1);
    changed
}

pub fn show(app: &mut App, ui: &mut egui::Ui, t: &Theme) {
    let mut cmds: Vec<Command> = Vec::new();
    let mut prefs_changed = false;
    let running = app.export.running.as_ref().is_some_and(|r| !r.finished);
    let selected = app.selection.targets(app.cursor).len();
    let in_view = app.view.len();
    let green = Color32::from_rgb(0x5F, 0xBF, 0x8A);
    let n = if app.export.scope_all {
        in_view
    } else {
        selected
    };
    let can = app.prefs.export.dest.is_some() && !running && n > 0;

    // The action bar is pinned; everything else scrolls above it.
    egui::TopBottomPanel::bottom("export_actions")
        .frame(
            egui::Frame::NONE
                .fill(t.panel)
                .inner_margin(egui::Margin::symmetric(S4 as i8, S3 as i8)),
        )
        .show_inside(ui, |ui| {
            let r = ui.available_rect_before_wrap();
            ui.painter()
                .hline(r.x_range(), r.top() - S3, egui::Stroke::new(1.0, t.line));
            ui.horizontal(|ui| {
                let label = format!("Export {n} photo{}", if n == 1 { "" } else { "s" });
                let b = ui
                    .add_enabled(
                        can,
                        egui::Button::new(RichText::new(label).strong()).fill(if can {
                            t.accent_wash
                        } else {
                            t.raised
                        }),
                    )
                    .on_disabled_hover_text(if app.prefs.export.dest.is_none() {
                        "Choose a destination folder"
                    } else {
                        "Nothing to export"
                    });
                if b.clicked() {
                    cmds.push(Command::ExportStart);
                }
                if running && ui.button("Cancel after the current files").clicked() {
                    cmds.push(Command::ExportCancel);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let dest = app
                        .prefs
                        .export
                        .dest
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "no destination yet".into());
                    ui.add(
                        egui::Label::new(RichText::new(dest).monospace().size(10.5).color(t.muted))
                            .truncate(),
                    );
                });
            });
        });

    egui::CentralPanel::default()
        .frame(
            egui::Frame::NONE
                .fill(t.photo)
                .inner_margin(egui::Margin::symmetric(S4 as i8, S2 as i8)),
        )
        .show_inside(ui, |ui| {
            ui.spacing_mut().item_spacing.y = S2;
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_enabled_ui(!running, |ui| {
                        ui.columns(2, |cols| {
                            left_column(
                                app,
                                &mut cols[0],
                                t,
                                &mut cmds,
                                &mut prefs_changed,
                                selected,
                                in_view,
                            );
                            right_column(app, &mut cols[1], t, &mut cmds, &mut prefs_changed);
                        });
                    });
                    progress(app, ui, t, &mut cmds, green);
                    ui.add_space(S4);
                });
        });

    if prefs_changed {
        app.prefs_dirty = true;
    }
    for c in cmds {
        app.dispatch(ui.ctx(), c);
    }
}

fn left_column(
    app: &mut App,
    ui: &mut egui::Ui,
    t: &Theme,
    cmds: &mut Vec<Command>,
    prefs_changed: &mut bool,
    selected: usize,
    in_view: usize,
) {
    // ---- Preset ----
    super::card(ui, t, "Preset", |ui| {
        ui.horizontal_wrapped(|ui| {
            let names: Vec<String> = app.prefs.export_presets.iter().map(|p| p.name.clone()).collect();
            let active = app.prefs.export_preset.clone();
            for n in &names {
                let on = active.as_deref() == Some(n.as_str());
                let matches = on && app.prefs.export_matches_preset(n);
                let label = if on && !matches { format!("{n} · edited") } else { n.clone() };
                if super::icons::chip(ui, t, &label, on).clicked() {
                    app.prefs.apply_export_preset(n);
                    *prefs_changed = true;
                }
                if on && !matches && ui.small_button("Update").on_hover_text("Save the current settings into this preset").clicked() {
                    let e = app.prefs.export.clone();
                    let s = app.prefs.send_after_export.clone();
                    if let Some(p) = app.prefs.export_presets.iter_mut().find(|p| &p.name == n) {
                        p.export = e;
                        p.send_to = s;
                    }
                    *prefs_changed = true;
                }
                if on && ui.small_button("Delete").clicked() {
                    app.prefs.export_presets.retain(|p| &p.name != n);
                    app.prefs.export_preset = None;
                    *prefs_changed = true;
                }
            }
            if names.is_empty() {
                ui.label(RichText::new("One per outlet: size, quality, naming, metadata, format and where it goes.").size(11.0).color(t.muted));
            }
        });
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut app.export.preset_draft)
                    .desired_width(160.0)
                    .hint_text("Daily Record"),
            );
            let name = app.export.preset_draft.trim().to_string();
            if !name.is_empty() && ui.small_button("Save as preset").clicked() {
                app.prefs.export_presets.retain(|p| p.name != name);
                app.prefs.export_presets.push(crate::prefs::ExportPreset {
                    name: name.clone(),
                    export: app.prefs.export.clone(),
                    send_to: app.prefs.send_after_export.clone(),
                });
                app.prefs.export_preset = Some(name);
                app.export.preset_draft.clear();
                *prefs_changed = true;
            }
        });
    });

    // ---- Photos ----
    super::card(ui, t, "Photos", |ui| {
        ui.horizontal(|ui| {
            if super::icons::chip(
                ui,
                t,
                &format!("Selected ({selected})"),
                !app.export.scope_all,
            )
            .clicked()
            {
                app.export.scope_all = false;
            }
            if super::icons::chip(
                ui,
                t,
                &format!("All in view ({in_view})"),
                app.export.scope_all,
            )
            .clicked()
            {
                app.export.scope_all = true;
            }
        });
    });

    // ---- Destination and names ----
    super::card(ui, t, "Destination", |ui| {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Choose…").clicked() {
                    cmds.push(Command::ExportChooseDest);
                }
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let p = app
                        .prefs
                        .export
                        .dest
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default();
                    ui.add(
                        egui::Label::new(
                            RichText::new(if p.is_empty() {
                                "choose a folder".into()
                            } else {
                                p
                            })
                            .monospace()
                            .size(10.5)
                            .color(t.text2),
                        )
                        .truncate(),
                    );
                });
            });
        });
        ui.add_space(S2);
        labeled(ui, t, "Naming", |ui| {
            if ui
                .add(
                    egui::TextEdit::singleline(&mut app.prefs.export.rename)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("{filename}  ·  {headline}-{seqn:3}  ·  empty keeps names"),
                )
                .changed()
            {
                *prefs_changed = true;
            }
        });
        let names = app.export_names();
        if names.is_empty() {
            let slug = app.assignment.as_ref().map(|a| a.slug.clone());
            ui.label(
                RichText::new(match &slug {
                    Some(s) => format!("headline from the loaded assignment: {s}"),
                    None => "no assignment loaded: {headline} expands to nothing".into(),
                })
                .size(10.5)
                .color(t.muted),
            );
        } else {
            ui.horizontal(|ui| {
                ui.label(RichText::new("E.G.").monospace().size(9.5).color(t.muted));
                let shown: Vec<String> = names
                    .iter()
                    .take(2)
                    .map(|(_, p)| {
                        p.file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default()
                    })
                    .collect();
                ui.label(
                    RichText::new(shown.join("   "))
                        .monospace()
                        .size(10.5)
                        .color(t.text2),
                );
                if names.len() > 2 {
                    ui.label(
                        RichText::new(format!("+{}", names.len() - 2))
                            .size(10.5)
                            .color(t.muted),
                    );
                }
            });
        }
    });
}

fn right_column(
    app: &mut App,
    ui: &mut egui::Ui,
    t: &Theme,
    cmds: &mut Vec<Command>,
    prefs_changed: &mut bool,
) {
    // ---- Image ----
    super::card(ui, t, "Image", |ui| {
        labeled(ui, t, "Format", |ui| {
            let cjxl = apature_export::cjxl_path();
            ui.horizontal(|ui| {
                if super::icons::chip(ui, t, "JPEG", !app.prefs.export.jxl).clicked() {
                    app.prefs.export.jxl = false;
                    *prefs_changed = true;
                }
                let r = super::icons::chip(ui, t, "JPEG XL", app.prefs.export.jxl);
                if r.clicked() && cjxl.is_some() {
                    app.prefs.export.jxl = true;
                    *prefs_changed = true;
                }
                if cjxl.is_none() {
                    ui.label(
                        RichText::new("JPEG XL needs cjxl")
                            .size(10.5)
                            .color(t.muted),
                    )
                    .on_hover_text("brew install jpeg-xl");
                }
            });
            if app.prefs.export.jxl {
                ui.label(RichText::new("The finished JPEG is transcoded losslessly to .jxl, metadata kept, about a fifth smaller. Desks must accept JPEG XL.").size(10.5).color(t.muted));
            }
        });
        labeled(ui, t, "Long side", |ui| {
            ui.horizontal(|ui| {
                for (px, label) in [
                    (2048, "2048"),
                    (3000, "3000"),
                    (4096, "4096"),
                    (0, "Full size"),
                ] {
                    if super::icons::chip(ui, t, label, app.prefs.export.long_side == px).clicked()
                    {
                        app.prefs.export.long_side = px;
                        *prefs_changed = true;
                    }
                }
                let mut v = app.prefs.export.long_side;
                if ui
                    .add(egui::DragValue::new(&mut v).range(0..=20000).suffix(" px"))
                    .changed()
                {
                    app.prefs.export.long_side = v;
                    *prefs_changed = true;
                }
            });
        });
        labeled(ui, t, "JPEG quality", |ui| {
            let mut q = app.prefs.export.quality;
            if ui.add(egui::Slider::new(&mut q, 60..=100)).changed() {
                app.prefs.export.quality = q;
                *prefs_changed = true;
            }
        });
        if switch_row(ui, t, "Decode RAW-only photos", "With the macOS RAW engine, slower, sRGB. Off: a RAW alone uses its embedded full-size preview. Pairs export their JPEG.", &mut app.prefs.export.decode_raw) {
            *prefs_changed = true;
        }
    });

    // ---- Metadata ----
    super::card(ui, t, "Metadata", |ui| {
        if switch_row(
            ui,
            t,
            "Keep EXIF",
            "Camera, lens, capture time, exposure.",
            &mut app.prefs.export.keep_exif,
        ) {
            *prefs_changed = true;
        }
        if switch_row(
            ui,
            t,
            "Write IPTC-IIM as well as XMP",
            "For desks on older systems that read only the legacy block.",
            &mut app.prefs.export.write_iim,
        ) {
            *prefs_changed = true;
        }
        ui.label(RichText::new("Captions, keywords, credit and the rest always go as XMP. A JPEG's own colour profile is kept.").size(10.5).color(t.muted));
    });

    // ---- Then ----
    super::card(ui, t, "Then", |ui| {
        if switch_row(
            ui,
            t,
            "Open the folder when the export finishes",
            "",
            &mut app.prefs.export.open_after,
        ) {
            *prefs_changed = true;
        }
        labeled(ui, t, "Send to", |ui| {
            if app.prefs.delivery.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("No delivery targets yet.")
                            .size(11.0)
                            .color(t.muted),
                    );
                    if ui.small_button("Settings › Delivery").clicked() {
                        app.settings_page = super::settings::Page::Delivery;
                        app.settings_open = true;
                    }
                });
            } else {
                ui.horizontal_wrapped(|ui| {
                    if super::icons::chip(ui, t, "nowhere", app.prefs.send_after_export.is_none())
                        .clicked()
                    {
                        app.prefs.send_after_export = None;
                        *prefs_changed = true;
                    }
                    let names: Vec<String> =
                        app.prefs.delivery.iter().map(|d| d.name.clone()).collect();
                    for n in names {
                        if super::icons::chip(
                            ui,
                            t,
                            &n,
                            app.prefs.send_after_export.as_deref() == Some(n.as_str()),
                        )
                        .clicked()
                        {
                            app.prefs.send_after_export = Some(n.clone());
                            *prefs_changed = true;
                        }
                    }
                });
                let n_out = app.deliver.last_outputs.len();
                let busy = app
                    .deliver
                    .running
                    .as_ref()
                    .is_some_and(|r| r.done.is_none());
                if n_out > 0
                    && !busy
                    && ui
                        .small_button(format!("Send the last export ({n_out} files)"))
                        .on_hover_text("Uses the target chosen above, or the first one")
                        .clicked()
                {
                    app.deliver.pick = app.prefs.send_after_export.clone();
                    cmds.push(Command::DeliverLast);
                }
                if let Some(r) = &app.deliver.running {
                    let sent = r
                        .statuses
                        .iter()
                        .filter(|s| **s == super::super::deliver_job::Status::Sent)
                        .count();
                    let failed = r
                        .statuses
                        .iter()
                        .filter(|s| **s == super::super::deliver_job::Status::Failed)
                        .count();
                    let secs = r.started.elapsed().as_secs_f64().max(0.001);
                    let line = if r.done.is_some() {
                        format!(
                            "{sent} of {} sent to {} · {failed} failed",
                            r.files.len(),
                            r.target
                        )
                    } else {
                        format!(
                            "Sending to {} · {sent} of {} · {:.0} MB/s",
                            r.target,
                            r.files.len(),
                            r.bytes as f64 / secs / 1e6
                        )
                    };
                    ui.label(RichText::new(line).size(12.0).color(if failed > 0 {
                        t.reject
                    } else {
                        t.text2
                    }));
                    for e in r.errors.iter().flatten().take(3) {
                        ui.label(RichText::new(e).size(10.5).color(t.reject));
                    }
                    if r.done.is_some()
                        && failed > 0
                        && ui.small_button(format!("Resend {failed} failed")).clicked()
                    {
                        cmds.push(Command::DeliverResendFailed);
                    }
                }
            }
        });
    });
}

fn progress(app: &mut App, ui: &mut egui::Ui, t: &Theme, cmds: &mut Vec<Command>, green: Color32) {
    let Some(r) = &app.export.running else { return };
    ui.add_space(S2);
    let flash = 1.0 - super::motion::progress(r.done_since, app.anim(0.6));
    let border = super::motion::lerp_color(t.line, t.accent, flash);
    let names = app.export.names.clone();
    super::eyebrow(ui, t, "Progress");
    ui.add_space(S1);
    egui::Frame::NONE
        .fill(t.raised)
        .stroke(egui::Stroke::new(1.0, border))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(egui::Margin::same(S3 as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let total = r.statuses.len().max(1);
            let done = r.ok + r.failed;
            let frac = ui.ctx().animate_value_with_time(egui::Id::new("export_progress"), done as f32 / total as f32, app.anim(0.16));
            let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 6.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, CornerRadius::same(3), t.panel);
            ui.painter().rect_filled(egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * frac, 6.0)), CornerRadius::same(3), t.accent);
            ui.label(RichText::new(format!("{done} of {total} files · {} failed", r.failed)).size(12.0).color(t.text2));
            if r.finished {
                let msg = if r.failed == 0 {
                    format!("{} JPEG{} written · {:.1} MB in {:.1} s", r.ok, if r.ok == 1 { "" } else { "s" }, r.bytes as f64 / 1e6, r.done_since.map(|d| d.duration_since(r.started).as_secs_f32()).unwrap_or(0.0))
                } else {
                    format!("{} failed; nothing was overwritten. Fix the destination or the names and export again.", r.failed)
                };
                ui.label(RichText::new(msg).size(13.0).color(if r.failed == 0 { t.text } else { t.reject }));
                if ui.small_button("Open the exported folder").clicked() {
                    cmds.push(Command::ExportOpenResult);
                }
            }
            ui.add_space(S2);
            egui::ScrollArea::vertical().max_height(260.0).auto_shrink([false, true]).show(ui, |ui| {
                for (i, s) in r.statuses.iter().enumerate() {
                    let row_k = super::motion::stagger(Some(r.started), i, 0.015, app.anim(0.16), 20);
                    let ok = ui.ctx().animate_bool_with_time(egui::Id::new(("export-ok", i)), *s == Status::Done, app.anim(0.16));
                    let (glyph, color) = match s {
                        Status::Pending => ("·", t.muted),
                        Status::Working => ("→", t.accent),
                        Status::Done => ("✓", super::motion::lerp_color(t.accent, green, ok)),
                        Status::Failed => ("✕", t.reject),
                    };
                    let name = names.get(i).and_then(|(_, p)| p.file_name().map(|n| n.to_string_lossy().into_owned())).unwrap_or_default();
                    let line = match &r.errors[i] {
                        Some(err) => format!("{glyph} {name}   {err}"),
                        None => format!("{glyph} {name}"),
                    };
                    ui.label(RichText::new(line).monospace().size(10.5).color(color.gamma_multiply(row_k)));
                }
            });
        });
}
