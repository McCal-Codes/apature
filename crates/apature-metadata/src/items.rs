//! One policy for where an item's marks live, matching what Photo Mechanic
//! and Lightroom do:
//!
//! * a RAW's marks live in its `.xmp` sidecar;
//! * a JPEG's marks live inside the JPEG;
//! * a RAW+JPEG pair gets **both**, so either app sees the same thing
//!   whichever file it treats as primary.

use std::path::PathBuf;

use apature_core::{ColorClassScheme, Kind, Marks, PhotoItem};

use crate::iptc::{self, Iptc};
use crate::jpeg_xmp;
use crate::xmp::{self, Conflict, XmpMarks};

#[derive(Debug, thiserror::Error)]
pub enum ItemError {
    #[error("sidecar {path}: {source}")]
    Sidecar {
        path: PathBuf,
        source: xmp::XmpError,
    },
    #[error("jpeg {path}: {source}")]
    Jpeg {
        path: PathBuf,
        source: jpeg_xmp::JpegXmpError,
    },
}

/// Everything readable about an item's marks, before any precedence is applied.
#[derive(Debug, Default, Clone)]
pub struct MarkSources {
    pub sidecar: Option<XmpMarks>,
    /// One entry per JPEG variant that carries a packet.
    pub embedded: Vec<(PathBuf, XmpMarks)>,
}

impl MarkSources {
    /// Precedence: for an item with a RAW, the sidecar; otherwise the first
    /// JPEG packet; otherwise unmarked. Unreadable sources count as absent.
    pub fn effective(&self, item: &PhotoItem, scheme: &ColorClassScheme) -> Marks {
        let has_raw = item.variants().any(|v| v.kind == Kind::Raw);
        let chosen = if has_raw {
            self.sidecar
                .as_ref()
                .or_else(|| self.embedded.first().map(|(_, m)| m))
        } else {
            self.embedded
                .first()
                .map(|(_, m)| m)
                .or(self.sidecar.as_ref())
        };
        chosen.map(|m| m.to_marks(scheme)).unwrap_or_default()
    }

    /// Disagreements between the sidecar and any embedded packet.
    pub fn conflicts(&self) -> Vec<(PathBuf, Conflict)> {
        let Some(side) = &self.sidecar else {
            return Vec::new();
        };
        self.embedded
            .iter()
            .flat_map(|(p, emb)| {
                xmp::conflicts(side, emb)
                    .into_iter()
                    .map(move |c| (p.clone(), c))
            })
            .collect()
    }
}

/// Read every source for an item. I/O errors on individual files are
/// treated as "no marks there"; a damaged JPEG must not hide the sidecar.
pub fn read(item: &PhotoItem) -> MarkSources {
    let sidecar = xmp::read_sidecar(&item.sidecar_path()).ok().flatten();
    let embedded = item
        .variants()
        .filter(|v| v.kind == Kind::Jpeg)
        .filter_map(|v| {
            xmp::read_embedded(&v.path)
                .ok()
                .flatten()
                .map(|m| (v.path.clone(), m))
        })
        .collect();
    MarkSources { sidecar, embedded }
}

/// Convenience: the marks the app should show.
pub fn read_marks(item: &PhotoItem, scheme: &ColorClassScheme) -> Marks {
    read(item).effective(item, scheme)
}

/// Write marks everywhere they belong. Returns the paths written. Stops at
/// the first failure; paths written before it stay written (each write is
/// itself atomic), and the error names the file.
pub fn write(
    item: &PhotoItem,
    marks: &Marks,
    scheme: &ColorClassScheme,
) -> Result<Vec<PathBuf>, ItemError> {
    let xm = XmpMarks::from_marks(marks, scheme);
    let mut written = Vec::new();
    if item.variants().any(|v| v.kind == Kind::Raw) {
        let p = item.sidecar_path();
        xmp::write_marks(&p, &xm).map_err(|source| ItemError::Sidecar {
            path: p.clone(),
            source,
        })?;
        written.push(p);
    }
    for v in item.variants().filter(|v| v.kind == Kind::Jpeg) {
        jpeg_xmp::write_marks(&v.path, &xm).map_err(|source| ItemError::Jpeg {
            path: v.path.clone(),
            source,
        })?;
        written.push(v.path.clone());
    }
    Ok(written)
}

/// Read IPTC for an item: the sidecar when a RAW exists, else the first JPEG
/// packet. Returns an empty `Iptc` when nothing is there.
pub fn read_iptc(item: &PhotoItem) -> Iptc {
    let has_raw = item.variants().any(|v| v.kind == Kind::Raw);
    let from_sidecar = || {
        std::fs::read_to_string(item.sidecar_path())
            .ok()
            .map(|t| iptc::parse(&t))
    };
    let from_jpeg = || {
        item.variants()
            .filter(|v| v.kind == Kind::Jpeg)
            .find_map(|v| {
                let data = std::fs::read(&v.path).ok()?;
                jpeg_xmp::packet(&data).ok().flatten().map(iptc::parse)
            })
    };
    if has_raw {
        from_sidecar().or_else(from_jpeg)
    } else {
        from_jpeg().or_else(from_sidecar)
    }
    .unwrap_or_default()
}

/// Write IPTC everywhere marks go (sidecar for a RAW, packet for each JPEG).
/// `fields` with every `{variable}` in its text expanded for this photo:
/// the date from the photo's own capture time (else the pad's date), the
/// credit, headline and the rest from the pad itself. Photo Mechanic does the
/// same to a template's fields; unknown names stay as typed.
pub fn expand_pad(item: &PhotoItem, fields: &Iptc) -> Iptc {
    use crate::codes::{self, Vars};
    let has_braces = |s: &Option<String>| s.as_deref().is_some_and(|t| t.contains('{'));
    let any = has_braces(&fields.title)
        || has_braces(&fields.headline)
        || has_braces(&fields.description)
        || has_braces(&fields.credit)
        || has_braces(&fields.source)
        || has_braces(&fields.copyright)
        || has_braces(&fields.instructions)
        || has_braces(&fields.sublocation)
        || has_braces(&fields.caption_writer)
        || has_braces(&fields.city)
        || fields
            .keywords
            .iter()
            .flatten()
            .chain(fields.creator.iter().flatten())
            .chain(fields.supplemental_categories.iter().flatten())
            .any(|k| k.contains('{'));
    if !any {
        return fields.clone();
    }
    let date = crate::exif::read(item.path())
        .ok()
        .and_then(|e| e.date_time_original)
        .and_then(|d| codes::parse_date(&d.replace(':', "-")))
        .or_else(|| fields.date_created.as_deref().and_then(codes::parse_date));
    let vars = Vars {
        date,
        credit: fields.credit.clone(),
        headline: fields.headline.clone(),
        title: fields.title.clone(),
        city: fields.city.clone(),
        state: fields.state.clone(),
        country: fields.country.clone(),
        sublocation: fields.sublocation.clone(),
        source: fields.source.clone(),
        copyright: fields.copyright.clone(),
        creator: fields.creator.as_ref().and_then(|c| c.first().cloned()),
        caption_writer: fields.caption_writer.clone(),
        transmission_reference: fields.transmission_reference.clone(),
        keywords: None,
        filename: item
            .path()
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned()),
    };
    let x = |s: &Option<String>| s.as_ref().map(|t| codes::expand_variables(t, &vars));
    let xl = |l: &Option<Vec<String>>| {
        l.as_ref().map(|v| {
            v.iter()
                .map(|t| codes::expand_variables(t, &vars))
                .collect()
        })
    };
    Iptc {
        title: x(&fields.title),
        headline: x(&fields.headline),
        description: x(&fields.description),
        keywords: xl(&fields.keywords),
        creator: xl(&fields.creator),
        creator_title: x(&fields.creator_title),
        credit: x(&fields.credit),
        source: x(&fields.source),
        copyright: x(&fields.copyright),
        caption_writer: x(&fields.caption_writer),
        city: x(&fields.city),
        state: x(&fields.state),
        country: x(&fields.country),
        sublocation: x(&fields.sublocation),
        instructions: x(&fields.instructions),
        transmission_reference: x(&fields.transmission_reference),
        category: x(&fields.category),
        supplemental_categories: xl(&fields.supplemental_categories),
        usage_terms: x(&fields.usage_terms),
        ..fields.clone()
    }
}

pub fn write_iptc(item: &PhotoItem, fields: &Iptc) -> Result<Vec<PathBuf>, ItemError> {
    let fields = &expand_pad(item, fields);
    let mut written = Vec::new();
    if item.variants().any(|v| v.kind == Kind::Raw) {
        let p = item.sidecar_path();
        xmp::write_sidecar_with(
            &p,
            |t| iptc::apply(t, fields),
            |t| iptc::satisfies(fields, &iptc::parse(t)),
        )
        .map_err(|source| ItemError::Sidecar {
            path: p.clone(),
            source,
        })?;
        written.push(p);
    }
    for v in item.variants().filter(|v| v.kind == Kind::Jpeg) {
        jpeg_xmp::write_with(
            &v.path,
            |t| iptc::apply(t, fields),
            |t| iptc::satisfies(fields, &iptc::parse(t)),
        )
        .map_err(|source| ItemError::Jpeg {
            path: v.path.clone(),
            source,
        })?;
        written.push(v.path.clone());
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use apature_core::{ColorClass, Rating, ScanOptions};
    use std::fs;

    fn jpeg_skeleton() -> Vec<u8> {
        let mut v = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        v.extend_from_slice(b"JFIF\0\x01\x01\x01\0\x48\0\x48\0\0");
        v.extend_from_slice(&[
            0xFF, 0xDA, 0x00, 0x08, 1, 1, 0, 0, 0x3F, 0, 0x12, 0xFF, 0xD9,
        ]);
        v
    }

    #[test]
    fn pair_gets_sidecar_and_embedded_and_reads_back_consistently() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("DSC_0001.NEF"), b"not a real nef").unwrap();
        fs::write(dir.path().join("DSC_0001.JPG"), jpeg_skeleton()).unwrap();
        fs::write(dir.path().join("DSC_0002.JPG"), jpeg_skeleton()).unwrap();
        let items = apature_core::scan(dir.path(), &ScanOptions::default()).unwrap();
        let scheme = ColorClassScheme::default();
        let marks = Marks {
            rating: Rating(3),
            color_class: ColorClass(2),
            tagged: true,
        };

        let written = write(&items[0], &marks, &scheme).unwrap();
        assert_eq!(written.len(), 2);
        assert!(written[0].ends_with("DSC_0001.XMP"));
        assert!(written[1].ends_with("DSC_0001.JPG"));
        let src = read(&items[0]);
        assert_eq!(src.sidecar.as_ref().unwrap().rating, Some(3));
        assert_eq!(src.embedded.len(), 1);
        assert!(src.conflicts().is_empty());
        assert_eq!(src.effective(&items[0], &scheme), marks);

        // JPEG-only item: embedded only, no sidecar created.
        let written = write(&items[1], &marks, &scheme).unwrap();
        assert_eq!(written, vec![dir.path().join("DSC_0002.JPG")]);
        assert!(!dir.path().join("DSC_0002.xmp").exists());
        assert_eq!(read_marks(&items[1], &scheme), marks);

        // IPTC goes to both files of the pair and reads back from the sidecar.
        let fields = Iptc {
            description: Some("A caption".into()),
            keywords: Some(vec!["k1".into(), "k2".into()]),
            ..Default::default()
        };
        let w = write_iptc(&items[0], &fields).unwrap();
        assert_eq!(w.len(), 2);
        let back = read_iptc(&items[0]);
        assert_eq!(back.description.as_deref(), Some("A caption"));
        assert_eq!(back.keywords, Some(vec!["k1".into(), "k2".into()]));
        assert_eq!(
            read_marks(&items[0], &scheme),
            marks,
            "marks survive an IPTC write"
        );
        let jpg_only = read_iptc(&items[1]);
        assert!(jpg_only.is_empty());

        // A later sidecar edit by another app wins for the pair, and is reported.
        xmp::write_marks(
            &items[0].sidecar_path(),
            &XmpMarks {
                rating: Some(5),
                ..Default::default()
            },
        )
        .unwrap();
        let src = read(&items[0]);
        assert_eq!(src.effective(&items[0], &scheme).rating, Rating(5));
        assert_eq!(src.conflicts().len(), 1);
        assert_eq!(src.conflicts()[0].1.field, "rating");
    }

    #[test]
    fn a_pad_with_variables_is_filled_in_for_each_photo() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("DSC_0001.JPG"), jpeg_skeleton()).unwrap();
        let items = apature_core::scan(dir.path(), &ScanOptions::default()).unwrap();
        let pad = Iptc {
            headline: Some("103126 SPO FB A-B".into()),
            credit: Some("Caleb McCartney/Special to the Eagle".into()),
            date_created: Some("2026-10-30T00:00:00".into()),
            description: Some(
                "Highlands' PLAYER (##) ACTION in a football game {iptcdow}, {iptcmonthnameap} {iptcday}, {iptcyear4}. {credit}".into(),
            ),
            copyright: Some("\u{a9} {year4} Caleb McCartney".into()),
            keywords: Some(vec!["{headline}".into(), "{not_a_variable}".into(), "football".into()]),
            ..Default::default()
        };
        let out = expand_pad(&items[0], &pad);
        assert_eq!(
            out.description.as_deref(),
            Some("Highlands' PLAYER (##) ACTION in a football game Friday, Oct. 30, 2026. Caleb McCartney/Special to the Eagle")
        );
        assert_eq!(
            out.copyright.as_deref(),
            Some("\u{a9} 2026 Caleb McCartney")
        );
        assert_eq!(
            out.keywords,
            Some(vec![
                "103126 SPO FB A-B".into(),
                "{not_a_variable}".into(),
                "football".into()
            ]),
            "{{headline}} is filled in, an unknown name stays as typed"
        );
        // and written to the file, the variables are gone
        write_iptc(&items[0], &pad).unwrap();
        let back = iptc::parse(&String::from_utf8_lossy(
            &fs::read(dir.path().join("DSC_0001.JPG")).unwrap(),
        ));
        assert!(
            back.description
                .as_deref()
                .unwrap_or("")
                .contains("Friday, Oct. 30, 2026"),
            "{back:?}"
        );
    }
}
