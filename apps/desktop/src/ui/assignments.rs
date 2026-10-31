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
    let mut date_from_slug_line = false;
    for raw in text.lines() {
        let line = strip_list_number(raw.trim());
        if line.is_empty() || is_page_chrome(line) {
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
                date_from_slug_line = !d.date.is_empty();
            }
        } else if let Some(day) = date_in_line(line) {
            // The day of the event beats the slug's run date: a game on Friday
            // can carry Sunday's slug, and the photos are of Friday.
            if d.date.is_empty() || date_from_slug_line {
                d.date = day;
                date_from_slug_line = false;
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
            // "Venue, street, Town, PA 15065" → the town is the part before the state and ZIP.
            let mut parts: Vec<&str> = v.split(',').map(|s| s.trim()).collect();
            if let Some(last) = parts.last().copied() {
                if let Some(st) = state_and_zip(last) {
                    if d.state.is_empty() {
                        d.state = st;
                    }
                    parts.pop();
                }
            }
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
        } else if let Some((sport, matchup)) = sport_line(line) {
            // "FOOTBALL: North Catholic at Highlands, 7 pm"
            if d.what.is_empty() {
                d.what = format!("a {sport} game");
            }
            if d.keywords.is_empty() {
                let teams: Vec<&str> = matchup
                    .split(',')
                    .next()
                    .unwrap_or("")
                    .split(" at ")
                    .map(|t| t.trim())
                    .filter(|t| !t.is_empty())
                    .collect();
                let mut kw = vec![sport.clone()];
                kw.extend(teams.iter().map(|t| t.to_string()));
                d.keywords = kw.join("; ");
            }
            notes.push(line.to_string());
        } else if !line.starts_with("http") {
            notes.push(line.to_string());
        }
    }
    // The caption ending names the place as the desk writes it ("at Highlands
    // Golden Rams Stadium."): better for the IPTC Location than the school's name.
    if let Some(i) = d.ending.rfind(", at ") {
        let place = &d.ending[i + 5..];
        let place = place
            .split(". ")
            .next()
            .unwrap_or(place)
            .trim_end_matches('.');
        if !place.is_empty() && place.len() < 80 {
            d.venue = place.to_string();
        }
    }
    if d.notes.is_empty() {
        d.notes = notes.join(" ");
    }
    if d.state.is_empty() {
        d.state = "Pa.".into();
    }
}

/// "3. 110126 SPO FB …" → "110126 SPO FB …": the list position is not the slug.
fn strip_list_number(line: &str) -> &str {
    let t = line.trim_start();
    let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 && digits <= 3 {
        if let Some(rest) = t[digits..].strip_prefix(". ") {
            return rest.trim_start();
        }
    }
    line
}

/// Labels the scheduling page prints around the sheet (not part of it).
fn is_page_chrome(line: &str) -> bool {
    let l = line.trim();
    matches!(
        l,
        "Plan"
            | "Event"
            | "Sharing"
            | "Details"
            | "Assignment"
            | "Show Less"
            | "Show More"
            | "Plan missing assignments"
    ) || l.starts_with("Type Plan")
        || l.starts_with("Created ")
        || l.starts_with("Updated ")
}

/// "PA 15065" → "Pa."
fn state_and_zip(part: &str) -> Option<String> {
    let mut it = part.split_whitespace();
    let (st, zip) = (it.next()?, it.next()?);
    if it.next().is_some()
        || st.len() != 2
        || !st.chars().all(|c| c.is_ascii_uppercase())
        || zip.len() != 5
        || !zip.chars().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    Some(
        match st {
            "PA" => "Pa.",
            "OH" => "Ohio",
            "WV" => "W.Va.",
            "NY" => "N.Y.",
            "MD" => "Md.",
            "VA" => "Va.",
            "NJ" => "N.J.",
            other => other,
        }
        .to_string(),
    )
}

/// "Friday October 30 2026 19:00 - …" → "2026-10-30".
fn date_in_line(line: &str) -> Option<String> {
    const MONTHS: [&str; 12] = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    let toks: Vec<&str> = line.split_whitespace().collect();
    for (i, t) in toks.iter().enumerate() {
        let Some(m) = MONTHS
            .iter()
            .position(|n| n.eq_ignore_ascii_case(t.trim_end_matches(',')))
        else {
            continue;
        };
        let day: u32 = toks.get(i + 1)?.trim_end_matches(',').parse().ok()?;
        let year: i32 = toks.get(i + 2)?.trim_end_matches(',').parse().ok()?;
        if (1..=31).contains(&day) && (2000..2100).contains(&year) {
            return Some(format!("{year}-{:02}-{day:02}", m + 1));
        }
    }
    None
}

/// "FOOTBALL: North Catholic at Highlands, 7 pm" → ("football", "North Catholic at Highlands, 7 pm").
fn sport_line(line: &str) -> Option<(String, String)> {
    let (k, v) = line.split_once(':')?;
    let k = k.trim();
    let known = [
        "FOOTBALL",
        "SOCCER",
        "BASKETBALL",
        "BASEBALL",
        "SOFTBALL",
        "VOLLEYBALL",
        "HOCKEY",
        "LACROSSE",
        "WRESTLING",
        "GOLF",
        "TENNIS",
        "SWIMMING",
        "TRACK",
        "CROSS COUNTRY",
    ];
    known
        .contains(&k)
        .then(|| (k.to_ascii_lowercase(), v.trim().to_string()))
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
    use crate::prefs::Outlet;

    #[test]
    fn reads_a_sports_sheet() {
        let mut d = Draft {
            pasted: "102526 SPO FB Norwin-SV\n\nFOOTBALL: Norwin at Seneca Valley, 7\n\nADDRESS: Seneca Valley HS\n\nCAPTION ENDING: ... in a football game Friday, Oct. 23, 2026, at Seneca Valley's NexTier Stadium. Photog/Special to the Eagle\n\nREPORTER: none\n".into(),
            ..Default::default()
        };
        fill_from_sheet(&mut d);
        assert_eq!(d.slug, "102526 SPO FB Norwin-SV");
        assert_eq!(d.date, "2026-10-25");
        assert_eq!(
            d.venue, "Seneca Valley's NexTier Stadium",
            "the place the caption ending names beats the school"
        );
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

    #[test]
    fn reads_the_eagle_planning_page_as_pasted() {
        let mut d = Draft {
            pasted: "Plan\n3. 110126 SPO FB NC-Highlands\nType Plan Status Assigned Urgency 3 Section Sports\nFriday October 30 2026 19:00 - Friday October 30 2026 21:00\nFOOTBALL: North Catholic at Highlands, 7 pm\nADDRESS: Highlands HS, 1500 Pacific Ave, Natrona Heights, PA 15065\nCAPTION ENDING: ... in a football game Friday, Oct. 30, 2026, at Highlands Golden Rams Stadium. Photog/Special to the Eagle\nREPORTER: XXX\nDEADLINE: 2-4 print photos within 75 minutes, rest of gallery ASAP after\nASSIGNED: Jake\nShow Less\nAssignment\nPlan missing assignments\n".into(),
            ..Default::default()
        };
        fill_from_sheet(&mut d);
        assert_eq!(d.slug, "110126 SPO FB NC-Highlands");
        assert_eq!(
            d.date, "2026-10-30",
            "the game day, not the slug's run date"
        );
        assert_eq!(d.what, "a football game");
        assert_eq!(d.venue, "Highlands Golden Rams Stadium");
        assert_eq!(d.city, "Natrona Heights");
        assert_eq!(d.state, "Pa.");
        assert_eq!(d.keywords, "football; North Catholic; Highlands");
        assert!(d
            .ending
            .starts_with("in a football game Friday, Oct. 30, 2026"));
        assert!(!d.notes.contains("Plan missing"), "{}", d.notes);
        assert!(d.notes.contains("DEADLINE") && d.notes.contains("ASSIGNED: Jake"));
        let text = render(
            &d,
            &Outlet {
                credit: "Caleb McCartney/Special to the Eagle".into(),
                ..Default::default()
            },
        );
        let a = apature_metadata::stationery::Assignment::parse(&text).unwrap();
        assert!(a.unfilled().is_empty(), "{:?}", a.unfilled());
        assert_eq!(a.iptc().category.as_deref(), Some("SPO"));
    }
}
