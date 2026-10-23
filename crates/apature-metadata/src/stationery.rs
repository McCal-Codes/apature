//! Assignment stationery: the `key: value` config the owner already writes
//! for each Example Press assignment becomes an IPTC pad, a caption template
//! and a code table. Keys beyond the generator's set are accepted so one
//! file can carry everything a newsroom needs.

use crate::codes::{CodeTable, AP_MONTHS};
use crate::iptc::Iptc;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Assignment {
    pub date: Option<String>,
    pub slug: String,
    pub venue: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub event: Option<String>,
    pub keywords: Vec<String>,
    pub team: Option<String>,
    pub photographer: Option<String>,
    pub credit: Option<String>,
    pub source: Option<String>,
    pub creator: Option<String>,
    pub creator_title: Option<String>,
    pub caption_writer: Option<String>,
    pub copyright: Option<String>,
    pub codes: Vec<(String, String)>,
}

#[derive(Debug, thiserror::Error)]
pub enum StationeryError {
    #[error("assignment file has no 'slug:' line")]
    NoSlug,
}

fn split_list(v: &str) -> Vec<String> {
    v.split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

impl Assignment {
    pub fn parse(text: &str) -> Result<Assignment, StationeryError> {
        let mut a = Assignment::default();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((k, v)) = line.split_once(':') else {
                continue;
            };
            let (k, v) = (k.trim(), v.trim());
            match k {
                "date" => a.date = Some(v.into()),
                "slug" => a.slug = v.into(),
                "venue" => a.venue = Some(v.into()),
                "city" => a.city = Some(v.into()),
                "state" => a.state = Some(v.into()),
                "country" => a.country = Some(v.into()),
                "country_code" | "countrycode" => a.country_code = Some(v.into()),
                "event" => a.event = Some(v.into()),
                "keywords" => a.keywords.extend(split_list(v)),
                "supplemental" => a.keywords.extend(split_list(v)),
                "team" => a.team = Some(v.into()),
                "photographer" => a.photographer = Some(v.into()),
                "credit" => a.credit = Some(v.into()),
                "source" => a.source = Some(v.into()),
                "creator" => a.creator = Some(v.into()),
                "creator_title" | "title" => a.creator_title = Some(v.into()),
                "caption_writer" => a.caption_writer = Some(v.into()),
                "copyright" => a.copyright = Some(v.into()),
                "code" => {
                    if let Some((c, r)) = v.split_once('=') {
                        a.codes.push((c.trim().into(), r.trim().into()));
                    }
                }
                _ => {}
            }
        }
        if a.slug.is_empty() {
            return Err(StationeryError::NoSlug);
        }
        Ok(a)
    }

    /// "Tuesday, Oct. 6, 2026" from the event date.
    pub fn text_date(&self) -> Option<String> {
        let (y, m, d) = crate::codes::parse_date(self.date.as_deref()?)?;
        Some(format!(
            "{}, {} {d}, {y}",
            [
                "Sunday",
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday"
            ][crate::codes::weekday(y, m, d)],
            AP_MONTHS[(m - 1) as usize]
        ))
    }

    /// The credit line used in captions: `credit:` if given, else the photographer.
    pub fn credit_line(&self) -> Option<String> {
        self.credit.clone().or_else(|| self.photographer.clone())
    }

    /// The IPTC pad to apply to every frame of the assignment.
    pub fn iptc(&self) -> Iptc {
        let creator = self.creator.clone().or_else(|| {
            self.photographer
                .as_ref()
                .map(|p| p.split('/').next().unwrap_or(p).trim().to_string())
        });
        let mut keywords = vec![self.slug.clone()];
        keywords.extend(self.keywords.iter().cloned());
        keywords.dedup();
        Iptc {
            title: Some(self.slug.clone()),
            headline: Some(self.slug.clone()),
            transmission_reference: Some(self.slug.clone()),
            keywords: Some(keywords),
            creator: creator.map(|c| vec![c]),
            creator_title: self.creator_title.clone(),
            credit: self.credit_line(),
            source: self.source.clone(),
            copyright: self.copyright.clone(),
            caption_writer: self.caption_writer.clone(),
            city: self.city.clone(),
            state: self.state.clone(),
            country: self.country.clone(),
            country_code: self.country_code.clone(),
            sublocation: self.venue.clone(),
            date_created: self.date.as_ref().map(|d| {
                if d.len() == 10 {
                    format!("{d}T00:00:00")
                } else {
                    d.clone()
                }
            }),
            instructions: self.credit_line().map(|c| format!("Mandatory credit: {c}")),
            ..Default::default()
        }
    }

    /// Boilerplate codes (`tail`, `cr`, `dt`, `stad`, `game`) plus the
    /// assignment's own, with Photo Mechanic variable names so the date
    /// follows each photo.
    pub fn codes(&self) -> CodeTable {
        const DATE: &str = "{iptcdow}, {iptcmonthnameap} {iptcday}, {iptcyear4}";
        let mut codes = Vec::new();
        if let (Some(event), Some(venue)) = (&self.event, &self.venue) {
            codes.push((
                "tail".to_string(),
                format!(" in {event} held {DATE}, at {venue}. {{credit}}"),
            ));
            codes.push((
                "game".to_string(),
                format!(" in {event} held {DATE}, at {venue}."),
            ));
        }
        if let Some(c) = self.credit_line() {
            codes.push(("cr".to_string(), c));
        }
        codes.push(("dt".to_string(), DATE.to_string()));
        if let Some(v) = &self.venue {
            codes.push(("stad".to_string(), v.clone()));
        }
        codes.extend(self.codes.iter().cloned());
        CodeTable { codes }
    }

    /// Caption starter: "<team> PLAYER ACTION in <event> held …, at <venue>. <credit>"
    pub fn caption_template(&self) -> String {
        let subject = format!(
            "{} PLAYER ACTION",
            self.team.as_deref().unwrap_or("SUBJECT")
        );
        match (&self.event, &self.venue) {
            (Some(e), Some(v)) => format!("{subject} in {e} held {{iptcdow}}, {{iptcmonthnameap}} {{iptcday}}, {{iptcyear4}}, at {v}. {{credit}}"),
            _ => format!("{subject}. {{credit}}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CFG: &str = "# Example Press assignment config.\ndate: 2026-10-06\nslug: 100726 SPO BGLF WPIAL Championships\nvenue: Oakmont Country Club\ncity: Oakmont\nstate: Pa.\ncountry: USA\nevent: the WPIAL Class 2A Boys Individual Golf Championship\nkeywords: North Catholic; Trojans; WPIAL\nsupplemental: golf; high school\nteam: North Catholic's\nphotographer: Caleb McCartney/Special to the Eagle\nsource: Example Press\ncopyright: © 2026 Example Press\ncode: cf = North Catholic's Christopher Faber\ncode: oak = Oakmont Country Club\n";

    #[test]
    fn parses_the_generator_format() {
        let a = Assignment::parse(CFG).unwrap();
        assert_eq!(a.slug, "100726 SPO BGLF WPIAL Championships");
        assert_eq!(
            a.keywords,
            vec!["North Catholic", "Trojans", "WPIAL", "golf", "high school"]
        );
        assert_eq!(a.codes.len(), 2);
        assert_eq!(a.text_date().as_deref(), Some("Tuesday, Oct. 6, 2026"));
        assert!(Assignment::parse("date: 2026-01-01\n").is_err());
    }

    #[test]
    fn builds_pad_codes_and_template() {
        let a = Assignment::parse(CFG).unwrap();
        let i = a.iptc();
        assert_eq!(
            i.headline.as_deref(),
            Some("100726 SPO BGLF WPIAL Championships")
        );
        assert_eq!(i.creator, Some(vec!["Caleb McCartney".into()]));
        assert_eq!(
            i.credit.as_deref(),
            Some("Caleb McCartney/Special to the Eagle")
        );
        assert_eq!(i.sublocation.as_deref(), Some("Oakmont Country Club"));
        assert_eq!(i.date_created.as_deref(), Some("2026-10-06T00:00:00"));
        assert_eq!(
            i.keywords.as_ref().unwrap()[0],
            "100726 SPO BGLF WPIAL Championships"
        );
        let codes = a.codes();
        assert!(codes
            .get("tail")
            .unwrap()
            .starts_with(" in the WPIAL Class 2A"));
        assert_eq!(codes.get("cf"), Some("North Catholic's Christopher Faber"));
        let vars = crate::codes::Vars {
            date: Some((2026, 10, 6)),
            credit: i.credit.clone(),
            headline: None,
            ..Default::default()
        };
        let caption = crate::codes::expand(r"\cf\ tees off on No. 1\tail\", &codes, &vars, '\\');
        assert!(
            caption.ends_with("at Oakmont Country Club. Caleb McCartney/Special to the Eagle"),
            "{caption}"
        );
        assert!(a
            .caption_template()
            .starts_with("North Catholic's PLAYER ACTION in the WPIAL"));
    }
}
