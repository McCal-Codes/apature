use egui::{Align, Layout, RichText};

use super::tokens::{Theme, S3, S4, TOOLBAR_H};
use crate::app::App;
use crate::commands::{Command, Workspace};

pub fn show(app: &mut App, ctx: &egui::Context, t: &Theme) {
    let mut cmds = Vec::new();
    egui::TopBottomPanel::top("toolbar")
        .exact_height(TOOLBAR_H)
        .frame(
            egui::Frame::NONE
                .fill(t.panel)
                .inner_margin(egui::Margin::symmetric(S4 as i8, 0))
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = S3;
                let sb = ui
                    .add(
                        egui::Button::new(RichText::new("⊞").size(15.0))
                            .selected(app.prefs.sidebar_visible),
                    )
                    .on_hover_text("Show or hide the sidebar (⌃⌘S)");
                if sb.clicked() {
                    cmds.push(Command::ToggleSidebar);
                }
                // Workspace switcher.
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    for w in [
                        Workspace::Sheet,
                        Workspace::Preview,
                        Workspace::Caption,
                        Workspace::Ingest,
                    ] {
                        let selected = app.workspace == w;
                        match w.available() {
                            Ok(()) => {
                                if ui
                                    .add(egui::Button::new(w.label()).selected(selected))
                                    .clicked()
                                {
                                    cmds.push(Command::SetWorkspace(w));
                                }
                            }
                            Err(why) => {
                                ui.add_enabled(false, egui::Button::new(w.label()))
                                    .on_disabled_hover_text(why);
                            }
                        }
                    }
                });
                ui.add_enabled(false, egui::Button::new("Filter"))
                    .on_disabled_hover_text("Filtering by rating, class and tag arrives in M3");
                ui.add_enabled(false, egui::Button::new("Sort: name ▾"))
                    .on_disabled_hover_text("Sort by capture time arrives in M3");

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = S3;
                    let ins = ui
                        .add(
                            egui::Button::new(RichText::new("ⓘ").size(15.0))
                                .selected(app.prefs.inspector_visible),
                        )
                        .on_hover_text("Show or hide the inspector (⌥⌘I)");
                    if ins.clicked() {
                        cmds.push(Command::ToggleInspector);
                    }
                    let pending = app.pool.pending();
                    if pending > 0 {
                        ui.label(super::mono(format!("{pending} decoding"), 11.0, t.muted));
                    }
                    let mut cell = app.prefs.cell;
                    let slider = ui
                        .add(egui::Slider::new(&mut cell, 80.0..=600.0).show_value(false))
                        .on_hover_text("Thumbnail size (⌘= / ⌘-)");
                    if slider.changed() {
                        app.set_cell(cell);
                    }
                });
            });
        });
    // A 1 pt line under the toolbar, drawn by the next panel's top stroke.
    for c in cmds {
        app.dispatch(ctx, c);
    }
}
