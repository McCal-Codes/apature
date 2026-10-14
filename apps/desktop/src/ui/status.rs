use egui::{Align, Layout};

use super::tokens::{Theme, S4, STATUS_H};
use crate::app::App;

pub fn show(app: &mut App, ctx: &egui::Context, t: &Theme) {
    egui::TopBottomPanel::bottom("status")
        .exact_height(STATUS_H)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(S4 as i8, 0)).stroke(egui::Stroke::NONE))
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = S4;
                for part in app.status_parts() {
                    ui.label(super::mono(part, 10.5, t.muted));
                }
                if !app.status.is_empty() {
                    ui.add(egui::Label::new(super::mono(&app.status, 10.5, t.accent)).truncate());
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
                });
            });
        });
}
