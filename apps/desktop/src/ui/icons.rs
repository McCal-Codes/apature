//! Toolbar icons drawn with the painter, so they follow the tokens and need
//! no icon font. Each is a 16 pt glyph inside a 28×24 button.

use egui::{CornerRadius, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use super::tokens::Theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Sidebar,
    Inspector,
    Funnel,
}

pub fn button(
    ui: &mut egui::Ui,
    t: &Theme,
    icon: Icon,
    selected: bool,
    tooltip: &str,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(30.0, 24.0), Sense::click());
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, CornerRadius::same(6), t.accent_wash);
        p.rect_stroke(
            rect,
            CornerRadius::same(6),
            Stroke::new(1.0, t.accent),
            StrokeKind::Inside,
        );
    } else if resp.hovered() {
        p.rect_filled(rect, CornerRadius::same(6), t.raised);
    }
    let fg = if selected { t.accent } else { t.text2 };
    let s = Stroke::new(1.5, fg);
    let g = Rect::from_center_size(rect.center(), Vec2::new(16.0, 13.0));
    match icon {
        Icon::Sidebar => {
            p.rect_stroke(g, CornerRadius::same(2), s, StrokeKind::Inside);
            let x = g.left() + 6.0;
            p.line_segment([Pos2::new(x, g.top()), Pos2::new(x, g.bottom())], s);
        }
        Icon::Inspector => {
            p.rect_stroke(g, CornerRadius::same(2), s, StrokeKind::Inside);
            let x = g.right() - 6.0;
            p.line_segment([Pos2::new(x, g.top()), Pos2::new(x, g.bottom())], s);
        }
        Icon::Funnel => {
            let top_l = Pos2::new(g.left(), g.top() + 1.0);
            let top_r = Pos2::new(g.right(), g.top() + 1.0);
            let mid_l = Pos2::new(g.center().x - 2.5, g.center().y + 1.0);
            let mid_r = Pos2::new(g.center().x + 2.5, g.center().y + 1.0);
            let bot_l = Pos2::new(g.center().x - 2.5, g.bottom());
            let bot_r = Pos2::new(g.center().x + 2.5, g.bottom() - 2.0);
            p.line_segment([top_l, top_r], s);
            p.line_segment([top_l, mid_l], s);
            p.line_segment([top_r, mid_r], s);
            p.line_segment([mid_l, bot_l], s);
            p.line_segment([mid_r, bot_r], s);
            p.line_segment([bot_l, bot_r], s);
        }
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tooltip));
    resp.on_hover_text(tooltip)
}
