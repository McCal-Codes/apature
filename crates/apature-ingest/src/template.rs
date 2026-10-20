//! Folder and file naming templates with Photo Mechanic's variable names,
//! so an existing `{headline}-{seqn}` habit carries over.
//!
//! Supported (Photo Mechanic names and aliases):
//! file: `{filename}` `{ext}` `{frame}` `{seqn}` `{seqn:N}` `{folder}`
//! assignment: `{headline}` `{head}` `{slug}`
//! capture date: `{date}` (2026-10-06) `{datesort}` (20261006) `{yymmdd}`
//! `{year4}` `{yr2}` `{month0}` `{monthname}` `{mnn3}` `{day0}` `{dow}` `{dow3}`
//! capture time: `{time}` (140311) `{hour}` `{hour12}` `{ampm}` `{minute}` `{second}`
//! camera: `{camera}` `{make}` `{lens}` `{iso}` `{aperture}` `{shutter}` `{focal}`
//! ingest day: `{today}` (2026-10-15) `{todaysort}` (20261015)
//! Unknown tokens are left literal so a typo shows up in the preview instead
//! of vanishing; known tokens with no value (no headline loaded, no capture
//! date) expand to nothing, and `sanitize_component` trims the separators
//! they leave.

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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Facts {
    /// Original file name without extension.
    pub stem: String,
    pub ext: String,
    /// Capture date (y, m, d) and time (h, m, s) when known.
    pub date: Option<(i32, u32, u32)>,
    pub time: Option<(u32, u32, u32)>,
    pub headline: Option<String>,
    /// Name of the folder the source file sits in (`100NIKON`).
    pub folder: Option<String>,
    /// Camera model, maker and lens as EXIF reports them.
    pub camera: Option<String>,
    pub make: Option<String>,
    pub lens: Option<String>,
    pub iso: Option<u32>,
    pub f_number: Option<f64>,
    /// Exposure as (numerator, denominator), e.g. (1, 250).
    pub exposure: Option<(u32, u32)>,
    pub focal_mm: Option<f64>,
    /// The day the ingest runs (Photo Mechanic's "today" variables).
    pub today: Option<(i32, u32, u32)>,
    /// Name of the destination root folder (`{dest}`), set by the planner.
    pub dest: Option<String>,
}

const DAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// Day of the week, 0 = Sunday (Sakamoto's method, proleptic Gregorian).
pub fn weekday(y: i32, m: u32, d: u32) -> usize {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if m < 3 { y - 1 } else { y };
    let m = m.clamp(1, 12) as usize;
    (y + y / 4 - y / 100 + y / 400 + T[m - 1] + d as i32).rem_euclid(7) as usize
}

/// A number for a file name: 2.8 → "2.8", 70.0 → "70", 1.0 / 250 → "1-250".
fn trim_float(v: f64) -> String {
    if (v - v.round()).abs() < 1e-6 {
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{v:.1}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

impl Facts {
    /// Trailing digits of the stem ("DSC_0412" → "0412", "850_7783" → "7783").
    pub fn frame(&self) -> String {
        let digits: String = self
            .stem
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        digits.chars().rev().collect()
    }
}

/// Expand `template` for one file. `seq` is the per-run sequence number.
pub fn expand(template: &str, facts: &Facts, seq: u32) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            out.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let token = &after[..end];
        let (name, arg) = match token.split_once(':') {
            Some((n, a)) => (n, Some(a)),
            None => (token, None),
        };
        // Ok(Some) = value, Ok(None) = known but missing, Err = unknown token.
        let value: Result<Option<String>, ()> = match name {
            "filename" | "stem" => Ok(Some(facts.stem.clone())),
            "ext" => Ok(Some(facts.ext.clone())),
            "frame" => Ok(Some(facts.frame())),
            "seqn" | "seq" => {
                let width = arg.and_then(|a| a.parse::<usize>().ok()).unwrap_or(3);
                Ok(Some(format!("{seq:0width$}")))
            }
            "headline" | "head" | "slug" => Ok(facts.headline.clone()),
            "date" => Ok(facts.date.map(|(y, m, d)| format!("{y:04}-{m:02}-{d:02}"))),
            "yymmdd" => Ok(facts
                .date
                .map(|(y, m, d)| format!("{:02}{m:02}{d:02}", y.rem_euclid(100)))),
            "year" | "year4" | "iptcyear4" | "iyr4" => {
                Ok(facts.date.map(|(y, _, _)| format!("{y:04}")))
            }
            "yr2" | "iyr2" | "iptcyear2" | "year2" => Ok(facts
                .date
                .map(|(y, _, _)| format!("{:02}", y.rem_euclid(100)))),
            "month" | "month0" | "imn0" | "iptcmonth0" => {
                Ok(facts.date.map(|(_, m, _)| format!("{m:02}")))
            }
            "day" | "day0" | "idy0" | "iptcday0" => {
                Ok(facts.date.map(|(_, _, d)| format!("{d:02}")))
            }
            "monthname" | "iptcmonthname" => Ok(facts
                .date
                .map(|(_, m, _)| MONTHS[(m - 1) as usize].to_string())),
            "mnn3" | "iptcmonthname3" | "monthname3" => Ok(facts
                .date
                .map(|(_, m, _)| MONTHS[(m - 1) as usize][..3].to_string())),
            "datesort" | "iptcdatesort" => {
                Ok(facts.date.map(|(y, m, d)| format!("{y:04}{m:02}{d:02}")))
            }
            "dow" | "dayname" | "weekday" => Ok(facts
                .date
                .map(|(y, m, d)| DAYS[weekday(y, m, d)].to_string())),
            "dow3" | "dayname3" => Ok(facts
                .date
                .map(|(y, m, d)| DAYS[weekday(y, m, d)][..3].to_string())),
            "hour" | "hour24" | "hr24" => Ok(facts.time.map(|(h, _, _)| format!("{h:02}"))),
            "hour12" | "hr12" => Ok(facts.time.map(|(h, _, _)| {
                let h12 = match h % 12 {
                    0 => 12,
                    x => x,
                };
                format!("{h12:02}")
            })),
            "ampm" => Ok(facts
                .time
                .map(|(h, _, _)| if h < 12 { "AM" } else { "PM" }.to_string())),
            "minute" | "min" => Ok(facts.time.map(|(_, m, _)| format!("{m:02}"))),
            "second" | "sec" => Ok(facts.time.map(|(_, _, s)| format!("{s:02}"))),
            "time" => Ok(facts.time.map(|(h, m, s)| format!("{h:02}{m:02}{s:02}"))),
            "folder" | "foldername" => Ok(facts.folder.clone()),
            "dest" | "destination" => Ok(facts.dest.clone()),
            "camera" | "model" => Ok(facts.camera.clone()),
            "make" => Ok(facts.make.clone()),
            "lens" => Ok(facts.lens.clone()),
            "iso" => Ok(facts.iso.map(|v| v.to_string())),
            "aperture" | "fnumber" | "fstop" => Ok(facts.f_number.map(trim_float)),
            "shutter" | "exposure" => Ok(facts.exposure.map(|(n, d)| {
                if d == 1 {
                    format!("{n}s")
                } else {
                    format!("{n}-{d}")
                }
            })),
            "focal" | "focallength" | "focallen" => Ok(facts.focal_mm.map(trim_float)),
            "today" | "todaysdate" | "ingestdate" => {
                Ok(facts.today.map(|(y, m, d)| format!("{y:04}-{m:02}-{d:02}")))
            }
            "todaysort" | "ingestdatesort" => {
                Ok(facts.today.map(|(y, m, d)| format!("{y:04}{m:02}{d:02}")))
            }
            "tyr2" => Ok(facts
                .today
                .map(|(y, _, _)| format!("{:02}", y.rem_euclid(100)))),
            "tmn0" => Ok(facts.today.map(|(_, m, _)| format!("{m:02}"))),
            "tdy0" => Ok(facts.today.map(|(_, _, d)| format!("{d:02}"))),
            _ => Err(()),
        };
        match value {
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

/// Make a template result safe as one path component: no separators, no
/// `..`, no leading dot, trimmed. Returns `None` if nothing usable is left.
pub fn sanitize_component(s: &str) -> Option<String> {
    let cleaned: String = s
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '\0') {
                '-'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned
        .trim()
        .trim_matches(|c: char| c == '-' || c == '_' || c == ' ')
        .trim_start_matches('.')
        .trim()
        .to_string();
    if cleaned.is_empty() || cleaned == ".." {
        None
    } else {
        Some(cleaned)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_trib_pattern_without_section() {
        use super::*;
        let f = Facts {
            stem: "DSC_0412".into(),
            ext: "jpg".into(),
            date: Some((2026, 9, 12)),
            time: None,
            headline: Some("TasteOfSewickley".into()),
            ..Default::default()
        };
        assert_eq!(
            expand("{headline}{seqn:3}-{month0}{day0}{yr2}", &f, 1),
            "TasteOfSewickley001-091226"
        );
    }

    use super::*;

    fn facts() -> Facts {
        Facts {
            stem: "DSC_0412".into(),
            ext: "NEF".into(),
            date: Some((2026, 10, 6)),
            time: Some((14, 3, 11)),
            headline: Some("100726 SPO BGLF".into()),
            ..Default::default()
        }
    }

    #[test]
    fn expands_pm_style_names() {
        let f = facts();
        assert_eq!(expand("{headline}-{seqn}", &f, 7), "100726 SPO BGLF-007");
        assert_eq!(expand("{head}-{seqn:4}", &f, 7), "100726 SPO BGLF-0007");
        assert_eq!(expand("{date}/{filename}", &f, 1), "2026-10-06/DSC_0412");
        assert_eq!(
            expand("{yymmdd}_{frame}_{time}", &f, 1),
            "261006_0412_140311"
        );
        assert_eq!(expand("{year}/{month}/{day}", &f, 1), "2026/10/06");
        // The owner's Photo Mechanic conventions, verbatim.
        assert_eq!(
            expand("{year4}/{month0} {monthname}/{mnn3} {day0} {head}", &f, 1),
            "2026/10 October/Oct 06 100726 SPO BGLF"
        );
        assert_eq!(
            expand("{iyr2}{imn0}{idy0}{headline}", &f, 1),
            "261006100726 SPO BGLF"
        );
        assert_eq!(
            expand(
                "BUTLR-{headline}{seqn}-{iptcmonth0}{iptcday0}{iptcyear4}",
                &f,
                12
            ),
            "BUTLR-100726 SPO BGLF012-10062026"
        );
        assert_eq!(
            expand("{iptcmonthname3} {hour}:{minute}", &f, 1),
            "Oct 14:03"
        );
        assert_eq!(
            expand("{nope}-{seq}", &f, 2),
            "{nope}-002",
            "unknown tokens stay literal"
        );
        assert_eq!(expand("{unclosed", &f, 1), "{unclosed");
        let nodate = Facts {
            date: None,
            headline: None,
            ..facts()
        };
        assert_eq!(
            expand("{date}-{headline}", &nodate, 1),
            "-",
            "known but missing tokens vanish"
        );
        assert_eq!(
            sanitize_component(&expand("{date}-{headline}", &nodate, 1)),
            None
        );
        assert_eq!(
            sanitize_component(&expand("{headline}-{seqn}", &nodate, 4)).as_deref(),
            Some("004")
        );
    }

    #[test]
    fn camera_time_and_today_variables_expand_like_photo_mechanic() {
        let f = Facts {
            stem: "DSC_0412".into(),
            ext: "NEF".into(),
            date: Some((2026, 9, 26)),
            time: Some((19, 4, 7)),
            headline: Some("RyansBday".into()),
            folder: Some("100NIKON".into()),
            camera: Some("NIKON Z 6_2".into()),
            make: Some("NIKON CORPORATION".into()),
            lens: Some("NIKKOR Z 24-70mm f/2.8 S".into()),
            iso: Some(6400),
            f_number: Some(2.8),
            exposure: Some((1, 250)),
            focal_mm: Some(70.0),
            today: Some((2026, 10, 15)),
            dest: Some("CMcCartney".into()),
        };
        assert_eq!(
            expand("{datesort}-{dow3}-{dow}", &f, 1),
            "20260926-Sat-Saturday"
        );
        assert_eq!(
            expand("{hour12}{ampm}-{hour}{min}{sec}", &f, 1),
            "07PM-190407"
        );
        assert_eq!(
            expand("{camera}_{iso}_{aperture}_{shutter}_{focal}", &f, 1),
            "NIKON Z 6_2_6400_2.8_1-250_70"
        );
        assert_eq!(
            expand("{today}/{todaysort}/{tyr2}{tmn0}{tdy0}", &f, 1),
            "2026-10-15/20261015/261015"
        );
        assert_eq!(
            expand("{folder}/{make}", &f, 1),
            "100NIKON/NIKON CORPORATION"
        );
        assert_eq!(weekday(2026, 10, 15), 4, "2026-10-15 is a Thursday");
        assert_eq!(weekday(2000, 1, 1), 6, "2000-01-01 was a Saturday");
        // A missing value expands to nothing; sanitising cleans the leftovers.
        let bare = Facts {
            stem: "x".into(),
            ..Default::default()
        };
        assert_eq!(
            sanitize_component(&expand("{camera}-{iso}-{stem}", &bare, 1)).as_deref(),
            Some("x")
        );
    }

    #[test]
    fn sanitizes_components() {
        assert_eq!(sanitize_component("a/b:c").as_deref(), Some("a-b-c"));
        assert_eq!(sanitize_component("  ..  "), None);
        assert_eq!(sanitize_component("...hidden").as_deref(), Some("hidden"));
        assert_eq!(sanitize_component(""), None);
    }
}
