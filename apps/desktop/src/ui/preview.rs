//! Preview workspace: one photo fitted or at 1:1 with drag/scroll panning,
//! or two to four photos compared side by side with zoom and pan in sync.
//!
//! "1:1" means one pixel of the decoded preview per physical screen pixel.
//! For a JPEG that is the file itself; for a RAW it is the camera's
//! full-size embedded preview, capped at the GPU's largest texture.

use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use super::tokens::{Theme, S3, S5};
use crate::app::{App, ZoomMode};
use crate::selection::Mods;

/// Largest preview we ask for at 1:1 (also bounded by the GPU).
const FULL_CAP: u32 = 16384;

pub fn show(app: &mut App, ui: &mut egui::Ui, t: &Theme) {
    let rect = ui.available_rect_before_wrap();
    let panes: Vec<usize> = match &app.pv.compare {
        Some(set) if set.len() >= 2 => set.clone(),
        _ => vec![app.cursor],
    };
    let compare = panes.len() > 1;
    let rects = split(rect, panes.len());

    ui.painter().rect_filled(rect, CornerRadius::ZERO, t.photo);
    let mut focus_click: Option<usize> = None;
    for (i, (&idx, &r)) in panes.iter().zip(rects.iter()).enumerate() {
        let r = if compare { r.shrink(1.0) } else { r };
        if draw_pane(app, ui, t, r, idx, i, compare) {
            focus_click = Some(idx);
        }
    }
    if let Some(idx) = focus_click {
        if idx != app.cursor {
            app.cursor = idx;
            app.selection.focus_moved(idx, Mods::default(), &app.view);
        }
    }

    if !compare {
        let ppp = ui.ctx().pixels_per_point();
        let fit_px = ((rect.width().max(rect.height())) * ppp).ceil() as u32;
        if let Some(n) = app.view_neighbor(1) {
            app.request_preview(n, fit_px, 1);
        }
        if let Some(p) = app.view_neighbor(-1) {
            app.request_preview(p, fit_px, 2);
        }
    }
    if app.cols > 0 {
        let row = app.cursor_pos() / app.cols;
        let rows = app.row_count();
        app.schedule_thumbs(row.saturating_sub(1)..(row + 2).min(rows));
    }
}

/// Split the viewport for `n` panes: side by side for two (or stacked when
/// the viewport is taller than wide), a 2×2 grid for three or four.
fn split(rect: Rect, n: usize) -> Vec<Rect> {
    match n {
        0 | 1 => vec![rect],
        2 => {
            if rect.width() >= rect.height() {
                let w = rect.width() / 2.0;
                vec![
                    Rect::from_min_size(rect.min, Vec2::new(w, rect.height())),
                    Rect::from_min_size(rect.min + Vec2::new(w, 0.0), Vec2::new(w, rect.height())),
                ]
            } else {
                let h = rect.height() / 2.0;
                vec![
                    Rect::from_min_size(rect.min, Vec2::new(rect.width(), h)),
                    Rect::from_min_size(rect.min + Vec2::new(0.0, h), Vec2::new(rect.width(), h)),
                ]
            }
        }
        _ => {
            let w = rect.width() / 2.0;
            let h = rect.height() / 2.0;
            (0..n.min(4))
                .map(|i| {
                    let (c, r) = ((i % 2) as f32, (i / 2) as f32);
                    Rect::from_min_size(rect.min + Vec2::new(c * w, r * h), Vec2::new(w, h))
                })
                .collect()
        }
    }
}

/// Draw one pane. Returns true when it was clicked (focus request).
fn draw_pane(
    app: &mut App,
    ui: &mut egui::Ui,
    t: &Theme,
    rect: Rect,
    idx: usize,
    pane_i: usize,
    compare: bool,
) -> bool {
    let ppp = ui.ctx().pixels_per_point();
    let max_tex = ui.ctx().input(|i| i.max_texture_side) as u32;
    let fit_px = ((rect.width().max(rect.height())) * ppp).ceil() as u32;
    let want = match app.pv.mode {
        ZoomMode::Fit => fit_px,
        ZoomMode::OneToOne => FULL_CAP.min(max_tex),
    };
    app.request_preview(idx, want, 0);

    let focused = idx == app.cursor;
    let resp = ui.interact(
        rect,
        egui::Id::new(("preview-pane", pane_i)),
        Sense::click_and_drag(),
    );
    let p = ui.painter().with_clip_rect(rect);

    let have = app
        .previews
        .get(&idx)
        .map(|(h, long, full)| (h.clone(), *long, *full));
    let (tex, is_full, preview_px) = match &have {
        Some((h, long, full)) => (Some(h.clone()), *full, Some(*long)),
        None => (app.thumbs[idx].clone(), false, None),
    };
    let mut zoom_label = String::from("loading…");
    let mut shown_rect: Option<Rect> = None;
    if let Some(tex) = tex {
        let size = tex.size_vec2();
        let inner = rect.shrink(S5);
        let s_fit = (inner.width() / size.x)
            .min(inner.height() / size.y)
            .min(1.0);
        let fitted = Rect::from_center_size(rect.center(), size * s_fit);

        // 1:1 — one texture pixel per physical pixel, panned by `center`.
        let one = {
            let size_pts = size / ppp;
            let c = app.pv.center;
            let mut r =
                Rect::from_center_size(rect.center() - (c - Vec2::splat(0.5)) * size_pts, size_pts);
            // Keep the image covering the pane; centre it on axes where it
            // is smaller than the pane. Write the clamped centre back so
            // panning stops at the edges.
            let mut c2 = c;
            if size_pts.x <= rect.width() {
                r = r.translate(Vec2::new(rect.center().x - r.center().x, 0.0));
                c2.x = 0.5;
            } else if r.min.x > rect.min.x {
                r = r.translate(Vec2::new(rect.min.x - r.min.x, 0.0));
            } else if r.max.x < rect.max.x {
                r = r.translate(Vec2::new(rect.max.x - r.max.x, 0.0));
            }
            if size_pts.y <= rect.height() {
                r = r.translate(Vec2::new(0.0, rect.center().y - r.center().y));
                c2.y = 0.5;
            } else if r.min.y > rect.min.y {
                r = r.translate(Vec2::new(0.0, rect.min.y - r.min.y));
            } else if r.max.y < rect.max.y {
                r = r.translate(Vec2::new(0.0, rect.max.y - r.max.y));
            }
            if size_pts.x > rect.width() {
                c2.x = (rect.center().x - r.min.x) / size_pts.x;
            }
            if size_pts.y > rect.height() {
                c2.y = (rect.center().y - r.min.y) / size_pts.y;
            }
            if focused || !compare {
                app.pv.center = c2;
            }
            r
        };

        let target = match app.pv.mode {
            ZoomMode::Fit => fitted,
            ZoomMode::OneToOne => one,
        };
        // Zoom toggle eases between fit and 1:1 (160 ms); entering the
        // workspace grows out of the sheet cell.
        let k_zoom = super::motion::progress(app.pv.zoom_since, app.anim(0.16));
        let k_enter = super::motion::progress(app.preview_since, app.anim(0.16));
        let shown = if k_zoom < 1.0 && (focused || !compare) {
            match app.pv.zoom_from {
                Some(from) => super::motion::lerp_rect(from, target, k_zoom),
                None => target,
            }
        } else if k_enter < 1.0 && !compare {
            match app.preview_from {
                Some(from) => {
                    let fs = (from.width() / size.x).min(from.height() / size.y);
                    let from_fit = Rect::from_center_size(from.center(), size * fs);
                    super::motion::lerp_rect(from_fit, target, k_enter)
                }
                None => target,
            }
        } else {
            target
        };
        p.image(
            tex.id(),
            shown,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        shown_rect = Some(shown);
        zoom_label = match (app.pv.mode, preview_px) {
            (ZoomMode::Fit, _) if compare => "fit".into(),
            (ZoomMode::Fit, Some(px)) => format!("fit · {px} px preview"),
            (ZoomMode::Fit, None) => "fit · thumbnail, full preview loading".into(),
            (ZoomMode::OneToOne, Some(_)) if is_full && compare => "1:1".into(),
            (ZoomMode::OneToOne, Some(px)) if is_full => format!("1:1 · {px} px"),
            (ZoomMode::OneToOne, Some(_)) if compare => "1:1 · loading…".into(),
            (ZoomMode::OneToOne, Some(px)) => format!("1:1 of {px} px · full resolution loading…"),
            (ZoomMode::OneToOne, None) => "1:1 · loading…".into(),
        };

        // ---- interaction ----
        let size_pts = size / ppp;
        if app.pv.mode == ZoomMode::OneToOne && (focused || !compare) {
            if resp.dragged() {
                let d = resp.drag_delta();
                app.pv.center -= Vec2::new(d.x / size_pts.x, d.y / size_pts.y);
            }
            if resp.hovered() {
                let scroll = ui.input(|i| i.raw_scroll_delta);
                if scroll != Vec2::ZERO {
                    app.pv.center -= Vec2::new(scroll.x / size_pts.x, scroll.y / size_pts.y);
                }
            }
        }
        if resp.double_clicked() {
            let at = resp
                .interact_pointer_pos()
                .map(|pos| {
                    Vec2::new(
                        (pos.x - shown.min.x) / shown.width(),
                        (pos.y - shown.min.y) / shown.height(),
                    )
                })
                .unwrap_or(Vec2::splat(0.5));
            app.toggle_zoom(Some(at), Some(shown));
        }
    }
    if focused || !compare {
        app.pv.last_rect = shown_rect;
    }

    // ---- overlay ----
    let m = app.marks[idx];
    let line1 = format!(
        "{}   {} / {}",
        app.items[idx].display_name(),
        app.pos[idx].map(|p| p + 1).unwrap_or(idx + 1),
        app.view.len()
    );
    let mut line2 = Vec::new();
    if let Some(e) = &app.exif[idx] {
        let ex = e.exposure_line();
        if !ex.is_empty() {
            line2.push(ex);
        }
    }
    let stars = super::rating_text(m.rating);
    if !stars.is_empty() {
        line2.push(stars);
    }
    if let Some(n) = app.scheme.name(m.color_class) {
        line2.push(format!("{} {n}", m.color_class.0));
    }
    if m.tagged {
        line2.push("tagged".into());
    }
    line2.push(zoom_label);
    let font1 = if compare { 11.0 } else { 12.0 };
    let font2 = if compare { 10.0 } else { 11.0 };
    // Text sits on a soft plate so it reads over any photo.
    let plate = Rect::from_min_size(
        rect.left_top() + Vec2::new(S3 - 4.0, S3 - 4.0),
        Vec2::new(
            (line1.chars().count().max(line2.join("   ").chars().count()) as f32) * 6.6 + 16.0,
            40.0,
        ),
    );
    p.rect_filled(
        plate.intersect(rect),
        CornerRadius::same(4),
        Color32::from_black_alpha(if t.dark { 110 } else { 150 }),
    );
    p.text(
        rect.left_top() + Vec2::new(S3 + 2.0, S3),
        Align2::LEFT_TOP,
        line1,
        FontId::monospace(font1),
        Color32::WHITE,
    );
    p.text(
        rect.left_top() + Vec2::new(S3 + 2.0, S3 + 18.0),
        Align2::LEFT_TOP,
        line2.join("   "),
        FontId::monospace(font2),
        Color32::from_white_alpha(200),
    );
    if let Some(c) = t.class_color(&app.scheme, m.color_class) {
        p.rect_filled(
            Rect::from_min_size(rect.left_top(), Vec2::new(4.0, rect.height())),
            CornerRadius::ZERO,
            c,
        );
    }
    if compare && focused {
        p.rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(2.0, t.accent),
            StrokeKind::Inside,
        );
    }
    resp.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            true,
            app.cell_accessible_name(idx),
        )
    });
    resp.clicked() && compare
}
