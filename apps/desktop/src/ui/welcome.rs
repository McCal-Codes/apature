//! Onboarding: a short welcome on first launch, and one tip the first time
//! each workspace opens. The point is to teach the loop (card in, cull,
//! caption, export, send) while the photographer runs it, not to lecture
//! before they can touch anything: every page has a way out and the whole
//! thing is four pages. Help › Welcome to Apature… brings it back; Settings
//! turns the tips off.

use egui::RichText;

use super::tokens::{Theme, S1, S2, S3, S4};
use crate::app::App;
use crate::commands::{Command, Workspace};
use crate::prefs::{Accent, Appearance, NumberKeys};

const PAGES: usize = 4;

/// The welcome, as a modal over the main window.
pub fn show(app: &mut App, ctx: &egui::Context, t: &Theme) {
    if !app.welcome_open {
        return;
    }
    let mut cmds: Vec<Command> = Vec::new();
    let mut close = false;
    let step = app.welcome_step.min(PAGES - 1);
    let modal = egui::Modal::new(egui::Id::new("welcome")).show(ctx, |ui| {
        ui.set_width(620.0);
        ui.spacing_mut().item_spacing.y = S2;
        // progress: four small marks
        ui.horizontal(|ui| {
            for i in 0..PAGES {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 4.0), egui::Sense::hover());
                let c = if i <= step { t.accent } else { t.line_strong };
                ui.painter()
                    .rect_filled(rect, egui::CornerRadius::same(2), c);
            }
            ui.add_space(S2);
            ui.label(
                RichText::new(format!("{} of {PAGES}", step + 1))
                    .size(10.5)
                    .color(t.muted),
            );
        });
        ui.add_space(S2);
        match step {
            0 => page_welcome(app, ui, t),
            1 => page_photos(app, ui, t, &mut cmds, &mut close),
            2 => page_keys(app, ui, t),
            _ => page_done(ui, t),
        }
        ui.add_space(S4);
        ui.horizontal(|ui| {
            if step > 0 && ui.button("Back").clicked() {
                app.welcome_step = step - 1;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if step + 1 < PAGES {
                    if ui
                        .add(egui::Button::new(RichText::new("Continue").strong()))
                        .clicked()
                    {
                        app.welcome_step = step + 1;
                    }
                    if ui
                        .button("Skip")
                        .on_hover_text("Help › Welcome to Apature… brings this back")
                        .clicked()
                    {
                        close = true;
                    }
                } else if ui
                    .add(egui::Button::new(RichText::new("Start").strong()))
                    .clicked()
                {
                    close = true;
                }
            });
        });
    });
    if modal.should_close() {
        close = true;
    }
    if close {
        app.welcome_open = false;
        app.welcome_step = 0;
        if !app.prefs.onboarded {
            app.prefs.onboarded = true;
            app.prefs_dirty = true;
        }
    }
    if app.welcome_import {
        app.welcome_import = false;
        app.welcome_open = false;
        app.welcome_step = 0;
        if !app.prefs.onboarded {
            app.prefs.onboarded = true;
            app.prefs_dirty = true;
        }
        app.dispatch(ctx, Command::ImportPhotoMechanic);
    }
    for c in cmds {
        app.dispatch(ctx, c);
    }
}

fn title(ui: &mut egui::Ui, t: &Theme, text: &str) {
    ui.label(RichText::new(text).size(20.0).strong().color(t.text));
}

fn body(ui: &mut egui::Ui, t: &Theme, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).size(12.5).color(t.text2)).wrap());
}

fn step_row(ui: &mut egui::Ui, t: &Theme, n: &str, name: &str, what: &str) {
    ui.horizontal_top(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), 11.0, t.accent_wash);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            n,
            egui::FontId::proportional(11.0),
            t.accent,
        );
        ui.add_space(S1);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(RichText::new(name).size(13.0).strong().color(t.text));
            ui.add(egui::Label::new(RichText::new(what).size(12.0).color(t.text2)).wrap());
        });
    });
}

fn page_welcome(app: &mut App, ui: &mut egui::Ui, t: &Theme) {
    title(ui, t, "Welcome to your daily ingesting of files");
    body(ui, t, "This can seem daunting, but don't worry, we can learn this one step at a time. It is similar to how Lightroom or Capture One copies your files from your card to its catalog, but this is faster, and it is easier to see the JPEGs, caption, tag and cull what you need and don't need (and a few other things too).");
    ui.add_space(S2);
    step_row(
        ui,
        t,
        "1",
        "Card in",
        "Copies and checks every file. A card never goes in twice.",
    );
    step_row(
        ui,
        t,
        "2",
        "Cull",
        "Number keys rate, Enter previews. Keep the keepers.",
    );
    step_row(
        ui,
        t,
        "3",
        "Caption",
        "Write it once, save it to the whole selection.",
    );
    step_row(
        ui,
        t,
        "4",
        "Send",
        "A preset makes the JPEGs; delivery takes them to the desk.",
    );
    ui.add_space(S2);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Already using Photo Mechanic? Feel free to skip this and import your workflow from there.").size(12.0).color(t.text2));
    });
    if ui
        .button("Import from Photo Mechanic…")
        .on_hover_text("An ingest snapshot (.snap) or a code replacement file")
        .clicked()
    {
        app.welcome_import = true;
    }
    ui.add_space(S3);
    super::eyebrow(ui, t, "Appearance");
    ui.horizontal(|ui| {
        for (a, label) in [
            (Appearance::System, "System"),
            (Appearance::Dark, "Apature Dark"),
            (Appearance::Light, "Apature Light"),
            (Appearance::Oled, "OLED"),
        ] {
            if super::icons::chip(ui, t, label, app.prefs.appearance == a).clicked() {
                app.prefs.appearance = a;
                app.prefs_dirty = true;
            }
        }
        ui.add_space(S3);
        ui.label(RichText::new("Accent").size(11.5).color(t.text2));
        for a in Accent::ALL {
            let swatch = Theme::for_ctx(ui.ctx()).with_accent(a);
            let selected = app.prefs.accent == a;
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
            let resp = resp.on_hover_text(a.label());
            let p = ui.painter();
            p.circle_filled(rect.center(), 8.0, swatch.accent);
            if selected {
                p.circle_stroke(rect.center(), 10.0, egui::Stroke::new(2.0, t.text));
            }
            if resp.clicked() {
                app.prefs.accent = a;
                app.prefs_dirty = true;
            }
        }
    });
}

fn page_photos(
    app: &mut App,
    ui: &mut egui::Ui,
    t: &Theme,
    cmds: &mut Vec<Command>,
    close: &mut bool,
) {
    title(ui, t, "Where are the pictures?");
    body(ui, t, "Apature works on folders and never edits an original. Marks and captions live in a sidecar next to each file.");
    ui.add_space(S2);
    let cards: Vec<(String, std::path::PathBuf)> = app
        .volumes
        .iter()
        .filter(|(_, p)| p.join("DCIM").is_dir())
        .cloned()
        .collect();
    if cards.is_empty() {
        body(ui, t, "No card plugged in yet. When one shows up, it lands in the sidebar and Ingest is one click away.");
    } else {
        super::eyebrow(ui, t, "Card detected");
        for (name, path) in &cards {
            ui.horizontal(|ui| {
                ui.label(RichText::new(name).size(13.0).strong().color(t.text));
                if ui.button("Ingest this card…").clicked() {
                    app.ingest.opts.source = path.clone();
                    cmds.push(Command::SetWorkspace(Workspace::Ingest));
                    *close = true;
                }
            });
        }
    }
    ui.add_space(S2);
    super::eyebrow(ui, t, "Or start with a folder");
    ui.horizontal(|ui| {
        if ui
            .add(egui::Button::new(RichText::new("Open a folder…").strong()))
            .clicked()
        {
            cmds.push(Command::PickFolder);
            *close = true;
        }
        if !app.prefs.recent.is_empty() {
            ui.label(
                RichText::new("Recent folders are in the sidebar.")
                    .size(11.5)
                    .color(t.muted),
            );
        }
    });
    ui.add_space(S1);
    body(
        ui,
        t,
        "Drop a folder on the window any time. More folders open as tabs.",
    );
}

fn page_keys(app: &mut App, ui: &mut egui::Ui, t: &Theme) {
    title(ui, t, "Keys that pay the rent");
    body(
        ui,
        t,
        "Hands on the keys, eyes on the frame. The rest is under Help › Keyboard Shortcuts.",
    );
    ui.add_space(S2);
    let rating_first = app.prefs.number_keys == NumberKeys::Rating;
    super::eyebrow(ui, t, "Number keys");
    ui.horizontal(|ui| {
        if super::icons::chip(ui, t, "1–5 rate, ⌘1–8 class", rating_first).clicked() {
            app.prefs.number_keys = NumberKeys::Rating;
            app.prefs_dirty = true;
        }
        if super::icons::chip(
            ui,
            t,
            "1–8 class, ⌘0–5 rate (Photo Mechanic)",
            !rating_first,
        )
        .clicked()
        {
            app.prefs.number_keys = NumberKeys::ColorClass;
            app.prefs_dirty = true;
        }
    });
    ui.add_space(S2);
    egui::Grid::new("welcome-keys")
        .num_columns(2)
        .spacing([S4, S1])
        .show(ui, |ui| {
            for (k, what) in [
                ("↑ ↓ ← →", "Move on the sheet; ⇧ extends the selection"),
                ("Enter / Esc", "Preview the frame, back to the sheet"),
                ("Z", "Loupe at 1:1 where the mouse is"),
                ("C", "Compare the selected frames, up to four"),
                ("T", "Tag; X rejects"),
                ("⌘⇧3", "Caption workspace; ⌘S saves to the selection"),
                ("⌘I / ⌘E", "Ingest a card / Export the selection"),
                ("⌘⌫", "Move to the Trash (asks first)"),
            ] {
                ui.label(RichText::new(k).monospace().size(12.0).color(t.accent));
                ui.label(RichText::new(what).size(12.0).color(t.text2));
                ui.end_row();
            }
        });
    ui.add_space(S2);
    if ui
        .checkbox(
            &mut app.prefs.show_tips,
            "Teach me as I go: one tip the first time each screen opens",
        )
        .changed()
    {
        app.prefs_dirty = true;
    }
}

fn page_done(ui: &mut egui::Ui, t: &Theme) {
    title(ui, t, "Why this exists");
    body(
        ui,
        t,
        "Photo Mechanic usually needs someone in the industry to teach it to you, and not many students learn it unless they land an internship. This is where this app comes in. It is free, for life. Yes, I am one person, still learning all of this and researching it actively.",
    );
    ui.add_space(S2);
    body(
        ui,
        t,
        "I took it on mainly because the alternative isn't cheap, and I was inspired by others. I feel that gatekeeping in the industry is wrong, and this is my way of making what I like and helping those who need the help. When I was throwing my SD cards into my desktop with no proper workflow, no backups, none of that, I get it. So I want to teach and learn continuously, so more photojournalists can have good practices.",
    );
    ui.add_space(S2);
    body(
        ui,
        t,
        "Open a folder or plug in a card to start. Want this again? Help › Welcome to Apature. Something broke? Help › Apature on GitHub.",
    );
}

/// The tip for a workspace: a short card the first time it opens.
fn tip_for(
    w: Workspace,
) -> (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
) {
    match w {
        Workspace::Sheet => (
            "sheet",
            "The contact sheet",
            &[
                ("1–5", "rate the focused frame"),
                ("⌘1–8", "colour class"),
                ("Enter", "preview"),
                ("C", "compare the selection"),
                (
                    "⌘A",
                    "select all; the inspector shows what the selection has in common",
                ),
            ],
        ),
        Workspace::Preview => (
            "preview",
            "Preview",
            &[
                ("Z", "loupe at 1:1 under the mouse"),
                ("Space", "1:1 toggle, one shared magnification"),
                ("R / ⇧R", "rotate; stored in the sidecar"),
                ("K", "crop; drag the frame, Enter applies"),
                ("Esc", "back to the sheet"),
            ],
        ),
        Workspace::Caption => (
            "caption",
            "Captions",
            &[
                (
                    "Load assignment",
                    "an assignment file fills the pad and the codes",
                ),
                ("\\code\\", "a code expands to its text as you type"),
                (
                    "{variables}",
                    "{headline}, {date}, {seqn}: the Photo Mechanic set",
                ),
                ("⌘S", "save to every selected photo"),
            ],
        ),
        Workspace::Ingest => (
            "ingest",
            "Ingest",
            &[
                ("Day chips", "pick the days to copy before anything moves"),
                (
                    "Templates",
                    "the example line shows the name every file will get",
                ),
                ("Verify", "every copy is hashed at every destination"),
                (
                    "Ledger",
                    "a card already ingested is skipped, never duplicated",
                ),
            ],
        ),
        Workspace::Export => (
            "export",
            "Export and deliver",
            &[
                ("Preset", "size, quality, naming and format per outlet"),
                ("Metadata", "EXIF, XMP and IPTC-IIM are all written"),
                ("Then send to", "a delivery target from Settings › Delivery"),
                ("Sent ledger", "every photo remembers where it went"),
            ],
        ),
    }
}

/// One tip per workspace, the first time it opens, in the top right of the
/// window. "Got it" marks it seen; "No more tips" turns them off.
pub fn tips(app: &mut App, ctx: &egui::Context, t: &Theme) {
    if !app.prefs.show_tips || app.welcome_open || app.view.is_empty() {
        return;
    }
    let (key, heading, rows) = tip_for(app.workspace);
    if app.prefs.tips_seen.iter().any(|k| k == key) {
        return;
    }
    let mut seen = false;
    let mut off = false;
    egui::Area::new(egui::Id::new("workspace-tip"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-S4, 92.0))
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(t.raised)
                .stroke(egui::Stroke::new(1.0, t.accent))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(S3 as i8))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 6],
                    blur: 18,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(90),
                })
                .show(ui, |ui| {
                    ui.set_width(300.0);
                    ui.spacing_mut().item_spacing.y = S1;
                    ui.label(RichText::new(heading).size(13.0).strong().color(t.text));
                    for (k, what) in rows {
                        ui.horizontal_top(|ui| {
                            ui.label(RichText::new(*k).monospace().size(11.5).color(t.accent));
                            ui.add(
                                egui::Label::new(RichText::new(*what).size(11.5).color(t.text2))
                                    .wrap(),
                            );
                        });
                    }
                    ui.add_space(S1);
                    ui.horizontal(|ui| {
                        if ui.small_button("Got it").clicked() {
                            seen = true;
                        }
                        if ui
                            .small_button("No more tips")
                            .on_hover_text("Settings › General turns them back on")
                            .clicked()
                        {
                            off = true;
                        }
                    });
                });
        });
    if seen || off {
        app.prefs.tips_seen.push(key.to_string());
        if off {
            app.prefs.show_tips = false;
        }
        app.prefs_dirty = true;
    }
}
