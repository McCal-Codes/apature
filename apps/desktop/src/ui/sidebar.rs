use std::path::PathBuf;

use egui::{Align2, FontId, RichText, Sense};

use super::tokens::{Theme, ROW_H, S1, S2, S3, SIDEBAR_RANGE};
use crate::app::App;
use crate::commands::Command;

fn row(
    ui: &mut egui::Ui,
    t: &Theme,
    label: &str,
    count: Option<usize>,
    selected: bool,
) -> egui::Response {
    row_glyph(ui, t, label, count, selected, None)
}

fn row_glyph(
    ui: &mut egui::Ui,
    t: &Theme,
    label: &str,
    count: Option<usize>,
    selected: bool,
    volume: Option<crate::platform::VolumeKind>,
) -> egui::Response {
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), ROW_H), Sense::click());
    let p = ui.painter();
    super::icons::focus_ring(p, rect, t, &resp);
    if selected {
        p.rect_filled(rect, egui::CornerRadius::same(6), t.accent_wash);
    } else if resp.hovered() {
        p.rect_filled(rect, egui::CornerRadius::same(6), t.raised);
    }
    let fg = if selected { t.accent } else { t.text2 };
    let glyph = egui::Rect::from_center_size(
        egui::pos2(rect.left() + S2 + 6.0, rect.center().y),
        egui::vec2(12.0, 12.0),
    );
    match volume {
        Some(kind) => super::icons::volume_glyph(p, glyph.center(), kind, fg),
        None => {
            p.rect_stroke(
                glyph,
                egui::CornerRadius::same(3),
                egui::Stroke::new(1.5, fg),
                egui::StrokeKind::Inside,
            );
        }
    }
    let text_x = rect.left() + S2 + 12.0 + S2;
    let mut max_w = rect.right() - S2 - text_x;
    if let Some(n) = count {
        let cs = p.text(
            egui::pos2(rect.right() - S2, rect.center().y),
            Align2::RIGHT_CENTER,
            n.to_string(),
            FontId::monospace(10.0),
            t.muted,
        );
        max_w -= cs.width() + S2;
    }
    let galley = ui.fonts_mut(|f| {
        f.layout(
            label.to_string(),
            FontId::proportional(13.0),
            fg,
            max_w.max(20.0),
        )
    });
    let galley = if galley.rows.len() > 1 {
        ui.fonts_mut(|f| {
            f.layout_no_wrap(
                truncate(label, (max_w / 7.0) as usize),
                FontId::proportional(13.0),
                fg,
            )
        })
    } else {
        galley
    };
    p.galley(
        egui::pos2(text_x, rect.center().y - galley.size().y / 2.0),
        galley,
        fg,
    );
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    resp.on_hover_text(label)
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n.max(4) {
        s.to_string()
    } else {
        format!(
            "{}…",
            s.chars()
                .take(n.saturating_sub(1).max(3))
                .collect::<String>()
        )
    }
}

fn name_of(p: &std::path::Path) -> String {
    p.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.display().to_string())
}

pub fn show(app: &mut App, ctx: &egui::Context, t: &Theme) {
    let mut cmds: Vec<Command> = Vec::new();
    let current = app.folder.clone();
    let resp = egui::SidePanel::left("sidebar")
        .resizable(true)
        .default_width(app.prefs.sidebar_width)
        .width_range(SIDEBAR_RANGE)
        .frame(
            egui::Frame::NONE
                .fill(t.panel)
                .inner_margin(egui::Margin::symmetric(S2 as i8, S3 as i8))
                .stroke(egui::Stroke::NONE),
        )
        .show_animated(ctx, app.prefs.sidebar_visible, |ui| {
            ui.spacing_mut().item_spacing.y = S1;
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    super::eyebrow(ui, t, "Favorites");
                    if app.prefs.favorites.is_empty() {
                        ui.label(
                            RichText::new(
                                "Add the open folder with File › Add Folder to Favorites",
                            )
                            .size(11.0)
                            .color(t.muted),
                        );
                    }
                    for f in app.prefs.favorites.clone() {
                        let r = row(
                            ui,
                            t,
                            &name_of(&f),
                            None,
                            current.as_deref() == Some(f.as_path()),
                        );
                        if r.clicked() {
                            cmds.push(Command::OpenFolder(f.clone()));
                        }
                        r.context_menu(|ui| {
                            if ui.button("Remove from Favorites").clicked() {
                                cmds.push(Command::RemoveFavorite(f.clone()));
                                ui.close();
                            }
                        });
                    }
                    ui.add_space(S3);
                    super::eyebrow(ui, t, "Recent");
                    for f in app.prefs.recent.clone() {
                        if row(
                            ui,
                            t,
                            &name_of(&f),
                            None,
                            current.as_deref() == Some(f.as_path()),
                        )
                        .clicked()
                        {
                            cmds.push(Command::OpenFolder(f));
                        }
                    }
                    ui.add_space(S3);
                    super::eyebrow(ui, t, "Drives & cards");
                    for (name, path) in app.volumes.clone() {
                        let kind = app.volume_kinds.get(&path).copied();
                        if row_glyph(ui, t, &name, None, false, kind).clicked() {
                            cmds.push(Command::OpenFolder(path));
                        }
                    }
                    ui.add_space(S3);
                    super::eyebrow(ui, t, "Collections");
                    ui.label(
                        RichText::new("Named contact sheets arrive in M3")
                            .size(11.0)
                            .color(t.muted),
                    );
                });
        });
    let Some(resp) = resp else {
        for c in cmds {
            app.dispatch(ctx, c);
        }
        return;
    };
    let w = resp.response.rect.width();
    if app.prefs.sidebar_visible
        && (w - app.prefs.sidebar_width).abs() > 0.5
        && SIDEBAR_RANGE.contains(&w)
    {
        app.prefs.sidebar_width = w;
        app.prefs_dirty = true;
    }
    // Divider.
    let r = resp.response.rect;
    ctx.layer_painter(egui::LayerId::background()).line_segment(
        [r.right_top(), r.right_bottom()],
        egui::Stroke::new(1.0, t.line),
    );
    for c in cmds {
        app.dispatch(ctx, c);
    }
    let _ = PathBuf::new();
}
