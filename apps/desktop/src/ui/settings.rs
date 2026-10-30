//! Settings, in its own window (⌘,), as macOS expects.
//!
//! Built the way TerraNova's settings dialog is: a rail of categories on the
//! left with a count of what differs from the default, a search box that
//! finds any setting by its label or description and shows where it lives,
//! and rows that all look the same, label and description on the left,
//! the control on the right, a Reset when the value is not the default.
//! Every plain setting is one entry in `DEFS`; the few things that are not a
//! single value (delivery targets, colour classes, the shortcut table) are
//! panels that live under a category and are found by their section name.

use egui::{Color32, RichText};

use apature_core::ColorClassScheme;

use super::tokens::{Theme, S1, S2, S3, S4};
use crate::app::App;
use crate::commands::Command;
use crate::prefs::{Accent, Appearance, Motion, NumberKeys, Prefs};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    General,
    Classes,
    Metadata,
    Ingest,
    Accessibility,
    Delivery,
    Cache,
    Shortcuts,
    Resources,
}

const PAGES: [(Page, &str); 9] = [
    (Page::General, "General"),
    (Page::Classes, "Colour classes"),
    (Page::Metadata, "Metadata"),
    (Page::Ingest, "Ingest"),
    (Page::Accessibility, "Accessibility"),
    (Page::Delivery, "Delivery"),
    (Page::Cache, "Cache & diagnostics"),
    (Page::Shortcuts, "Shortcuts"),
    (Page::Resources, "Resources"),
];

fn page_label(p: Page) -> &'static str {
    PAGES
        .iter()
        .find(|(q, _)| *q == p)
        .map(|(_, l)| *l)
        .unwrap_or("")
}

/* -- The registry ---------------------------------------------------------- */

/// Every single-value setting. Adding one means adding a line here and an
/// arm in `control`, `modified` and `reset`; the pages never change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Id {
    Appearance,
    Accent,
    ShowTips,
    AdvanceAfterMark,
    PreviewInWindow,
    StackBursts,
    OpenAfterIngest,
    CheckUpdates,
    BetaUpdates,
    Cell,
    CodeDelimiter,
    SeqContinue,
    FolderTemplate,
    RenameTemplate,
    OnlyNew,
    IncludeSubfolders,
    PreserveFolders,
    CopyCompanions,
    OpenIngestOnCard,
    ReceivePort,
    UiScale,
    Motion,
    HighContrast,
    ShowPerfStats,
    NumberKeys,
}

struct Def {
    id: Id,
    page: Page,
    section: &'static str,
    label: &'static str,
    description: &'static str,
}

const DEFS: &[Def] = &[
    Def { id: Id::Appearance, page: Page::General, section: "Appearance", label: "Appearance", description: "System follows System Settings › Appearance. OLED is Dark on a true black ground." },
    Def { id: Id::Accent, page: Page::General, section: "Appearance", label: "Accent", description: "Selection, links and focus take the accent. Lavender is the Apature default." },
    Def { id: Id::ShowTips, page: Page::General, section: "Learning the loop", label: "Tips as you go", description: "One short tip the first time each workspace opens." },
    Def { id: Id::AdvanceAfterMark, page: Page::General, section: "Behaviour", label: "Advance after marking", description: "Move to the next photo after a rating, class or tag." },
    Def { id: Id::PreviewInWindow, page: Page::General, section: "Behaviour", label: "Preview in its own window", description: "The sheet stays where it is; good for two screens. Enter and Esc still toggle it." },
    Def { id: Id::StackBursts, page: Page::General, section: "Behaviour", label: "Stack bursts", description: "Frames shot within a second sit behind one thumbnail. S expands the stack." },
    Def { id: Id::OpenAfterIngest, page: Page::General, section: "Behaviour", label: "Open the ingested folder", description: "When an ingest finishes, open its folder in a new tab." },
    Def { id: Id::CheckUpdates, page: Page::General, section: "Updates", label: "Check for updates once a day", description: "One request to GitHub Releases. Nothing about you or your photos is sent." },
    Def { id: Id::BetaUpdates, page: Page::General, section: "Updates", label: "Include beta releases", description: "Offer pre-releases as well as final builds." },
    Def { id: Id::Cell, page: Page::General, section: "Sheet", label: "Default thumbnail size", description: "The size a new folder opens at. ⌘= and ⌘− change it while you work." },
    Def { id: Id::CodeDelimiter, page: Page::Metadata, section: "Code replacements", label: "Code delimiter", description: "Type \\cf\\ (or =cf= with Photo Mechanic's default) to expand a code while captioning." },
    Def { id: Id::SeqContinue, page: Page::Ingest, section: "Sequence", label: "Keep {seqn} counting", description: "The sequence continues across ingests with the same headline, as Photo Mechanic's persistent counter does." },
    Def { id: Id::FolderTemplate, page: Page::Ingest, section: "Templates", label: "Folder template", description: "Where each day's photos go, under the destination." },
    Def { id: Id::RenameTemplate, page: Page::Ingest, section: "Templates", label: "Rename template", description: "The name every copied file gets; the default is the Photo Mechanic ingest rename (260505_CAL500). Tokens: {filename} {ext} {frame} {seqn} {seqn:4} {headline} {date} {yymmdd} {yr2} {month0} {day0} {time}." },
    Def { id: Id::OnlyNew, page: Page::Ingest, section: "Defaults", label: "Only photos not ingested before", description: "The ledger remembers every card; files already copied are skipped." },
    Def { id: Id::IncludeSubfolders, page: Page::Ingest, section: "Defaults", label: "Include subfolders", description: "Look inside DCIM/100NIKON and the like." },
    Def { id: Id::PreserveFolders, page: Page::Ingest, section: "Defaults", label: "Keep the card's folders", description: "Copy the card's folder structure instead of the folder template." },
    Def { id: Id::CopyCompanions, page: Page::Ingest, section: "Defaults", label: "Copy voice memos and sidecars", description: "WAV memos and XMP sidecars travel with their photos." },
    Def { id: Id::OpenIngestOnCard, page: Page::Ingest, section: "Cards", label: "Open Ingest when a card mounts", description: "A volume with a DCIM folder switches to Ingest with it as the source." },
    Def { id: Id::ReceivePort, page: Page::Ingest, section: "Receive over Wi-Fi", label: "Port", description: "The FTP server's port for phones and cameras on the same network. 2121 needs no special rights." },
    Def { id: Id::UiScale, page: Page::Accessibility, section: "Text and control size", label: "Scale", description: "Scales every panel, label and control. ⌘+ and ⌘− do the same while the app is open." },
    Def { id: Id::Motion, page: Page::Accessibility, section: "Motion", label: "Motion", description: "Reduce turns fades, slides and the ingest stagger into instant changes. Nothing loops or autoplays either way." },
    Def { id: Id::HighContrast, page: Page::Accessibility, section: "Contrast", label: "Higher contrast", description: "Lifts the quiet grey labels and strengthens panel borders in both appearances." },
    Def { id: Id::ShowPerfStats, page: Page::Cache, section: "Diagnostics", label: "Timings in the status bar", description: "Scan, frame and thumbnail times. The same numbers go to apature.log either way." },
    Def { id: Id::NumberKeys, page: Page::Shortcuts, section: "Number keys", label: "What 1–8 do", description: "Stars: 1–5 rate and ⌘1–⌘8 class. Colour class: 1–8 class and ⌘0–⌘5 rate, as in Photo Mechanic." },
];

/// Panels that are not one value: found by search through their section
/// name and a few words, opened by jumping to the page.
const PANELS: &[(Page, &str, &str)] = &[
    (
        Page::General,
        "Welcome",
        "welcome tour onboarding tips reset",
    ),
    (
        Page::General,
        "Edit photos in",
        "capture one photoshop lightroom editor open edit photos external hand off",
    ),
    (
        Page::General,
        "Files Apature keeps",
        "prefs.json log reveal reset settings folder",
    ),
    (
        Page::Classes,
        "Colour classes",
        "label colour color class names photo mechanic xmp:Label",
    ),
    (
        Page::Metadata,
        "Assignment file",
        "assignment pad iptc stationery",
    ),
    (
        Page::Metadata,
        "Code replacement file",
        "codes replacements",
    ),
    (
        Page::Delivery,
        "Delivery targets",
        "ftp sftp ftps folder server password keychain send desk",
    ),
    (
        Page::Cache,
        "Thumbnail cache",
        "cache clear size thumbnails",
    ),
    (Page::Shortcuts, "All shortcuts", "keyboard keys shortcuts"),
    (
        Page::Resources,
        "Style and ethics",
        "ap stylebook caption ethics nppa spj",
    ),
    (
        Page::Resources,
        "Internships, workshops and contests",
        "internship job workshop student eddie adams missouri cpoy dow jones chips quinn",
    ),
];

fn ingest_opts(app: &App) -> apature_ingest::Options {
    app.prefs
        .ingest
        .clone()
        .unwrap_or_else(crate::prefs::default_ingest_options)
}

fn set_ingest(app: &mut App, o: apature_ingest::Options) {
    app.ingest.opts = o.clone();
    app.prefs.ingest = Some(o);
    app.prefs_dirty = true;
}

fn modified(app: &App, id: Id) -> bool {
    let d = Prefs::default();
    let p = &app.prefs;
    let io = ingest_opts(app);
    let id_ = crate::prefs::default_ingest_options();
    match id {
        Id::Appearance => p.appearance != d.appearance,
        Id::Accent => p.accent != d.accent,
        Id::ShowTips => p.show_tips != d.show_tips,
        Id::AdvanceAfterMark => p.advance_after_mark != d.advance_after_mark,
        Id::PreviewInWindow => p.preview_in_window != d.preview_in_window,
        Id::StackBursts => p.stack_bursts != d.stack_bursts,
        Id::OpenAfterIngest => p.open_after_ingest != d.open_after_ingest,
        Id::CheckUpdates => p.check_updates != d.check_updates,
        Id::BetaUpdates => p.beta_updates != d.beta_updates,
        Id::Cell => (p.cell - d.cell).abs() > 0.5,
        Id::CodeDelimiter => p.code_delimiter != d.code_delimiter,
        Id::SeqContinue => p.seq_continue != d.seq_continue,
        Id::FolderTemplate => io.folder_template != id_.folder_template,
        Id::RenameTemplate => io.rename_template != id_.rename_template,
        Id::OnlyNew => io.only_new != id_.only_new,
        Id::IncludeSubfolders => io.include_subfolders != id_.include_subfolders,
        Id::PreserveFolders => io.preserve_folders != id_.preserve_folders,
        Id::CopyCompanions => io.copy_companions != id_.copy_companions,
        Id::OpenIngestOnCard => p.open_ingest_on_card != d.open_ingest_on_card,
        Id::ReceivePort => p.receive_port != d.receive_port,
        Id::UiScale => (p.ui_scale - d.ui_scale).abs() > 0.01,
        Id::Motion => p.motion != d.motion,
        Id::HighContrast => p.high_contrast != d.high_contrast,
        Id::ShowPerfStats => p.show_perf_stats != d.show_perf_stats,
        Id::NumberKeys => p.number_keys != d.number_keys,
    }
}

fn reset(app: &mut App, id: Id) {
    let d = Prefs::default();
    let id_ = crate::prefs::default_ingest_options();
    let mut io = ingest_opts(app);
    match id {
        Id::Appearance => app.prefs.appearance = d.appearance,
        Id::Accent => app.prefs.accent = d.accent,
        Id::ShowTips => app.prefs.show_tips = d.show_tips,
        Id::AdvanceAfterMark => app.prefs.advance_after_mark = d.advance_after_mark,
        Id::PreviewInWindow => app.prefs.preview_in_window = d.preview_in_window,
        Id::StackBursts => {
            app.prefs.stack_bursts = d.stack_bursts;
            app.rebuild_view();
        }
        Id::OpenAfterIngest => app.prefs.open_after_ingest = d.open_after_ingest,
        Id::CheckUpdates => app.prefs.check_updates = d.check_updates,
        Id::BetaUpdates => app.prefs.beta_updates = d.beta_updates,
        Id::Cell => app.set_cell(d.cell),
        Id::CodeDelimiter => app.prefs.code_delimiter = d.code_delimiter,
        Id::SeqContinue => app.prefs.seq_continue = d.seq_continue,
        Id::FolderTemplate => {
            io.folder_template = id_.folder_template;
            set_ingest(app, io);
        }
        Id::RenameTemplate => {
            io.rename_template = id_.rename_template;
            set_ingest(app, io);
        }
        Id::OnlyNew => {
            io.only_new = id_.only_new;
            set_ingest(app, io);
        }
        Id::IncludeSubfolders => {
            io.include_subfolders = id_.include_subfolders;
            set_ingest(app, io);
        }
        Id::PreserveFolders => {
            io.preserve_folders = id_.preserve_folders;
            set_ingest(app, io);
        }
        Id::CopyCompanions => {
            io.copy_companions = id_.copy_companions;
            set_ingest(app, io);
        }
        Id::OpenIngestOnCard => app.prefs.open_ingest_on_card = d.open_ingest_on_card,
        Id::ReceivePort => app.prefs.receive_port = d.receive_port,
        Id::UiScale => app.prefs.ui_scale = d.ui_scale,
        Id::Motion => app.prefs.motion = d.motion,
        Id::HighContrast => app.prefs.high_contrast = d.high_contrast,
        Id::ShowPerfStats => app.prefs.show_perf_stats = d.show_perf_stats,
        Id::NumberKeys => app.prefs.number_keys = d.number_keys,
    }
    app.prefs_dirty = true;
}

/// The control for one setting. Returns true when the value changed.
fn control(app: &mut App, ui: &mut egui::Ui, t: &Theme, id: Id) -> bool {
    match id {
        Id::Appearance => chips(
            ui,
            t,
            &mut app.prefs.appearance,
            &[
                (Appearance::System, "System"),
                (Appearance::Dark, "Dark"),
                (Appearance::Light, "Light"),
                (Appearance::Oled, "OLED"),
            ],
        ),
        Id::Accent => accent_swatches(app, ui, t),
        Id::ShowTips => toggle(ui, t, &mut app.prefs.show_tips),
        Id::AdvanceAfterMark => toggle(ui, t, &mut app.prefs.advance_after_mark),
        Id::PreviewInWindow => toggle(ui, t, &mut app.prefs.preview_in_window),
        Id::StackBursts => {
            let c = toggle(ui, t, &mut app.prefs.stack_bursts);
            if c {
                app.rebuild_view();
            }
            c
        }
        Id::OpenAfterIngest => toggle(ui, t, &mut app.prefs.open_after_ingest),
        Id::CheckUpdates => toggle(ui, t, &mut app.prefs.check_updates),
        Id::BetaUpdates => {
            ui.add_enabled_ui(app.prefs.check_updates, |ui| {
                toggle(ui, t, &mut app.prefs.beta_updates)
            })
            .inner
        }
        Id::Cell => {
            let mut cell = app.prefs.cell;
            let c = ui
                .add(
                    egui::Slider::new(&mut cell, 80.0..=600.0)
                        .suffix(" pt")
                        .show_value(true),
                )
                .changed();
            if c {
                app.set_cell(cell);
            }
            c
        }
        Id::CodeDelimiter => {
            let mut d = app.prefs.code_delimiter.to_string();
            let c = ui
                .add(
                    egui::TextEdit::singleline(&mut d)
                        .desired_width(32.0)
                        .char_limit(1),
                )
                .changed();
            if c {
                if let Some(ch) = d.chars().next() {
                    app.prefs.code_delimiter = ch;
                }
            }
            c
        }
        Id::SeqContinue => toggle(ui, t, &mut app.prefs.seq_continue),
        Id::FolderTemplate | Id::RenameTemplate => {
            let mut io = ingest_opts(app);
            let field = if id == Id::FolderTemplate {
                &mut io.folder_template
            } else {
                &mut io.rename_template
            };
            let c = ui
                .add(
                    egui::TextEdit::singleline(field)
                        .desired_width(300.0)
                        .font(egui::TextStyle::Monospace),
                )
                .changed();
            if c {
                set_ingest(app, io);
            }
            c
        }
        Id::OnlyNew | Id::IncludeSubfolders | Id::PreserveFolders | Id::CopyCompanions => {
            let mut io = ingest_opts(app);
            let field = match id {
                Id::OnlyNew => &mut io.only_new,
                Id::IncludeSubfolders => &mut io.include_subfolders,
                Id::PreserveFolders => &mut io.preserve_folders,
                _ => &mut io.copy_companions,
            };
            let c = toggle(ui, t, field);
            if c {
                set_ingest(app, io);
            }
            c
        }
        Id::OpenIngestOnCard => toggle(ui, t, &mut app.prefs.open_ingest_on_card),
        Id::ReceivePort => ui
            .add(egui::DragValue::new(&mut app.prefs.receive_port).range(1024..=65535))
            .changed(),
        Id::UiScale => {
            let mut pct = (app.prefs.ui_scale * 100.0).round();
            let c = ui
                .add(
                    egui::Slider::new(&mut pct, 80.0..=180.0)
                        .step_by(5.0)
                        .suffix(" %"),
                )
                .changed();
            if c {
                app.prefs.ui_scale = pct / 100.0;
            }
            c
        }
        Id::Motion => chips(
            ui,
            t,
            &mut app.prefs.motion,
            &[
                (Motion::System, "System"),
                (Motion::Reduce, "Reduce"),
                (Motion::Full, "Full"),
            ],
        ),
        Id::HighContrast => toggle(ui, t, &mut app.prefs.high_contrast),
        Id::ShowPerfStats => toggle(ui, t, &mut app.prefs.show_perf_stats),
        Id::NumberKeys => chips(
            ui,
            t,
            &mut app.prefs.number_keys,
            &[
                (NumberKeys::Rating, "Stars"),
                (NumberKeys::ColorClass, "Colour class"),
            ],
        ),
    }
}

/* -- Controls -------------------------------------------------------------- */

/// A switch: a pill that slides, the way System Settings draws one.
pub(crate) fn toggle(ui: &mut egui::Ui, t: &Theme, on: &mut bool) -> bool {
    let size = egui::vec2(34.0, 20.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let mut changed = false;
    if resp.clicked() {
        *on = !*on;
        changed = true;
    }
    let k = ui.ctx().animate_bool_with_time(resp.id, *on, 0.12);
    let p = ui.painter();
    let track = if *on { t.accent } else { t.line_strong };
    p.rect_filled(rect, egui::CornerRadius::same(10), track);
    let x = rect.left() + 10.0 + k * (size.x - 20.0);
    p.circle_filled(
        egui::pos2(x, rect.center().y),
        8.0,
        if *on { t.window } else { t.text },
    );
    if resp.has_focus() {
        p.rect_stroke(
            rect.expand(2.0),
            egui::CornerRadius::same(12),
            egui::Stroke::new(1.5, t.accent),
            egui::StrokeKind::Outside,
        );
    }
    let _ = resp.on_hover_text(if *on { "On" } else { "Off" });
    changed
}

fn chips<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    t: &Theme,
    value: &mut T,
    options: &[(T, &str)],
) -> bool {
    let mut changed = false;
    // Inside the right-aligned control column egui lays rows out right to
    // left, so draw the options in reverse to keep their reading order.
    let rtl = ui.layout().main_dir() == egui::Direction::RightToLeft;
    let ordered: Vec<&(T, &str)> = if rtl {
        options.iter().rev().collect()
    } else {
        options.iter().collect()
    };
    ui.horizontal(|ui| {
        for (v, label) in ordered {
            if super::icons::chip(ui, t, label, *value == *v).clicked() && *value != *v {
                *value = *v;
                changed = true;
            }
        }
    });
    changed
}

fn accent_swatches(app: &mut App, ui: &mut egui::Ui, t: &Theme) -> bool {
    let mut changed = false;
    let rtl = ui.layout().main_dir() == egui::Direction::RightToLeft;
    let mut all = Accent::ALL.to_vec();
    if rtl {
        all.reverse();
    }
    ui.horizontal(|ui| {
        if rtl {
            ui.label(
                RichText::new(app.prefs.accent.label())
                    .size(11.5)
                    .color(t.muted),
            );
        }
        for a in all {
            let swatch = Theme::for_ctx(ui.ctx()).with_accent(a);
            let selected = app.prefs.accent == a;
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
            let resp = resp.on_hover_text(a.label());
            let p = ui.painter();
            p.circle_filled(rect.center(), 9.0, swatch.accent);
            if selected {
                p.circle_stroke(rect.center(), 11.0, egui::Stroke::new(2.0, t.text));
            } else if resp.hovered() {
                p.circle_stroke(rect.center(), 11.0, egui::Stroke::new(1.0, t.line_strong));
            }
            if resp.clicked() && !selected {
                app.prefs.accent = a;
                changed = true;
            }
        }
        ui.label(
            RichText::new(app.prefs.accent.label())
                .size(11.5)
                .color(t.muted),
        );
    });
    changed
}

/* -- Layout ---------------------------------------------------------------- */

fn heading(ui: &mut egui::Ui, t: &Theme, text: &str) {
    ui.label(RichText::new(text).size(17.0).strong().color(t.text));
}

fn hint(ui: &mut egui::Ui, t: &Theme, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).size(11.0).color(t.muted)).wrap());
}

/// A card for one section: eyebrow title, then rows separated by hairlines.
fn card<R>(ui: &mut egui::Ui, t: &Theme, title: &str, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.add_space(S2);
    super::eyebrow(ui, t, title);
    ui.add_space(S1);
    egui::Frame::NONE
        .fill(t.panel)
        .stroke(egui::Stroke::new(1.0, t.line))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(S3 as i8, S1 as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            add(ui)
        })
        .inner
}

/// One row: label and description left, control right, an accent bar and a
/// Reset button when the value differs from the default.
fn setting_row(
    app: &mut App,
    ui: &mut egui::Ui,
    t: &Theme,
    def: &Def,
    breadcrumb: Option<String>,
    last: bool,
) {
    let is_mod = modified(app, def.id);
    let top = ui.cursor().top();
    let mut reset_clicked = false;
    let mut changed = false;
    ui.horizontal_top(|ui| {
        ui.add_space(S1);
        let label_w = (ui.available_width() - 300.0).max(160.0);
        let left = ui.vertical(|ui| {
            ui.add_space(S2 + 2.0);
            ui.set_max_width(label_w);
            ui.set_min_width(label_w);
            if let Some(b) = &breadcrumb {
                ui.label(RichText::new(b.to_uppercase()).size(9.5).color(t.muted));
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new(def.label).size(13.0).color(t.text));
                if is_mod {
                    ui.label(RichText::new("Modified").size(9.5).color(t.accent));
                }
            });
            if !def.description.is_empty() {
                ui.add(
                    egui::Label::new(RichText::new(def.description).size(11.0).color(t.muted))
                        .wrap(),
                );
            }
            ui.add_space(S2 + 2.0);
        });
        let h = left.response.rect.height();
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), h),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                ui.add_space(S1);
                if is_mod {
                    if ui
                        .small_button("Reset")
                        .on_hover_text("Back to the default")
                        .clicked()
                    {
                        reset_clicked = true;
                    }
                    ui.add_space(S1);
                }
                changed = control(app, ui, t, def.id);
            },
        );
    });
    if changed {
        app.prefs_dirty = true;
    }
    if reset_clicked {
        reset(app, def.id);
    }
    if is_mod {
        let bottom = ui.cursor().top();
        let r = ui.max_rect();
        ui.painter().rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(r.left() - S3 + 2.0, top + S2),
                egui::pos2(r.left() - S3 + 4.0, bottom - S2),
            ),
            egui::CornerRadius::same(1),
            t.accent,
        );
    }
    if !last {
        let r = ui.available_rect_before_wrap();
        ui.painter().hline(
            r.x_range(),
            ui.cursor().top(),
            egui::Stroke::new(1.0, t.line),
        );
    }
}

/// The rows of one section on one page, as a card.
fn section_card(app: &mut App, ui: &mut egui::Ui, t: &Theme, page: Page, section: &str) {
    let defs: Vec<&Def> = DEFS
        .iter()
        .filter(|d| d.page == page && d.section == section)
        .collect();
    if defs.is_empty() {
        return;
    }
    let n = defs.len();
    card(ui, t, section, |ui| {
        for (i, d) in defs.iter().enumerate() {
            setting_row(app, ui, t, d, None, i + 1 == n);
        }
    });
}

fn sections_of(page: Page) -> Vec<&'static str> {
    let mut v: Vec<&str> = Vec::new();
    for d in DEFS.iter().filter(|d| d.page == page) {
        if !v.contains(&d.section) {
            v.push(d.section);
        }
    }
    v
}

fn modified_count(app: &App, page: Page) -> usize {
    DEFS.iter()
        .filter(|d| d.page == page && modified(app, d.id))
        .count()
}

pub fn show(app: &mut App, ctx: &egui::Context, t: &Theme) {
    if !app.settings_open {
        return;
    }
    let mut cmds: Vec<Command> = Vec::new();
    let mut close = false;
    let t = *t;
    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("apature-settings"),
        egui::ViewportBuilder::default()
            .with_title("Apature Settings")
            .with_inner_size([760.0, 600.0])
            .with_min_inner_size([620.0, 460.0]),
        |ctx, _class| {
            app.apply_pending_edit(ctx, false);
            if ctx.input(|i| i.viewport().close_requested()) {
                close = true;
            }
            // Screenshot support for this window (see main.rs --settings). The
            // viewport only receives the Screenshot event on a frame it paints,
            // so keep it painting while a capture is pending.
            if let Some(path) = app.screenshot_path() {
                ctx.request_repaint();
                let shot = ctx.input(|i| {
                    i.events.iter().find_map(|e| match e {
                        egui::Event::Screenshot { image, .. } => Some(image.clone()),
                        _ => None,
                    })
                });
                if let Some(img) = shot {
                    let [w, h] = img.size;
                    if let Some(buf) =
                        image::RgbaImage::from_raw(w as u32, h as u32, img.as_raw().to_vec())
                    {
                        let _ = buf.save(&path);
                    }
                    ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
                }
            }
            body(app, ctx, &t, &mut cmds);
        },
    );
    if close {
        app.settings_open = false;
    }
    for c in cmds {
        app.dispatch(ctx, c);
    }
}

const SHORTCUTS: &[(&str, &str)] = &[
    (
        "← → ↑ ↓",
        "Move focus (⇧ extends the selection, ⌘ keeps it)",
    ),
    ("X · T", "Reject · toggle tag"),
    ("⏎ / Space · Esc", "Preview · back"),
    (
        "Z · C",
        "Preview: 1:1 at the pointer · compare the selection",
    ),
    (
        "L · ⌘scroll",
        "Preview: loupe under the pointer · magnification 100–400 %",
    ),
    (
        "K · R · G",
        "Preview: crop tool · next ratio · next guides (⏎ apply, ⌫ remove, Esc cancel)",
    ),
    ("B", "Preview: brighten +1/+2 stops, view only"),
    ("⌘[ · ⌘]", "Rotate left · right (sidecar only)"),
    ("S", "Expand or collapse the focused burst stack"),
    ("⌘R", "Refresh the folder, keeping cursor and selection"),
    ("⌘A · ⌘⇧A", "Select all · none"),
    ("⌘Z · ⌘⇧Z", "Undo · redo"),
    ("⌘⇧1 … ⌘⇧5", "Sheet · Preview · Caption · Ingest · Export"),
    ("⌘E", "Export…"),
    ("⌘= · ⌘-", "Larger · smaller thumbnails"),
    ("⌃⌘S · ⌥⌘I · ⇧F", "Sidebar · inspector · filmstrip"),
    ("⌥⌘F", "Clear filters"),
    ("⌘O / ⌘T · ⌘W", "Open folder in a new tab · close tab"),
    ("⌘⇧] · ⌘⇧[", "Next · previous tab"),
    ("⌘⏎", "Save caption and advance"),
    ("⌘,", "Settings"),
];

fn dir_size(dir: &std::path::Path) -> (usize, u64) {
    let mut files = 0;
    let mut bytes = 0;
    for e in walkdir::WalkDir::new(dir).into_iter().flatten() {
        if e.file_type().is_file() {
            files += 1;
            bytes += e.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    (files, bytes)
}

/// The window's content: search on top, rail on the left, one page on the
/// right. Also drawn inline on the main window by `show_inline` for headless
/// screenshots.
fn body(app: &mut App, ctx: &egui::Context, t: &Theme, cmds: &mut Vec<Command>) {
    egui::TopBottomPanel::top("settings_search")
        .frame(
            egui::Frame::NONE
                .fill(t.panel)
                .inner_margin(egui::Margin::symmetric(S3 as i8, S2 as i8)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let r = ui.add(
                    egui::TextEdit::singleline(&mut app.settings_query)
                        .hint_text("Search settings…")
                        .desired_width(ui.available_width() - 60.0),
                );
                if r.changed() && !app.settings_query.is_empty() {
                    // typing leaves the page: results replace it until cleared
                }
                if !app.settings_query.is_empty() && ui.small_button("Clear").clicked() {
                    app.settings_query.clear();
                }
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) && !app.settings_query.is_empty()
                {
                    app.settings_query.clear();
                }
                let _ = r;
            });
        });
    egui::SidePanel::left("settings_nav")
        .exact_width(170.0)
        .frame(
            egui::Frame::NONE
                .fill(t.panel)
                .inner_margin(egui::Margin::same(S2 as i8)),
        )
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            for (page, label) in PAGES {
                let on = app.settings_page == page && app.settings_query.is_empty();
                let n = modified_count(app, page);
                let (rect, resp) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 28.0),
                    egui::Sense::click(),
                );
                let p = ui.painter();
                if on {
                    p.rect_filled(rect, egui::CornerRadius::same(6), t.accent_wash);
                } else if resp.hovered() {
                    p.rect_filled(rect, egui::CornerRadius::same(6), t.raised);
                }
                p.text(
                    egui::pos2(rect.left() + S3, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(13.0),
                    if on { t.accent } else { t.text2 },
                );
                if n > 0 {
                    let badge = egui::Rect::from_center_size(
                        egui::pos2(rect.right() - 16.0, rect.center().y),
                        egui::vec2(20.0, 16.0),
                    );
                    p.rect_stroke(
                        badge,
                        egui::CornerRadius::same(8),
                        egui::Stroke::new(1.0, t.accent),
                        egui::StrokeKind::Inside,
                    );
                    p.text(
                        badge.center(),
                        egui::Align2::CENTER_CENTER,
                        n.to_string(),
                        egui::FontId::proportional(10.0),
                        t.accent,
                    );
                }
                if resp.clicked() {
                    app.settings_page = page;
                    app.settings_query.clear();
                }
            }
        });
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(t.window).inner_margin(egui::Margin {
            left: S4 as i8 + 4,
            right: S4 as i8,
            top: S3 as i8,
            bottom: S3 as i8,
        }))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = S2;
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if !app.settings_query.is_empty() {
                        search_results(app, ui, t);
                    } else {
                        page_body(app, ui, t, cmds);
                    }
                });
        });
}

fn search_results(app: &mut App, ui: &mut egui::Ui, t: &Theme) {
    let q = app.settings_query.to_lowercase();
    let words: Vec<&str> = q.split_whitespace().collect();
    let hit = |hay: &str| words.iter().all(|w| hay.to_lowercase().contains(w));
    let defs: Vec<&Def> = DEFS
        .iter()
        .filter(|d| hit(&format!("{} {} {}", d.label, d.description, d.section)))
        .collect();
    let panels: Vec<&(Page, &str, &str)> = PANELS
        .iter()
        .filter(|(p, s, k)| hit(&format!("{} {} {}", page_label(*p), s, k)))
        .collect();
    heading(ui, t, &format!("Results for “{}”", app.settings_query));
    if defs.is_empty() && panels.is_empty() {
        ui.add_space(S2);
        hint(
            ui,
            t,
            "No settings match. Try another word, or pick a category on the left.",
        );
        return;
    }
    if !defs.is_empty() {
        let n = defs.len();
        card(ui, t, "Settings", |ui| {
            for (i, d) in defs.iter().enumerate() {
                setting_row(
                    app,
                    ui,
                    t,
                    d,
                    Some(format!("{} › {}", page_label(d.page), d.section)),
                    i + 1 == n,
                );
            }
        });
    }
    if !panels.is_empty() {
        card(ui, t, "Elsewhere", |ui| {
            for (page, section, _) in panels {
                ui.horizontal(|ui| {
                    ui.add_space(S1);
                    ui.label(RichText::new(*section).size(13.0).color(t.text));
                    ui.label(
                        RichText::new(format!("in {}", page_label(*page)))
                            .size(11.0)
                            .color(t.muted),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Open").clicked() {
                            app.settings_page = *page;
                            app.settings_query.clear();
                        }
                    });
                });
                ui.add_space(S1);
            }
        });
    }
}

fn page_body(app: &mut App, ui: &mut egui::Ui, t: &Theme, cmds: &mut Vec<Command>) {
    let page = app.settings_page;
    ui.horizontal(|ui| {
        heading(ui, t, page_label(page));
        let n = modified_count(app, page);
        if n > 0 {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Reset this page").clicked() {
                    for d in DEFS.iter().filter(|d| d.page == page) {
                        reset(app, d.id);
                    }
                }
                ui.label(
                    RichText::new(format!(
                        "{n} differ{} from the default",
                        if n == 1 { "s" } else { "" }
                    ))
                    .size(11.0)
                    .color(t.muted),
                );
            });
        }
    });
    match page {
        Page::General => {
            for s in sections_of(page) {
                section_card(app, ui, t, page, s);
                if s == "Learning the loop" {
                    ui.horizontal(|ui| {
                        ui.add_space(S1);
                        if ui.small_button("Show the welcome again").clicked() {
                            app.welcome_open = true;
                            app.welcome_step = 0;
                            app.settings_open = false;
                        }
                        if !app.prefs.tips_seen.is_empty()
                            && ui.small_button("Reset tips").clicked()
                        {
                            app.prefs.tips_seen.clear();
                            app.prefs_dirty = true;
                        }
                    });
                }
            }
            card(ui, t, "Edit photos in", |ui| {
                ui.add_space(S2);
                let detected = app.editor_app();
                ui.horizontal(|ui| {
                    ui.label(RichText::new("APP").monospace().size(9.5).color(t.muted));
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut app.prefs.editor_app)
                            .hint_text(if detected.is_empty() {
                                "system default".to_string()
                            } else {
                                format!("{detected} (found)")
                            })
                            .desired_width(240.0),
                    );
                    if r.changed() {
                        app.prefs_dirty = true;
                    }
                    if ui.small_button("Choose…").clicked() {
                        if let Some(p) = rfd::FileDialog::new()
                            .set_title("Choose the editing app")
                            .pick_file()
                        {
                            app.prefs.editor_app = p.display().to_string();
                            app.prefs_dirty = true;
                        }
                    }
                });
                ui.add_space(S1);
                if ui
                    .checkbox(
                        &mut app.prefs.editor_with_jpeg,
                        "Send a RAW's JPEG partner too",
                    )
                    .changed()
                {
                    app.prefs_dirty = true;
                }
                ui.add_space(S1);
                hint(
                    ui,
                    t,
                    "Image › Edit Photos (or right-click) opens the selected photos there. The editor reads the rating, colour label, caption and crop from the XMP sidecar beside each RAW. In Capture One: Preferences › Image › Metadata, set Auto Sync Sidecar XMP to Load and tick Prefer Sidecar XMP over Embedded Metadata; Image › Load Metadata refreshes later changes.",
                );
                ui.add_space(S2);
            });
            card(ui, t, "Files Apature keeps", |ui| {
                ui.add_space(S2);
                ui.horizontal_wrapped(|ui| {
                    if ui.small_button(format!("Reveal settings in {}", crate::platform::Platform::file_manager())).clicked() {
                        if let Some(p) = Prefs::path() {
                            crate::platform::Platform::reveal(&p);
                        }
                    }
                    if ui.small_button("Reveal log").on_hover_text("apature.log: every ingest, export, send and error").clicked() {
                        if let Some(d) = directories::ProjectDirs::from("dev", "mccal", "Apature") {
                            crate::platform::Platform::reveal(&d.data_dir().join("apature.log"));
                        }
                    }
                    if ui.small_button("Reset all settings").on_hover_text("Keeps a copy as prefs.json.bak-reset; delivery passwords stay in the system store").clicked() {
                        if let Some(p) = Prefs::path() {
                            let _ = std::fs::copy(&p, p.with_extension("json.bak-reset"));
                        }
                        app.prefs = Prefs::default();
                        app.prefs_dirty = true;
                    }
                });
                ui.add_space(S1);
                hint(
                    ui,
                    t,
                    &format!(
                        "Settings file version {} · {}",
                        crate::prefs::PREFS_VERSION,
                        Prefs::path()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default()
                    ),
                );
                ui.add_space(S2);
            });
        }
        Page::Classes => {
            hint(ui, t, "Names are written into xmp:Label, so they should match your Photo Mechanic preferences for labels to round-trip. The numeric class is always written too.");
            let mut changed = false;
            card(ui, t, "Classes 1–8", |ui| {
                ui.add_space(S2);
                egui::Grid::new("classes")
                    .num_columns(3)
                    .spacing([S3, S2])
                    .show(ui, |ui| {
                        for i in 0..8 {
                            ui.label(
                                RichText::new(format!("{}", i + 1))
                                    .monospace()
                                    .color(t.muted),
                            );
                            let mut rgb = app.prefs.scheme.colors[i];
                            if ui.color_edit_button_srgb(&mut rgb).changed() {
                                app.prefs.scheme.colors[i] = rgb;
                                changed = true;
                            }
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut app.prefs.scheme.names[i])
                                        .desired_width(240.0),
                                )
                                .changed()
                            {
                                changed = true;
                            }
                            ui.end_row();
                        }
                    });
                ui.add_space(S2);
                if ui
                    .small_button("Reset to Photo Mechanic names and colours")
                    .clicked()
                {
                    app.prefs.scheme = ColorClassScheme::default();
                    changed = true;
                }
                ui.add_space(S2);
            });
            if changed {
                app.scheme_changed();
            }
        }
        Page::Metadata => {
            card(ui, t, "Assignment file", |ui| {
                ui.add_space(S2);
                hint(
                    ui,
                    t,
                    "An assignment file fills the caption pad and the code list for a shoot.",
                );
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(
                                app.prefs
                                    .assignment_file
                                    .as_ref()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_else(|| "none".into()),
                            )
                            .monospace()
                            .size(10.5)
                            .color(t.text2),
                        )
                        .truncate(),
                    );
                    if ui.small_button("Choose…").clicked() {
                        cmds.push(Command::LoadAssignment);
                    }
                    if app.prefs.assignment_file.is_some() && ui.small_button("Clear").clicked() {
                        app.prefs.assignment_file = None;
                        app.assignment = None;
                        app.prefs_dirty = true;
                    }
                });
                ui.add_space(S2);
            });
            card(ui, t, "Assignments folder", |ui| {
                ui.add_space(S2);
                hint(ui, t, "Where assignment files are kept and where New assignment writes them. Put it on Google Drive and the sheet you make at the desk is on the laptop at the game.");
                ui.horizontal(|ui| {
                    let shown = app
                        .assignments_dir()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "not set".into());
                    ui.add(
                        egui::Label::new(
                            RichText::new(shown).monospace().size(10.5).color(t.text2),
                        )
                        .truncate(),
                    );
                    if ui.small_button("Choose…").clicked() {
                        if let Some(p) = rfd::FileDialog::new().pick_folder() {
                            app.prefs.assignments_dir = Some(p);
                            app.prefs_dirty = true;
                        }
                    }
                    if app.prefs.assignments_dir.is_some()
                        && ui.small_button("Use Drive default").clicked()
                    {
                        app.prefs.assignments_dir = None;
                        app.prefs_dirty = true;
                    }
                });
                ui.add_space(S2);
            });
            card(ui, t, "Outlet", |ui| {
                ui.add_space(S2);
                hint(
                    ui,
                    t,
                    "Written into every new assignment file: who shot it and who it is for.",
                );
                let mut changed = false;
                egui::Grid::new("outlet")
                    .num_columns(2)
                    .spacing([S3, S1])
                    .show(ui, |ui| {
                        for (label, v, h) in [
                            (
                                "Photographer",
                                &mut app.prefs.outlet.photographer,
                                "Caleb McCartney/Special to the Eagle",
                            ),
                            (
                                "Credit",
                                &mut app.prefs.outlet.credit,
                                "Photog/Special to the Eagle",
                            ),
                            ("Source", &mut app.prefs.outlet.source, "the newspaper"),
                            (
                                "Copyright",
                                &mut app.prefs.outlet.copyright,
                                "© 2026 the newspaper",
                            ),
                        ] {
                            ui.label(RichText::new(label).size(12.0).color(t.text2));
                            if ui
                                .add(
                                    egui::TextEdit::singleline(v)
                                        .desired_width(320.0)
                                        .hint_text(h),
                                )
                                .changed()
                            {
                                changed = true;
                            }
                            ui.end_row();
                        }
                    });
                if changed {
                    app.prefs_dirty = true;
                }
                ui.add_space(S2);
            });
            card(ui, t, "Code replacement file", |ui| {
                ui.add_space(S2);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(
                                app.prefs
                                    .codes_file
                                    .as_ref()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_else(|| "none".into()),
                            )
                            .monospace()
                            .size(10.5)
                            .color(t.text2),
                        )
                        .truncate(),
                    );
                    if ui.small_button("Choose…").clicked() {
                        cmds.push(Command::LoadCodes);
                    }
                    if app.prefs.codes_file.is_some() && ui.small_button("Clear").clicked() {
                        app.prefs.codes_file = None;
                        app.prefs_dirty = true;
                    }
                });
                hint(ui, t, &format!("{} codes loaded.", app.codes.codes.len()));
                ui.add_space(S2);
            });
            section_card(app, ui, t, page, "Code replacements");
            let mut vars_changed = false;
            let mut remove: Option<usize> = None;
            card(ui, t, "Your variables", |ui| {
                ui.add_space(S2);
                hint(ui, t, "Use {name} in any template or caption, next to Photo Mechanic's own variables like {iptcmonthnameap}, {city} or {credit}.");
                ui.add_space(S1);
                egui::Grid::new("user-vars")
                    .num_columns(3)
                    .spacing([S3, S1])
                    .show(ui, |ui| {
                        for (i, (name, value)) in app.prefs.variables.iter_mut().enumerate() {
                            if ui
                                .add(
                                    egui::TextEdit::singleline(name)
                                        .desired_width(140.0)
                                        .hint_text("name")
                                        .font(egui::TextStyle::Monospace),
                                )
                                .changed()
                            {
                                vars_changed = true;
                            }
                            if ui
                                .add(
                                    egui::TextEdit::singleline(value)
                                        .desired_width(300.0)
                                        .hint_text("value"),
                                )
                                .changed()
                            {
                                vars_changed = true;
                            }
                            if ui.small_button("Remove").clicked() {
                                remove = Some(i);
                            }
                            ui.end_row();
                        }
                    });
                if ui.small_button("Add a variable").clicked() {
                    app.prefs.variables.push((String::new(), String::new()));
                    vars_changed = true;
                }
                ui.add_space(S2);
            });
            if let Some(i) = remove {
                app.prefs.variables.remove(i);
                vars_changed = true;
            }
            if vars_changed {
                apature_metadata::codes::set_user_vars(app.prefs.variables.clone());
                app.prefs_dirty = true;
            }
            card(ui, t, "Photo Mechanic", |ui| {
                ui.add_space(S2);
                hint(ui, t, "Bring your Photo Mechanic setup over: an ingest snapshot (.snap) sets the folder and rename templates; a code replacement file loads as is. The stationery pad comes later.");
                if ui.small_button("Import from Photo Mechanic…").clicked() {
                    cmds.push(Command::ImportPhotoMechanic);
                }
                ui.add_space(S2);
            });
        }
        Page::Ingest => {
            card(ui, t, "Camera over Wi-Fi", |ui| {
                ui.add_space(S2);
                hint(ui, t, "The folder Nikon's Wireless Transmitter Utility saves into. It is listed under Drives & cards; one click opens Ingest on it with Live Ingest on.");
                ui.horizontal(|ui| {
                    let shown = app
                        .wireless_inbox()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| {
                            "not set, and the utility's default folder does not exist yet".into()
                        });
                    ui.add(
                        egui::Label::new(
                            RichText::new(shown).monospace().size(10.5).color(t.text2),
                        )
                        .truncate(),
                    );
                    if ui.small_button("Choose…").clicked() {
                        if let Some(p) = rfd::FileDialog::new().pick_folder() {
                            app.prefs.wireless_inbox = Some(p);
                            app.prefs_dirty = true;
                            app.volumes_checked =
                                std::time::Instant::now() - std::time::Duration::from_secs(5);
                        }
                    }
                    if app.prefs.wireless_inbox.is_some()
                        && ui.small_button("Use default").clicked()
                    {
                        app.prefs.wireless_inbox = None;
                        app.prefs_dirty = true;
                    }
                    if ui.small_button("Forget pairing").on_hover_text("Drops the identity the camera paired with; the camera will ask to pair again").clicked() {
                        cmds.push(Command::CameraForget);
                    }
                });
                ui.add_space(S2);
            });
            for s in sections_of(page) {
                section_card(app, ui, t, page, s);
                if s == "Sequence" && !app.prefs.seq_counters.is_empty() {
                    ui.horizontal(|ui| {
                        ui.add_space(S1);
                        hint(
                            ui,
                            t,
                            &format!(
                                "{} headline counter(s) remembered.",
                                app.prefs.seq_counters.len()
                            ),
                        );
                        if ui.small_button("Forget all counters").clicked() {
                            app.prefs.seq_counters.clear();
                            app.prefs_dirty = true;
                        }
                    });
                }
            }
        }
        Page::Accessibility => {
            for s in sections_of(page) {
                section_card(app, ui, t, page, s);
            }
            ui.add_space(S2);
            hint(ui, t, "Keyboard: every workspace is reachable with ⌘⇧1–5, the sheet with arrows, marks with the number keys (see Shortcuts). Screen reader labels are still being filled in (see docs/ACCESSIBILITY.md).");
        }
        Page::Delivery => delivery_page(app, ui, t, cmds),
        Page::Cache => {
            section_card(app, ui, t, page, "Diagnostics");
            let dir = app.cache_dir();
            card(ui, t, "Thumbnail cache", |ui| {
                ui.add_space(S2);
                ui.add(
                    egui::Label::new(
                        RichText::new(dir.display().to_string())
                            .monospace()
                            .size(10.5)
                            .color(t.text2),
                    )
                    .truncate(),
                );
                ui.horizontal(|ui| {
                    if ui.small_button("Calculate size").clicked() {
                        app.cache_size = Some(dir_size(&dir));
                    }
                    if let Some((files, bytes)) = app.cache_size {
                        ui.label(
                            RichText::new(format!(
                                "{files} thumbnails · {:.0} MB",
                                bytes as f64 / 1e6
                            ))
                            .size(12.0)
                            .color(t.text2),
                        );
                    }
                    if ui
                        .small_button("Clear thumbnail cache")
                        .on_hover_text("Thumbnails are regenerated from the photos as needed")
                        .clicked()
                    {
                        cmds.push(Command::ClearThumbCache);
                    }
                });
                hint(
                    ui,
                    t,
                    "Originals are never touched; the cache only holds 320 px JPEG thumbnails.",
                );
                ui.add_space(S2);
            });
        }
        Page::Resources => resources_page(ui, t),
        Page::Shortcuts => {
            section_card(app, ui, t, page, "Number keys");
            let (num_keys, num_row, cmd_keys, cmd_row): (&str, &str, &str, &str) =
                match app.prefs.number_keys {
                    NumberKeys::Rating => (
                        "1–5 · 0",
                        "Rating · clear",
                        "⌘1–⌘8 · ⌘0",
                        "Colour class · clear",
                    ),
                    NumberKeys::ColorClass => {
                        ("1–8 · 0", "Colour class · clear", "⌘0–⌘5", "Rating · clear")
                    }
                };
            card(ui, t, "All shortcuts", |ui| {
                ui.add_space(S2);
                egui::Grid::new("keys")
                    .num_columns(2)
                    .spacing([S4, S2])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new(num_keys).monospace().size(11.5).color(t.text));
                        ui.label(RichText::new(num_row).size(12.0).color(t.text2));
                        ui.end_row();
                        ui.label(RichText::new(cmd_keys).monospace().size(11.5).color(t.text));
                        ui.label(RichText::new(cmd_row).size(12.0).color(t.text2));
                        ui.end_row();
                        for (k, v) in SHORTCUTS {
                            ui.label(RichText::new(*k).monospace().size(11.5).color(t.text));
                            ui.label(RichText::new(*v).size(12.0).color(t.text2));
                            ui.end_row();
                        }
                    });
                ui.add_space(S2);
                hint(ui, t, "Editable profiles arrive later.");
                ui.add_space(S2);
            });
        }
    }
}

fn delivery_page(app: &mut App, ui: &mut egui::Ui, t: &Theme, cmds: &mut Vec<Command>) {
    hint(ui, t, &format!("Where finished exports go. Passwords are kept in {store}, never in the prefs file. Each file lands as a hidden .part name and is renamed when complete.", store = crate::platform::Platform::secret_store()));
    let mut remove: Option<usize> = None;
    let mut rename: Option<(String, String)> = None;
    let n = app.prefs.delivery.len();
    for i in 0..n {
        let (old_name, title) = {
            let d = &app.prefs.delivery[i];
            (
                d.name.clone(),
                format!("{}  ·  {:?} {}:{}", d.name, d.kind, d.host, d.port),
            )
        };
        ui.add_space(S2);
        egui::Frame::NONE
            .fill(t.panel)
            .stroke(egui::Stroke::new(1.0, t.line))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(S3 as i8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                egui::CollapsingHeader::new(RichText::new(title).size(12.5).color(t.text)).id_salt(("deliv", i)).default_open(n == 1).show(ui, |ui| {
                    let d = &mut app.prefs.delivery[i];
                    egui::Grid::new(("deliv_grid", i)).num_columns(2).spacing([S3, S2]).show(ui, |ui| {
                        ui.label(RichText::new("Name").size(12.0).color(t.text2));
                        let mut nm = d.name.clone();
                        if ui.add(egui::TextEdit::singleline(&mut nm).desired_width(260.0)).lost_focus() && nm.trim() != d.name && !nm.trim().is_empty() {
                            rename = Some((d.name.clone(), nm.trim().to_string()));
                        }
                        ui.end_row();
                        ui.label(RichText::new("Protocol").size(12.0).color(t.text2));
                        ui.horizontal(|ui| {
                            for (k, label) in [(apature_deliver::Kind::Folder, "Folder"), (apature_deliver::Kind::Ftp, "FTP"), (apature_deliver::Kind::Ftps, "FTPS (TLS)"), (apature_deliver::Kind::Sftp, "SFTP")] {
                                if super::icons::chip(ui, t, label, d.kind == k).clicked() {
                                    d.kind = k;
                                    d.port = apature_deliver::Target::default_port(k);
                                    app.prefs_dirty = true;
                                }
                            }
                        });
                        ui.end_row();
                        let is_folder = d.kind == apature_deliver::Kind::Folder;
                        if !is_folder {
                            ui.label(RichText::new("Host").size(12.0).color(t.text2));
                            if ui.add(egui::TextEdit::singleline(&mut d.host).desired_width(260.0).hint_text("ftp.example.com or 192.168.1.20")).changed() { app.prefs_dirty = true; }
                            ui.end_row();
                            ui.label(RichText::new("Port").size(12.0).color(t.text2));
                            if ui.add(egui::DragValue::new(&mut d.port).range(1..=65535)).changed() { app.prefs_dirty = true; }
                            ui.end_row();
                            ui.label(RichText::new("User").size(12.0).color(t.text2));
                            if ui.add(egui::TextEdit::singleline(&mut d.user).desired_width(260.0)).changed() { app.prefs_dirty = true; }
                            ui.end_row();
                        }
                        ui.label(RichText::new("Folder").size(12.0).color(t.text2));
                        ui.horizontal(|ui| {
                            let hint_text = if is_folder { "a mounted share, or the Google Drive / OneDrive / Dropbox sync folder" } else { "incoming/photos (empty = login folder)" };
                            if ui.add(egui::TextEdit::singleline(&mut d.dir).desired_width(if is_folder { 200.0 } else { 260.0 }).hint_text(hint_text)).changed() { app.prefs_dirty = true; }
                            if is_folder && ui.small_button("Choose…").clicked() {
                                if let Some(p) = rfd::FileDialog::new().pick_folder() { d.dir = p.display().to_string(); app.prefs_dirty = true; }
                            }
                        });
                        ui.end_row();
                        if is_folder {
                            ui.label(RichText::new("").size(12.0));
                            ui.label(RichText::new("No sign-in needed: the Google Drive or OneDrive desktop app syncs this folder to the cloud.").size(11.0).color(t.muted));
                            ui.end_row();
                        }
                        if d.kind == apature_deliver::Kind::Sftp {
                            ui.label(RichText::new("Key file").size(12.0).color(t.text2));
                            ui.horizontal(|ui| {
                                let shown = d.key_file.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "password login".into());
                                ui.label(RichText::new(shown).monospace().size(10.5).color(t.text2));
                                if ui.small_button("Choose…").clicked() {
                                    if let Some(p) = rfd::FileDialog::new().pick_file() { d.key_file = Some(p); app.prefs_dirty = true; }
                                }
                                if d.key_file.is_some() && ui.small_button("Use password").clicked() { d.key_file = None; app.prefs_dirty = true; }
                            });
                            ui.end_row();
                        }
                        if !is_folder {
                            ui.label(RichText::new(if d.key_file.is_some() { "Passphrase" } else { "Password" }).size(12.0).color(t.text2));
                            ui.horizontal(|ui| {
                                let draft = app.deliver.pw_drafts.entry(old_name.clone()).or_default();
                                ui.add(egui::TextEdit::singleline(draft).password(true).desired_width(180.0));
                                let has = crate::platform::Platform::secret_get(&old_name).is_some();
                                if ui.small_button(if cfg!(target_os = "macos") { "Save to Keychain" } else { "Save password" }).clicked() {
                                    match crate::platform::Platform::secret_set(&old_name, draft) {
                                        Ok(()) => { draft.clear(); app.deliver.test_result = Some(format!("{old_name}: password saved")); }
                                        Err(e) => app.deliver.test_result = Some(format!("{old_name}: {e}")),
                                    }
                                }
                                ui.label(RichText::new(if has { "saved" } else { "none saved" }).size(10.5).color(t.muted));
                            });
                            ui.end_row();
                        }
                    });
                    ui.add_space(S1);
                    ui.horizontal(|ui| {
                        if ui.small_button("Test connection").clicked() {
                            cmds.push(Command::DeliverTest(old_name.clone()));
                        }
                        if ui.small_button("Remove").clicked() {
                            remove = Some(i);
                        }
                    });
                });
            });
    }
    if let Some((from, to)) = rename {
        if let Some(d) = app.prefs.delivery.iter_mut().find(|d| d.name == from) {
            d.name = to.clone();
        }
        if let Some(pw) = crate::platform::Platform::secret_get(&from) {
            let _ = crate::platform::Platform::secret_set(&to, &pw);
            crate::platform::Platform::secret_delete(&from);
        }
        if app.prefs.send_after_export.as_deref() == Some(from.as_str()) {
            app.prefs.send_after_export = Some(to);
        }
        app.prefs_dirty = true;
    }
    if let Some(i) = remove {
        let d = app.prefs.delivery.remove(i);
        crate::platform::Platform::secret_delete(&d.name);
        if app.prefs.send_after_export.as_deref() == Some(d.name.as_str()) {
            app.prefs.send_after_export = None;
        }
        app.prefs_dirty = true;
    }
    ui.add_space(S2);
    if ui.button("Add a target").clicked() {
        let k = apature_deliver::Kind::Sftp;
        app.prefs.delivery.push(apature_deliver::Target {
            name: format!("Desk {}", app.prefs.delivery.len() + 1),
            kind: k,
            host: String::new(),
            port: apature_deliver::Target::default_port(k),
            user: String::new(),
            dir: String::new(),
            key_file: None,
        });
        app.prefs_dirty = true;
    }
    if let Some(r) = &app.deliver.test_result {
        ui.add_space(S2);
        ui.label(RichText::new(r).size(12.0).color(
            if r.contains("connected") || r.contains("saved") {
                t.text
            } else {
                t.reject
            },
        ));
    }
    let _ = Color32::TRANSPARENT;
}

/// Headless screenshot path: draw the Settings content on the root window
/// (the separate viewport cannot be captured by the harness).
pub fn show_inline(app: &mut App, ctx: &egui::Context, t: &Theme) {
    let mut cmds: Vec<Command> = Vec::new();
    let t = *t;
    body(app, ctx, &t, &mut cmds);
    for c in cmds {
        app.dispatch(ctx, c);
    }
}

/// Links for photojournalists, students first: where the style and the
/// ethics come from, and where the internships are. Curated by hand; every
/// address was checked when it was added. Opens in the browser.
/// (name, one line on what it is, address)
type Link = (&'static str, &'static str, &'static str);

const RESOURCES: &[(&str, &[Link])] = &[
    (
        "Style and ethics",
        &[
            ("AP Stylebook", "The newsroom style every caption follows: names, titles, dates, numbers.", "https://www.apstylebook.com/"),
            ("NPPA resources", "The National Press Photographers Association: its Code of Ethics, advocacy and member resources.", "https://nppa.org/resources"),
            ("SPJ Code of Ethics", "Seek truth, minimise harm, act independently, be accountable. The baseline for every newsroom.", "https://www.spj.org/ethicscode.asp"),
            ("Poynter", "Reporting, ethics and craft from the Poynter Institute, with free courses.", "https://www.poynter.org/"),
        ],
    ),
    (
        "Internships, workshops and contests",
        &[
            ("Dow Jones News Fund", "Paid summer internships for students, with training before the newsroom.", "https://www.dowjonesnewsfund.org/"),
            ("Chips Quinn Scholars", "The Freedom Forum's fellowship for student journalists of colour, with a newsroom placement.", "https://www.freedomforum.org/chips-quinn-reporter-fellowship/"),
            ("The New York Times Student Journalism Institute", "A free, intensive newsroom for student journalists, photography included.", "https://nytimes-institute.com/"),
            ("AP internships", "The Associated Press's internship and early-career programmes.", "https://www.ap.org/careers/internships/"),
            ("Eddie Adams Workshop", "Four days, tuition-free, for 100 emerging photographers chosen by portfolio.", "https://eddieadamsworkshop.org/"),
            ("Missouri Photo Workshop", "A week documenting a small town with working editors, since 1949.", "https://mophotoworkshop.org/"),
            ("College Photographer of the Year", "The student contest run by the Missouri School of Journalism.", "https://cpoy.org/"),
            ("World Press Photo", "The contest, the exhibition and the education programmes.", "https://www.worldpressphoto.org/"),
            ("NPPF", "The National Press Photographers Foundation: scholarships for students in photojournalism.", "https://nppf.org/"),
        ],
    ),
];

fn resources_page(ui: &mut egui::Ui, t: &Theme) {
    hint(ui, t, "The industry is competitive and nobody hands you the map. These are the places to start; send a note on GitHub if one should be here and is not.");
    for (section, links) in RESOURCES {
        card(ui, t, section, |ui| {
            let n = links.len();
            for (i, (name, what, url)) in links.iter().enumerate() {
                ui.add_space(S2);
                ui.horizontal_top(|ui| {
                    ui.add_space(S1);
                    ui.vertical(|ui| {
                        ui.set_max_width((ui.available_width() - 90.0).max(160.0));
                        ui.label(RichText::new(*name).size(13.0).color(t.text));
                        ui.add(
                            egui::Label::new(RichText::new(*what).size(11.0).color(t.muted)).wrap(),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.add_space(S1);
                        if ui.small_button("Open").on_hover_text(*url).clicked() {
                            crate::platform::Platform::open_url(url);
                        }
                    });
                });
                ui.add_space(S2);
                if i + 1 < n {
                    let r = ui.available_rect_before_wrap();
                    ui.painter().hline(
                        r.x_range(),
                        ui.cursor().top(),
                        egui::Stroke::new(1.0, t.line),
                    );
                }
            }
        });
    }
}
