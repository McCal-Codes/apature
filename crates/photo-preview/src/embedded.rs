//! Locate JPEG previews embedded in TIFF-based RAW files.

use photo_core::tiff::{tag, ByteOrder, Ifd, Tiff};

use crate::jpeg_info;

/// One JPEG blob found inside a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmbeddedJpeg {
    pub offset: usize,
    pub length: usize,
    pub width: u32,
    pub height: u32,
    /// Where it came from, for diagnostics ("IFD0", "SubIFD1", "MakerNote", "file").
    pub source: String,
}

impl EmbeddedJpeg {
    pub fn long_side(&self) -> u32 {
        self.width.max(self.height)
    }
    pub fn bytes<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        &data[self.offset..self.offset + self.length]
    }
}

/// Find every embedded JPEG in `data`, largest first.
///
/// Handles: a plain JPEG file (returned as one candidate covering the whole
/// file); TIFF/RAW IFD0 and its chain; SubIFDs (where Nikon puts the
/// full-size preview); the Exif IFD; and Nikon MakerNote `PreviewIFD`
/// (the ~1620px preview, ideal for thumbnails).
pub fn find_embedded_jpegs(data: &[u8]) -> Vec<EmbeddedJpeg> {
    let mut out = Vec::new();

    if jpeg_info::is_jpeg(data) {
        let (w, h) = jpeg_info::dimensions(data).unwrap_or((0, 0));
        out.push(EmbeddedJpeg {
            offset: 0,
            length: data.len(),
            width: w,
            height: h,
            source: "file".into(),
        });
        return out;
    }

    let Ok((tiff, first)) = Tiff::parse(data) else {
        return out;
    };

    let mut visited = Vec::new();
    let mut ifd_offset = first;
    let mut n = 0;
    while ifd_offset != 0 && n < 16 {
        walk_ifd(
            &tiff,
            ifd_offset,
            &format!("IFD{n}"),
            0,
            &mut out,
            &mut visited,
        );
        let next = tiff.read_ifd(ifd_offset).map(|i| i.next).unwrap_or(0);
        if next == ifd_offset {
            break;
        }
        ifd_offset = next;
        n += 1;
    }

    finish(data, out)
}

fn finish(data: &[u8], mut out: Vec<EmbeddedJpeg>) -> Vec<EmbeddedJpeg> {
    // Validate, fill in dimensions, dedupe by offset, sort largest first.
    out.retain(|c| {
        c.length > 4
            && c.offset
                .checked_add(c.length)
                .map(|end| end <= data.len())
                .unwrap_or(false)
            && jpeg_info::is_jpeg(&data[c.offset..])
    });
    for c in out.iter_mut() {
        if c.width == 0 || c.height == 0 {
            if let Some((w, h)) = jpeg_info::dimensions(c.bytes(data)) {
                c.width = w;
                c.height = h;
            }
        }
    }
    out.sort_by_key(|c| c.offset);
    out.dedup_by_key(|c| c.offset);
    out.sort_by(|a, b| {
        b.long_side()
            .cmp(&a.long_side())
            .then(b.length.cmp(&a.length))
    });
    out
}

fn walk_ifd(
    tiff: &Tiff,
    offset: u32,
    name: &str,
    depth: u8,
    out: &mut Vec<EmbeddedJpeg>,
    visited: &mut Vec<u32>,
) {
    if depth > 4 || visited.contains(&offset) || visited.len() > 64 {
        return;
    }
    visited.push(offset);
    let Ok(ifd) = tiff.read_ifd(offset) else {
        return;
    };

    candidates_in_ifd(tiff, &ifd, name, out);

    if let Some(e) = ifd.get(tag::SUB_IFDS) {
        for (i, sub) in tiff.uints(e).into_iter().enumerate().take(8) {
            walk_ifd(
                tiff,
                sub as u32,
                &format!("{name}/SubIFD{i}"),
                depth + 1,
                out,
                visited,
            );
        }
    }
    if let Some(e) = ifd.get(tag::EXIF_IFD) {
        if let Some(exif_off) = tiff.uint(e, 0) {
            walk_ifd(
                tiff,
                exif_off as u32,
                &format!("{name}/Exif"),
                depth + 1,
                out,
                visited,
            );
        }
    }
    if let Some(e) = ifd.get(tag::MAKER_NOTE) {
        nikon_makernote(tiff, e, out);
    }
}

fn candidates_in_ifd(tiff: &Tiff, ifd: &Ifd, name: &str, out: &mut Vec<EmbeddedJpeg>) {
    let width = ifd
        .get(tag::IMAGE_WIDTH)
        .and_then(|e| tiff.uint(e, 0))
        .unwrap_or(0) as u32;
    let height = ifd
        .get(tag::IMAGE_LENGTH)
        .and_then(|e| tiff.uint(e, 0))
        .unwrap_or(0) as u32;

    // Old-style JPEG pointer pair (Exif thumbnails, Nikon previews).
    if let (Some(o), Some(l)) = (
        ifd.get(tag::JPEG_INTERCHANGE_FORMAT),
        ifd.get(tag::JPEG_INTERCHANGE_FORMAT_LENGTH),
    ) {
        if let (Some(off), Some(len)) = (tiff.uint(o, 0), tiff.uint(l, 0)) {
            out.push(EmbeddedJpeg {
                offset: off as usize,
                length: len as usize,
                width: 0,
                height: 0,
                source: format!("{name}:JPEGInterchangeFormat"),
            });
        }
    }

    // Strip-based JPEG (compression 6 = old JPEG, 7 = new JPEG). Nikon SubIFD0,
    // Canon CR2 IFD0 and DNG previews look like this: a single strip holding
    // one whole JPEG stream.
    let compression = ifd.get(tag::COMPRESSION).and_then(|e| tiff.uint(e, 0));
    if matches!(compression, Some(6) | Some(7)) {
        if let (Some(o), Some(l)) = (ifd.get(tag::STRIP_OFFSETS), ifd.get(tag::STRIP_BYTE_COUNTS)) {
            if o.count == 1 && l.count == 1 {
                if let (Some(off), Some(len)) = (tiff.uint(o, 0), tiff.uint(l, 0)) {
                    out.push(EmbeddedJpeg {
                        offset: off as usize,
                        length: len as usize,
                        width,
                        height,
                        source: format!("{name}:strip"),
                    });
                }
            }
        }
    }
}

/// Nikon MakerNote: "Nikon\0" + 4 bytes version, then a complete TIFF header
/// at +10 whose offsets are relative to that header. Tag 0x0011 points at a
/// PreviewIFD holding a JPEGInterchangeFormat pair, also relative to +10.
fn nikon_makernote(tiff: &Tiff, e: &photo_core::tiff::Entry, out: &mut Vec<EmbeddedJpeg>) {
    let Some(mn) = tiff.value_bytes(e) else {
        return;
    };
    if mn.len() < 18 || !mn.starts_with(b"Nikon\0") {
        return;
    }
    let base_rel = 10usize;
    let Some(inner) = mn.get(base_rel..) else {
        return;
    };
    let Ok((inner_tiff, first)) = Tiff::parse(inner) else {
        return;
    };
    let Ok(ifd) = inner_tiff.read_ifd(first) else {
        return;
    };
    let Some(pv) = ifd.get(0x0011) else { return };
    let Some(pv_off) = inner_tiff.uint(pv, 0) else {
        return;
    };
    let Ok(pifd) = inner_tiff.read_ifd(pv_off as u32) else {
        return;
    };
    let (Some(o), Some(l)) = (
        pifd.get(tag::JPEG_INTERCHANGE_FORMAT),
        pifd.get(tag::JPEG_INTERCHANGE_FORMAT_LENGTH),
    ) else {
        return;
    };
    let (Some(off), Some(len)) = (inner_tiff.uint(o, 0), inner_tiff.uint(l, 0)) else {
        return;
    };
    // Translate from makernote-relative to file-relative.
    let mn_start = mn.as_ptr() as usize - tiff.data().as_ptr() as usize;
    out.push(EmbeddedJpeg {
        offset: mn_start + base_rel + off as usize,
        length: len as usize,
        width: 0,
        height: 0,
        source: "MakerNote:PreviewIFD".into(),
    });
    let _ = ByteOrder::Little; // keep import used on all paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use photo_core::tiff::testutil::build_le;

    fn fake_jpeg(w: u16, h: u16) -> Vec<u8> {
        let mut v = vec![0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x11, 8];
        v.extend_from_slice(&h.to_be_bytes());
        v.extend_from_slice(&w.to_be_bytes());
        v.extend_from_slice(&[3, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xD9]);
        v
    }

    #[test]
    fn plain_jpeg_is_its_own_preview() {
        let j = fake_jpeg(300, 200);
        let found = find_embedded_jpegs(&j);
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].width, found[0].height), (300, 200));
        assert_eq!(found[0].source, "file");
    }

    #[test]
    fn finds_interchange_and_strip_jpegs_in_tiff() {
        let small = fake_jpeg(160, 120);
        let big = fake_jpeg(6000, 4000);
        let payload_off = 200usize;
        let big_off = payload_off + small.len();
        let mut payload = small.clone();
        payload.extend_from_slice(&big);
        let data = build_le(
            &[
                (tag::COMPRESSION, 3, 1, [6, 0, 0, 0]),
                (tag::STRIP_OFFSETS, 4, 1, (big_off as u32).to_le_bytes()),
                (
                    tag::STRIP_BYTE_COUNTS,
                    4,
                    1,
                    (big.len() as u32).to_le_bytes(),
                ),
                (
                    tag::JPEG_INTERCHANGE_FORMAT,
                    4,
                    1,
                    (payload_off as u32).to_le_bytes(),
                ),
                (
                    tag::JPEG_INTERCHANGE_FORMAT_LENGTH,
                    4,
                    1,
                    (small.len() as u32).to_le_bytes(),
                ),
            ],
            &payload,
            payload_off,
        );
        let found = find_embedded_jpegs(&data);
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!(found[0].long_side(), 6000);
        assert_eq!(found[1].long_side(), 160);
        assert_eq!(found[0].bytes(&data), &big[..]);
    }

    #[test]
    fn ignores_out_of_range_pointers() {
        let data = build_le(
            &[
                (tag::JPEG_INTERCHANGE_FORMAT, 4, 1, 100_000u32.to_le_bytes()),
                (
                    tag::JPEG_INTERCHANGE_FORMAT_LENGTH,
                    4,
                    1,
                    10u32.to_le_bytes(),
                ),
            ],
            b"",
            0,
        );
        assert!(find_embedded_jpegs(&data).is_empty());
        assert!(find_embedded_jpegs(b"").is_empty());
    }
}
