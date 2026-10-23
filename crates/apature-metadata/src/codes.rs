//! Code replacements, Photo Mechanic style: type `\cf\` in a caption and it
//! expands to the replacement, with `{variables}` resolved from the photo.
//!
//! File format: `code<TAB>replacement`, `//` comments, blank lines ignored.
//! Variables: Photo Mechanic's date and IPTC names (`{iptcdow}`,
//! `{iptcmonthnameap}`, `{credit}`, `{city}`, …), plain aliases, and the
//! photographer's own variables from `set_user_vars`.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CodeTable {
    pub codes: Vec<(String, String)>,
}

impl CodeTable {
    pub fn parse(text: &str) -> CodeTable {
        let mut codes = Vec::new();
        for line in text.lines() {
            let l = line.trim_end();
            if l.trim().is_empty() || l.trim_start().starts_with("//") {
                continue;
            }
            if let Some((c, r)) = l.split_once('\t') {
                let c = c.trim();
                if !c.is_empty() {
                    // Replacements keep leading spaces on purpose: `tail` starts with one.
                    codes.push((c.to_string(), r.to_string()));
                }
            }
        }
        CodeTable { codes }
    }

    pub fn get(&self, code: &str) -> Option<&str> {
        self.codes
            .iter()
            .find(|(c, _)| c == code)
            .map(|(_, r)| r.as_str())
    }

    /// Codes starting with `prefix`, for autocomplete.
    pub fn starting_with<'a>(
        &'a self,
        prefix: &'a str,
    ) -> impl Iterator<Item = &'a (String, String)> + 'a {
        self.codes
            .iter()
            .filter(move |(c, _)| c.starts_with(prefix))
    }

    pub fn merge(&mut self, other: &CodeTable) {
        for (c, r) in &other.codes {
            if let Some(slot) = self.codes.iter_mut().find(|(k, _)| k == c) {
                slot.1 = r.clone();
            } else {
                self.codes.push((c.clone(), r.clone()));
            }
        }
    }
}

/// What the variables resolve from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Vars {
    /// (year, month, day), from IPTC DateCreated or EXIF DateTimeOriginal.
    pub date: Option<(i32, u32, u32)>,
    pub credit: Option<String>,
    pub headline: Option<String>,
    pub title: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub sublocation: Option<String>,
    pub source: Option<String>,
    pub copyright: Option<String>,
    pub creator: Option<String>,
    pub caption_writer: Option<String>,
    pub transmission_reference: Option<String>,
    pub keywords: Option<String>,
    pub filename: Option<String>,
}

/// Variables the photographer defines (Settings › Metadata › Your
/// variables): `{name}` anywhere a template or a code replacement is
/// expanded. App-wide, so they live in one place rather than in every
/// `Vars` and `Facts`.
static USER_VARS: std::sync::RwLock<Vec<(String, String)>> = std::sync::RwLock::new(Vec::new());

pub fn set_user_vars(vars: Vec<(String, String)>) {
    if let Ok(mut w) = USER_VARS.write() {
        *w = vars;
    }
}

/// Look a user variable up, case-insensitively, trimming the name.
pub fn user_var(name: &str) -> Option<String> {
    let n = name.trim();
    USER_VARS
        .read()
        .ok()?
        .iter()
        .find(|(k, _)| k.trim().eq_ignore_ascii_case(n))
        .map(|(_, v)| v.clone())
}

pub const AP_MONTHS: [&str; 12] = [
    "Jan.", "Feb.", "March", "April", "May", "June", "July", "Aug.", "Sept.", "Oct.", "Nov.",
    "Dec.",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const DAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// Day of week, 0 = Sunday (Zeller's congruence, proleptic Gregorian).
pub fn weekday(y: i32, m: u32, d: u32) -> usize {
    let (mut y, mut m) = (y, m as i32);
    if m < 3 {
        m += 12;
        y -= 1;
    }
    let k = y % 100;
    let j = y / 100;
    let h = (d as i32 + 13 * (m + 1) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    // h: 0 = Saturday
    ((h + 6) % 7) as usize
}

/// Parse "YYYY-MM-DD…" or "YYYY:MM:DD…" into (y, m, d).
pub fn parse_date(s: &str) -> Option<(i32, u32, u32)> {
    let s = s.trim();
    let y: i32 = s.get(0..4)?.parse().ok()?;
    let m: u32 = s.get(5..7)?.parse().ok()?;
    let d: u32 = s.get(8..10)?.parse().ok()?;
    if (1..=12).contains(&m) && (1..=31).contains(&d) {
        Some((y, m, d))
    } else {
        None
    }
}

/// Resolve one `{name}` for a caption. `Ok(Some)` = value, `Ok(None)` =
/// known but empty for this photo (expands to nothing), `Err` = unknown
/// (left as typed so a typo is visible).
fn caption_var(name: &str, vars: &Vars) -> Result<Option<String>, ()> {
    let n = name.trim().to_ascii_lowercase();
    let date = vars.date;
    let v = match n.as_str() {
        "iptcdow" | "dow" | "dayname" => date.map(|(y, m, d)| DAYS[weekday(y, m, d)].to_string()),
        "iptcdow3" | "dow3" => date.map(|(y, m, d)| DAYS[weekday(y, m, d)][..3].to_string()),
        "iptcmonthnameap" | "monthnameap" | "apmonth" => {
            date.map(|(_, m, _)| AP_MONTHS[(m - 1) as usize].to_string())
        }
        "iptcmonthname" | "monthname" => date.map(|(_, m, _)| MONTHS[(m - 1) as usize].to_string()),
        "iptcmonthname3" | "mnn3" | "monthname3" => {
            date.map(|(_, m, _)| MONTHS[(m - 1) as usize][..3].to_string())
        }
        "iptcmonth0" | "month0" | "imn0" => date.map(|(_, m, _)| format!("{m:02}")),
        "iptcmonth" | "month" => date.map(|(_, m, _)| m.to_string()),
        "iptcday" | "day" => date.map(|(_, _, d)| d.to_string()),
        "iptcday0" | "day0" | "idy0" => date.map(|(_, _, d)| format!("{d:02}")),
        "iptcyear4" | "year4" | "year" | "iyr4" => date.map(|(y, _, _)| y.to_string()),
        "iptcyear2" | "yr2" | "year2" | "iyr2" => {
            date.map(|(y, _, _)| format!("{:02}", y.rem_euclid(100)))
        }
        "iptcdatesort" | "datesort" => date.map(|(y, m, d)| format!("{y:04}{m:02}{d:02}")),
        "date" | "iptcdate" => date.map(|(y, m, d)| format!("{y:04}-{m:02}-{d:02}")),
        "apdate" => date.map(|(y, m, d)| format!("{} {d}, {y}", AP_MONTHS[(m - 1) as usize])),
        "credit" => vars.credit.clone(),
        "headline" | "head" | "slug" => vars.headline.clone(),
        "title" | "objectname" | "object name" => vars.title.clone(),
        "city" | "iptccity" => vars.city.clone(),
        "state" | "iptcstate" | "province" => vars.state.clone(),
        "country" | "iptccountry" => vars.country.clone(),
        "sublocation" | "location" | "venue" => vars.sublocation.clone(),
        "source" => vars.source.clone(),
        "copyright" | "copyrightnotice" => vars.copyright.clone(),
        "creator" | "byline" | "photographer" => vars.creator.clone(),
        "captionwriter" | "writer" => vars.caption_writer.clone(),
        "transref" | "transmissionreference" | "jobid" => vars.transmission_reference.clone(),
        "keywords" => vars.keywords.clone(),
        "filename" | "filenamebase" => vars.filename.clone(),
        _ => return user_var(name).map(Some).ok_or(()),
    };
    Ok(v)
}

/// Expand every `{variable}` in `s`. Known variables resolve from `vars`
/// (Photo Mechanic's names and plain ones both work), then from the user's
/// own variables; anything else stays as typed.
pub fn expand_variables(s: &str, vars: &Vars) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            out.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let token = &after[..end];
        match caption_var(token, vars) {
            Ok(Some(v)) => out.push_str(&v),
            Ok(None) => {}
            Err(()) => {
                out.push('{');
                out.push_str(token);
                out.push('}');
            }
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Expand every `<delim>code<delim>` in `text`. Unknown codes are left as
/// typed so a typo is visible rather than silently swallowed.
pub fn expand(text: &str, table: &CodeTable, vars: &Vars, delim: char) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(delim) {
        out.push_str(&rest[..start]);
        let after = &rest[start + delim.len_utf8()..];
        match after.find(delim) {
            Some(end) => {
                let code = &after[..end];
                match table.get(code) {
                    Some(r) if !code.is_empty() => {
                        out.push_str(&expand_variables(r, vars));
                        rest = &after[end + delim.len_utf8()..];
                    }
                    _ => {
                        out.push(delim);
                        rest = after;
                    }
                }
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "// Photo Mechanic code replacement file\n\n// --- boilerplate ---\ntail\t in the WPIAL Class 2A Boys Individual Golf Championship held {iptcdow}, {iptcmonthnameap} {iptcday}, {iptcyear4}, at Oakmont Country Club. {credit}\ncr\tCaleb McCartney/Special to the Eagle\n\n// --- assignment codes ---\ncf\tNorth Catholic's Christopher Faber\ncc\tNorth Catholic's Colin Campbell\n";

    #[test]
    fn parses_pm_file() {
        let t = CodeTable::parse(FILE);
        assert_eq!(t.codes.len(), 4);
        assert_eq!(t.get("cf"), Some("North Catholic's Christopher Faber"));
        assert_eq!(t.starting_with("c").count(), 3);
    }

    #[test]
    fn weekday_and_ap_dates() {
        assert_eq!(weekday(2026, 10, 6), 2, "Oct 6 2026 is a Tuesday");
        assert_eq!(weekday(2025, 4, 10), 4, "Apr 10 2025 is a Thursday");
        assert_eq!(parse_date("2026-10-06T00:00:00-04:00"), Some((2026, 10, 6)));
        assert_eq!(parse_date("2025:04:10 19:00:19"), Some((2025, 4, 10)));
        assert_eq!(parse_date("nope"), None);
    }

    #[test]
    fn expands_codes_with_variables_and_keeps_typos() {
        let t = CodeTable::parse(FILE);
        let vars = Vars {
            date: Some((2026, 10, 6)),
            credit: Some("Caleb McCartney/Special to the Eagle".into()),
            headline: None,
            ..Default::default()
        };
        let s = expand(r"\cf\ tees off on No. 1\tail\", &t, &vars, '\\');
        assert_eq!(s, "North Catholic's Christopher Faber tees off on No. 1 in the WPIAL Class 2A Boys Individual Golf Championship held Tuesday, Oct. 6, 2026, at Oakmont Country Club. Caleb McCartney/Special to the Eagle");
        assert_eq!(
            expand(r"\zz\ ok \cf\", &t, &vars, '\\'),
            r"\zz\ ok North Catholic's Christopher Faber"
        );
        assert_eq!(expand(r"half \cf", &t, &vars, '\\'), r"half \cf");
        assert_eq!(expand("no codes", &t, &vars, '\\'), "no codes");
    }
}
