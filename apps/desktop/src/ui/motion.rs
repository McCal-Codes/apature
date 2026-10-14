//! Motion helpers. One curve, one duration, nothing under Reduce Motion.
//! See docs/design/VISUAL_IDENTITY.md and the roadmap's animation list.

use std::time::Instant;

/// The design's easing: fast start, soft landing (cubic-bezier(0.22, 1, 0.36, 1)
/// approximated by an ease-out cubic, which is visually equivalent at 160 ms).
pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Progress 0..=1 of an animation that started at `since` and lasts `secs`.
/// `secs == 0` (Reduce Motion) returns 1 immediately.
pub fn progress(since: Option<Instant>, secs: f32) -> f32 {
    match since {
        None => 1.0,
        Some(_) if secs <= 0.0 => 1.0,
        Some(s) => ease_out(s.elapsed().as_secs_f32() / secs),
    }
}

pub fn lerp_rect(a: egui::Rect, b: egui::Rect, t: f32) -> egui::Rect {
    egui::Rect::from_min_max(a.min.lerp(b.min, t), a.max.lerp(b.max, t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eases_and_clamps() {
        assert_eq!(ease_out(0.0), 0.0);
        assert_eq!(ease_out(1.0), 1.0);
        assert!(ease_out(0.5) > 0.5, "ease-out is ahead of linear mid-way");
        assert_eq!(ease_out(2.0), 1.0);
        assert_eq!(progress(None, 0.16), 1.0);
        assert_eq!(
            progress(Some(Instant::now()), 0.0),
            1.0,
            "reduce motion finishes instantly"
        );
    }
}
