//! Distraction-free preview workspace.

use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, Vec2};

use super::tokens::{Theme, S3, S5};
use crate::app::App;

pub fn show(app: &mut App, ui: &mut egui::Ui, t: &Theme) {
    let rect = ui.available_rect_before_wrap();
    let ppp = ui.ctx().pixels_per_point();
    let long_side = ((rect.width().max(rect.height())) * ppp).ceil() as u32;
    app.request_preview(app.cursor, long_side, 0);
    if let Some(n) = app.view_neighbor(1) {
        app.request_preview(n, long_side, 1);
    }
    if let Some(p) = app.view_neighbor(-1) {
        app.request_preview(p, long_side, 2);
    }
    let idx = app.cursor;
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::ZERO, t.photo);
    let tex = app
        .previews
        .get(&idx)
        .map(|(h, _)| h)
        .or(app.thumbs[idx].as_ref());
    let mut zoom_label = String::from("loading…");
    if let Some(tex) = tex {
        let size = tex.size_vec2();
        let inner = rect.shrink(S5);
        let s = (inner.width() / size.x)
            .min(inner.height() / size.y)
            .min(1.0);
        let fitted = Rect::from_center_size(rect.center(), size * s);
        p.image(
            tex.id(),
            fitted,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        zoom_label = match app.previews.get(&idx) {
            Some((_, ls)) => format!("fit · {ls} px preview"),
            None => "fit · thumbnail, full preview loading".into(),
        };
    }
    let m = app.marks[idx];
    let line1 = format!(
        "{}   {} / {}",
        app.items[idx].display_name(),
        idx + 1,
        app.items.len()
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
    p.text(
        rect.left_top() + Vec2::new(S3 + 2.0, S3),
        Align2::LEFT_TOP,
        line1,
        FontId::monospace(12.0),
        t.text,
    );
    p.text(
        rect.left_top() + Vec2::new(S3 + 2.0, S3 + 18.0),
        Align2::LEFT_TOP,
        line2.join("   "),
        FontId::monospace(11.0),
        t.text2,
    );
    if let Some(c) = t.class_color(&app.scheme, m.color_class) {
        p.rect_filled(
            Rect::from_min_size(rect.left_top(), Vec2::new(4.0, rect.height())),
            CornerRadius::ZERO,
            c,
        );
    }
    if app.cols > 0 {
        let row = app.cursor_pos() / app.cols;
        let rows = app.row_count();
        app.schedule_thumbs(row.saturating_sub(1)..(row + 2).min(rows));
    }
}
