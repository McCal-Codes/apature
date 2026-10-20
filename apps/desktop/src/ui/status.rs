use egui::{Align, Layout};

use super::tokens::{Theme, S4, STATUS_H};
use crate::app::App;

pub fn show(app: &mut App, ctx: &egui::Context, t: &Theme) {
    egui::TopBottomPanel::bottom("status")
        .exact_height(STATUS_H)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(S4 as i8, 0)).stroke(egui::Stroke::NONE))
        .show(ctx, |ui| {
            // The stats claim the right edge first; the status line fills the
            // rest and truncates, so the two never overlap.
            // The stats claim the right edge first (only when the window is wide
            // enough to hold both); the status line fills the rest as one
            // truncating label, so the two never overlap.
            let wide = ui.available_width() >= 1100.0 && app.prefs.show_perf_stats;
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = S4;
                if wide {
                    let avg = if app.stats.thumbs_done > 0 { app.stats.decode_ms_total / app.stats.thumbs_done as f32 } else { 0.0 };
                    ui.label(super::mono(
                        format!(
                            "advance after mark: {} · scan {:.0} ms · frame {:.1} ms · thumbs {} ({} cached, {avg:.0} ms)",
                            if app.prefs.advance_after_mark { "on" } else { "off" },
                            app.stats.scan_ms,
                            app.stats.frame_ms_ema,
                            app.stats.thumbs_done,
                            app.stats.cache_hits
                        ),
                        10.0,
                        t.muted,
                    ));
                    ui.add_space(S4);
                }
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = S4;
                    let line = app.status_parts().join("  ·  ");
                    let width = ui.available_width();
                    if !app.status.is_empty() {
                        // Status message keeps at least a third of the bar.
                        let a = super::motion::progress(app.status_since, app.anim(0.12));
                        ui.add_sized(
                            [width * 0.66, ui.available_height()],
                            egui::Label::new(super::mono(line, 10.5, t.muted)).truncate(),
                        );
                        ui.add(egui::Label::new(super::mono(&app.status, 10.5, t.accent.gamma_multiply(a))).truncate());
                    } else {
                        ui.add(egui::Label::new(super::mono(line, 10.5, t.muted)).truncate());
                    }
                });
            });
        });
}
