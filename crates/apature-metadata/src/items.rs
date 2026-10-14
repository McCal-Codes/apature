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
pub fn write_iptc(item: &PhotoItem, fields: &Iptc) -> Result<Vec<PathBuf>, ItemError> {
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
}
