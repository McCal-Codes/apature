//! Folder and file naming templates with Photo Mechanic's variable names,
//! so an existing `{headline}-{seqn}` habit carries over.
//!
//! Supported: `{filename}` `{ext}` `{frame}` `{seqn}` `{seqn:N}` `{headline}`
//! `{head}` `{slug}` `{date}` (2026-10-06) `{yymmdd}` (261006) `{year}`
//! `{yr2}` `{month}` `{day}` `{time}` (140311). Unknown tokens are left
//! literal so a typo shows up in the preview instead of vanishing; known
//! tokens with no value (no headline loaded, no capture date) expand to
//! nothing, and `sanitize_component` trims the separators they leave.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Facts {
    /// Original file name without extension.
    pub stem: String,
    pub ext: String,
    /// Capture date (y, m, d) and time (h, m, s) when known.
    pub date: Option<(i32, u32, u32)>,
    pub time: Option<(u32, u32, u32)>,
    pub headline: Option<String>,
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
            "year" => Ok(facts.date.map(|(y, _, _)| format!("{y:04}"))),
            "yr2" => Ok(facts
                .date
                .map(|(y, _, _)| format!("{:02}", y.rem_euclid(100)))),
            "month" => Ok(facts.date.map(|(_, m, _)| format!("{m:02}"))),
            "day" => Ok(facts.date.map(|(_, _, d)| format!("{d:02}"))),
            "time" => Ok(facts.time.map(|(h, m, s)| format!("{h:02}{m:02}{s:02}"))),
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
    use super::*;

    fn facts() -> Facts {
        Facts {
            stem: "DSC_0412".into(),
            ext: "NEF".into(),
            date: Some((2026, 10, 6)),
            time: Some((14, 3, 11)),
            headline: Some("100726 SPO BGLF".into()),
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
    fn sanitizes_components() {
        assert_eq!(sanitize_component("a/b:c").as_deref(), Some("a-b-c"));
        assert_eq!(sanitize_component("  ..  "), None);
        assert_eq!(sanitize_component("...hidden").as_deref(), Some("hidden"));
        assert_eq!(sanitize_component(""), None);
    }
}
