//! Assignments made ahead of time: a folder of assignment files (on a synced
//! drive if you like), a list of the upcoming ones in the Caption workspace,
//! and a "New assignment" sheet that takes the editor's assignment sheet as
//! pasted and turns it into an Apature assignment file, with the outlet's
//! credit, source and copyright filled from Settings. When an ingest for an
//! assignment finishes, the file is copied into the day's folder, so the
//! shoot carries its own sheet onto the archive drive.

use std::path::{Path, PathBuf};

use egui::RichText;

use super::tokens::{Theme, S1, S2, S3, S4};
use crate::app::App;
use crate::commands::Command;

/// One file in the assignments folder.
#[derive(Clone, Debug)]
pub struct Listed {
    pub path: PathBuf,
    pub date: String,
    pub slug: String,
}

/// Every `.txt` assignment in `dir`, newest date first.
pub fn list(dir: &Path) -> Vec<Listed> {
    let mut v = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return v;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("txt") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&p) else {
            continue;
        };
        let mut date = String::new();
        let mut slug = String::new();
        for line in text.lines() {
            let l = line.trim();
            if let Some(v) = l.strip_prefix("date:") {
                date = v.trim().to_string();
            } else if let Some(v) = l.strip_prefix("slug:") {
                slug = v.trim().to_string();
            }
        }
        if slug.is_empty() {
            slug = p
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
        }
        v.push(Listed {
            path: p,
            date,
            slug,
        });
    }
    v.sort_by(|a, b| b.date.cmp(&a.date).then(a.slug.cmp(&b.slug)));
    v
}

/// What the New assignment sheet edits.
#[derive(Clone, Debug, Default)]
pub struct Draft {
    pub pasted: String,
    pub slug: String,
    pub date: String,
    pub what: String,
    pub venue: String,
    pub city: String,
    pub state: String,
    pub team: String,
    pub keywords: String,
    pub ending: String,
    pub notes: String,
}

/// Pull what can be pulled from an editor's sheet. Lines like `WHAT:`,
/// `WHERE:`, `WHEN:`, `ADDRESS:`, `CAPTION ENDING:` are read; the first line
/// that looks like `MMDDYY SEC SLUG` becomes the slug and the date; anything
/// else is kept as notes. Fields already typed are not overwritten.
pub fn fill_from_sheet(d: &mut Draft) {
    let text = d.pasted.clone();
    let mut notes = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let upper = line.to_ascii_uppercase();
        let after = |p: &str| line[p.len()..].trim().to_string();
        if d.slug.is_empty() && looks_like_slug(line) {
            d.slug = line
                .split(['&', '('].as_ref())
                .next()
                .unwrap_or(line)
                .trim()
                .to_string();
            if d.date.is_empty() {
                d.date = date_from_slug(&d.slug).unwrap_or_default();
            }
        } else if upper.starts_with("WHAT:") && d.what.is_empty() {
            d.what = after("WHAT:");
        } else if (upper.starts_with("WHERE:") || upper.starts_with("ADDRESS:"))
            && d.venue.is_empty()
        {
            let v = if upper.starts_with("WHERE:") {
                after("WHERE:")
            } else {
                after("ADDRESS:")
            };
            // "Venue - detail, street, Town" → venue is the first part, town the last.
            let parts: Vec<&str> = v.split(',').map(|s| s.trim()).collect();
            d.venue = parts
                .first()
                .map(|s| s.split(" - ").next().unwrap_or(s).to_string())
                .unwrap_or_default();
            if d.city.is_empty() && parts.len() > 1 {
                d.city = parts.last().map(|s| s.to_string()).unwrap_or_default();
            }
        } else if upper.starts_with("CAPTION ENDING:") && d.ending.is_empty() {
            d.ending = after("CAPTION ENDING:")
                .trim_start_matches("...")
                .trim()
                .to_string();
        } else if !line.starts_with("http") {
            notes.push(line.to_string());
        }
    }
    if d.notes.is_empty() {
        d.notes = notes.join(" ");
    }
    if d.state.is_empty() {
        d.state = "Pa.".into();
    }
}

fn looks_like_slug(line: &str) -> bool {
    let mut it = line.split_whitespace();
    match (it.next(), it.next()) {
        (Some(a), Some(b)) => {
            a.len() == 6
                && a.chars().all(|c| c.is_ascii_digit())
                && b.len() == 3
                && b.chars().all(|c| c.is_ascii_uppercase())
        }
        _ => false,
    }
}

fn date_from_slug(slug: &str) -> Option<String> {
    let s = slug.split_whitespace().next()?;
    if s.len() != 6 {
        return None;
    }
    let (m, d, y) = (&s[0..2], &s[2..4], &s[4..6]);
    Some(format!("20{y}-{m}-{d}"))
}

/// The assignment file text.
pub fn render(d: &Draft, o: &crate::prefs::Outlet) -> String {
    let mut out = String::new();
    out.push_str("# Assignment. Load in Apature: Caption › Load assignment.\n");
    let put = |out: &mut String, k: &str, v: &str| {
        if !v.trim().is_empty() {
            out.push_str(&format!("{k}: {}\n", v.trim()));
        }
    };
    put(&mut out, "date", &d.date);
    put(&mut out, "slug", &d.slug);
    put(&mut out, "venue", &d.venue);
    put(&mut out, "city", &d.city);
    put(&mut out, "state", &d.state);
    put(&mut out, "country", "USA");
    put(&mut out, "event", &d.what);
    put(&mut out, "keywords", &d.keywords);
    put(&mut out, "team", &d.team);
    put(&mut out, "photographer", &o.photographer);
    put(&mut out, "credit", &o.credit);
    put(&mut out, "source", &o.source);
    put(&mut out, "copyright", &o.copyright);
    if !d.ending.trim().is_empty() {
        out.push_str(&format!("code: tail =  {}\n", d.ending.trim()));
    }
    if !d.notes.trim().is_empty() {
        out.push_str(&format!("# {}\n", d.notes.trim()));
    }
    out
}

/// A safe file name for the slug.
pub fn file_name(slug: &str) -> String {
    let s: String = slug
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':') {
                '-'
            } else {
                c
            }
        })
        .collect();
    format!("{}.txt", s.trim())
}

/// The New assignment sheet, as a modal.
pub fn show(app: &mut App, ctx: &egui::Context, t: &Theme) {
    if !app.assignment_sheet_open {
        return;
    }
    let mut close = false;
    let mut save = false;
    let modal = egui::Modal::new(egui::Id::new("new-assignment")).show(ctx, |ui| {
        ui.set_width(640.0);
        ui.spacing_mut().item_spacing.y = S2;
        ui.label(RichText::new("New assignment").size(18.0).strong().color(t.text));
        ui.label(RichText::new("Paste the editor's sheet, then fix anything it missed. The file goes to your assignments folder and loads now.").size(12.0).color(t.text2));
        let credit_line = if app.prefs.outlet.credit.is_empty() { "not set yet".to_string() } else { app.prefs.outlet.credit.clone() };
        let has_dir = app.assignments_dir().is_some();
        let d = &mut app.assignment_draft;
        super::eyebrow(ui, t, "Paste the sheet");
        let pasted = ui.add(egui::TextEdit::multiline(&mut d.pasted).desired_rows(5).desired_width(f32::INFINITY).hint_text("102526 SPO FB Norwin-SV\nFOOTBALL: Norwin at Seneca Valley, 7\nADDRESS: …\nCAPTION ENDING: …"));
        if pasted.changed() {
            fill_from_sheet(d);
        }
        ui.add_space(S1);
        egui::Grid::new("assignment-fields").num_columns(2).spacing([S3, S1]).show(ui, |ui| {
            for (label, v, hint) in [
                ("Slug", &mut d.slug, "102526 SPO FB Norwin-SV"),
                ("Date", &mut d.date, "2026-10-23"),
                ("What", &mut d.what, "a football game · the Great Pumpkin Festival"),
                ("Venue", &mut d.venue, "Seneca Valley's NexTier Stadium"),
                ("City", &mut d.city, "Jackson Township"),
                ("State", &mut d.state, "Pa."),
                ("Team", &mut d.team, "Seneca Valley's (for the caption template)"),
                ("Keywords", &mut d.keywords, "football; Norwin; Seneca Valley; WPIAL"),
                ("Caption ending", &mut d.ending, "in a football game Friday, Oct. 23, 2026, at … Photog/Special to the Eagle"),
            ] {
                ui.label(RichText::new(label.to_uppercase()).monospace().size(9.5).color(t.muted));
                ui.add(egui::TextEdit::singleline(v).desired_width(460.0).hint_text(hint));
                ui.end_row();
            }
        });
        ui.label(RichText::new(format!("Credit, source and copyright come from Settings › Metadata › Outlet: {credit_line}")).size(10.5).color(t.muted));
        ui.add_space(S4);
        ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
                close = true;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let ok = !d.slug.trim().is_empty() && has_dir;
                if ui.add_enabled(ok, egui::Button::new(RichText::new("Save and load").strong())).on_disabled_hover_text("Needs a slug and an assignments folder (Settings › Metadata)").clicked() {
                    save = true;
                }
            });
        });
    });
    if modal.should_close() {
        close = true;
    }
    if save {
        if let Some(dir) = app.assignments_dir() {
            let _ = std::fs::create_dir_all(&dir);
            let path = dir.join(file_name(&app.assignment_draft.slug));
            let text = render(&app.assignment_draft, &app.prefs.outlet);
            match std::fs::write(&path, text) {
                Ok(()) => {
                    app.dispatch(ctx, Command::LoadAssignmentFrom(path));
                    app.assignment_draft = Draft::default();
                    close = true;
                }
                Err(e) => app.toast(
                    super::toasts::Kind::Error,
                    format!("Could not write the assignment: {e}"),
                ),
            }
        }
    }
    if close {
        app.assignment_sheet_open = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_sports_sheet() {
        let mut d = Draft {
            pasted: "102526 SPO FB Norwin-SV\n\nFOOTBALL: Norwin at Seneca Valley, 7\n\nADDRESS: Seneca Valley HS\n\nCAPTION ENDING: ... in a football game Friday, Oct. 23, 2026, at Seneca Valley's NexTier Stadium. Photog/Special to the Eagle\n\nREPORTER: none\n".into(),
            ..Default::default()
        };
        fill_from_sheet(&mut d);
        assert_eq!(d.slug, "102526 SPO FB Norwin-SV");
        assert_eq!(d.date, "2026-10-25");
        assert_eq!(d.venue, "Seneca Valley HS");
        assert!(d.ending.starts_with("in a football game Friday"));
        let text = render(
            &d,
            &Outlet {
                credit: "Photog/Special to the Eagle".into(),
                ..Default::default()
            },
        );
        assert!(text.contains("slug: 102526 SPO FB Norwin-SV"));
        assert!(text.contains("code: tail =  in a football game"));
        assert!(apature_metadata::stationery::Assignment::parse(&text).is_ok());
    }

    #[test]
    fn reads_a_festival_sheet() {
        let mut d = Draft {
            pasted: "102526 LOC Great Pumpkin Fest & 102726_COM_Self-Made Halloween Costumes\nWHAT: Great Pumpkin Festival\nWHERE: Cranberry Township Municipal Building - Great Armstrong Lawn, 2525 Rochester Road, Cranberry Township\nWHEN: 5:30 p.m. -- 7:30 p.m.\n".into(),
            ..Default::default()
        };
        fill_from_sheet(&mut d);
        assert_eq!(d.slug, "102526 LOC Great Pumpkin Fest");
        assert_eq!(d.what, "Great Pumpkin Festival");
        assert_eq!(d.venue, "Cranberry Township Municipal Building");
        assert_eq!(d.city, "Cranberry Township");
    }
}
