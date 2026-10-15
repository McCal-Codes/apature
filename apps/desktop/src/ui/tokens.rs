//! Design tokens: one system, two appearances (see docs/design/VISUAL_IDENTITY.md).
//! Nothing in `ui/` uses a literal colour; everything comes through `Theme`.

use egui::{Color32, Visuals};

pub const S1: f32 = 4.0;
pub const S2: f32 = 8.0;
pub const S3: f32 = 12.0;
pub const S4: f32 = 16.0;
pub const S5: f32 = 24.0;

pub const TOOLBAR_H: f32 = 48.0;
pub const STATUS_H: f32 = 28.0;
pub const FILMSTRIP_H: f32 = 88.0;
pub const SIDEBAR_RANGE: std::ops::RangeInclusive<f32> = 200.0..=320.0;
pub const INSPECTOR_RANGE: std::ops::RangeInclusive<f32> = 280.0..=420.0;
pub const ROW_H: f32 = 26.0;
pub const LABEL_H: f32 = 34.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub dark: bool,
    pub window: Color32,
    pub panel: Color32,
    pub raised: Color32,
    pub photo: Color32,
    pub photo_cell: Color32,
    pub text: Color32,
    pub text2: Color32,
    pub muted: Color32,
    pub line: Color32,
    pub line_strong: Color32,
    pub accent: Color32,
    pub accent_wash: Color32,
    pub star: Color32,
    pub reject: Color32,
    pub tag: Color32,
    pub select_photo: Color32,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

impl Theme {
    /// Apature Dark: Professional Darkroom.
    pub const DARK: Theme = Theme {
        dark: true,
        window: rgb(0x12, 0x12, 0x14),
        panel: rgb(0x1A, 0x1A, 0x1D),
        raised: rgb(0x24, 0x24, 0x28),
        photo: rgb(0x16, 0x16, 0x18),
        photo_cell: rgb(0x20, 0x20, 0x24),
        text: rgb(0xE8, 0xE7, 0xEE),
        text2: rgb(0xB3, 0xB1, 0xC0),
        muted: rgb(0x8A, 0x88, 0x99),
        line: rgb(0x2A, 0x2A, 0x30),
        line_strong: rgb(0x3B, 0x3A, 0x4B),
        accent: rgb(0xCF, 0xC8, 0xF5),
        accent_wash: Color32::from_rgba_premultiplied(0x2A, 0x29, 0x33, 0xFF),
        star: rgb(0xE3, 0xB3, 0x41),
        reject: rgb(0xE0, 0x6C, 0x6C),
        tag: rgb(0x2F, 0x7F, 0xE0),
        select_photo: Color32::from_rgba_premultiplied(180, 180, 180, 180),
    };

    /// Apature Light: Mac Studio.
    pub const LIGHT: Theme = Theme {
        dark: false,
        window: rgb(0xF2, 0xF2, 0xF5),
        panel: rgb(0xFA, 0xFA, 0xFC),
        raised: rgb(0xE9, 0xE9, 0xEE),
        photo: rgb(0xD8, 0xD8, 0xDC),
        photo_cell: rgb(0xCF, 0xCF, 0xD4),
        text: rgb(0x1D, 0x1D, 0x22),
        text2: rgb(0x4D, 0x4D, 0x58),
        muted: rgb(0x6F, 0x6F, 0x7C),
        line: rgb(0xDE, 0xDE, 0xE4),
        line_strong: rgb(0xB9, 0xB6, 0xCB),
        accent: rgb(0x0A, 0x66, 0xD6),
        accent_wash: rgb(0xDD, 0xE8, 0xF9),
        star: rgb(0xB8, 0x86, 0x0B),
        reject: rgb(0xC0, 0x3C, 0x3C),
        tag: rgb(0x0A, 0x66, 0xD6),
        select_photo: Color32::from_rgba_premultiplied(0, 0, 0, 150),
    };

    pub fn for_ctx(ctx: &egui::Context) -> Theme {
        match ctx.theme() {
            egui::Theme::Dark => Theme::DARK,
            egui::Theme::Light => Theme::LIGHT,
        }
    }

    /// Accessibility: lift the quiet greys one step and strengthen borders.
    /// Hierarchy survives (text > text2 > muted), the floor just rises.
    pub fn high_contrast(mut self) -> Theme {
        self.muted = self.text2;
        self.text2 = self.text;
        self.line = self.line_strong;
        self.line_strong = self.text2;
        self
    }

    /// Push the tokens into egui's own widget styling so built-in widgets
    /// (buttons, sliders, combo boxes) match the panels.
    pub fn apply(&self, ctx: &egui::Context, reduce_motion: bool) {
        let mut v = if self.dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        v.panel_fill = self.panel;
        v.window_fill = self.raised;
        v.extreme_bg_color = self.raised;
        v.faint_bg_color = self.raised;
        v.override_text_color = Some(self.text);
        v.hyperlink_color = self.accent;
        v.selection.bg_fill = self.accent_wash;
        v.selection.stroke = egui::Stroke::new(1.0, self.accent);
        v.widgets.noninteractive.bg_fill = self.panel;
        v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, self.line);
        v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, self.text2);
        v.widgets.inactive.bg_fill = self.raised;
        v.widgets.inactive.weak_bg_fill = self.raised;
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, self.line);
        v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, self.text2);
        v.widgets.hovered.bg_fill = self.raised;
        v.widgets.hovered.weak_bg_fill = self.raised;
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, self.line_strong);
        v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, self.text);
        v.widgets.active.bg_fill = self.accent_wash;
        v.widgets.active.weak_bg_fill = self.accent_wash;
        v.widgets.active.bg_stroke = egui::Stroke::new(1.0, self.accent);
        v.widgets.active.fg_stroke = egui::Stroke::new(1.0, self.text);
        v.widgets.open.bg_fill = self.raised;
        v.widgets.open.weak_bg_fill = self.raised;
        v.widgets.open.bg_stroke = egui::Stroke::new(1.0, self.line_strong);
        v.widgets.open.fg_stroke = egui::Stroke::new(1.0, self.text);
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.corner_radius = egui::CornerRadius::same(6);
        }
        v.window_corner_radius = egui::CornerRadius::same(8);
        v.menu_corner_radius = egui::CornerRadius::same(8);
        v.window_stroke = egui::Stroke::new(1.0, self.line);
        v.window_shadow = egui::epaint::Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(90),
        };
        v.popup_shadow = v.window_shadow;
        ctx.set_visuals(v);
        ctx.style_mut(|s| {
            s.spacing.item_spacing = egui::vec2(S2, S2);
            s.spacing.button_padding = egui::vec2(10.0, 5.0);
            s.spacing.interact_size.y = 24.0;
            s.animation_time = if reduce_motion { 0.0 } else { 0.16 };
        });
    }

    pub fn class_color(
        &self,
        scheme: &apature_core::ColorClassScheme,
        c: apature_core::ColorClass,
    ) -> Option<Color32> {
        scheme.color(c).map(|[r, g, b]| Color32::from_rgb(r, g, b))
    }
}

/// System fonts when available (SF Pro / SF Mono on macOS), validated with
/// ab_glyph before registration so a bad file can never panic egui.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let mut changed = false;
    for (name, path, family) in [
        (
            "SF Pro",
            "/System/Library/Fonts/SFNS.ttf",
            egui::FontFamily::Proportional,
        ),
        (
            "SF Mono",
            "/System/Library/Fonts/SFNSMono.ttf",
            egui::FontFamily::Monospace,
        ),
    ] {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        if ab_glyph::FontRef::try_from_slice(&bytes).is_err() {
            log::info!("{path} is not loadable by ab_glyph; keeping the bundled font");
            continue;
        }
        fonts.font_data.insert(
            name.to_string(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, name.to_string());
        changed = true;
    }
    if changed {
        ctx.set_fonts(fonts);
    }
}
