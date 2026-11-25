//! A small ISO base media file format (ISO-BMFF, the MP4 family) box walker:
//! just enough to find what a Canon CR3 carries without decoding it. CR3 is
//! not TIFF, so the IFD reader cannot see into it (ADR-002 named this
//! second locator).
//!
//! The layout this reads (Canon's CR3, as documented by Laurent Clévy's
//! "Describing the Canon Raw v3 (CR3) file format" and matched by exiftool):
//!
//! * `ftyp` with major brand `crx `.
//! * `moov` / `uuid` 85c0b687-820f-11e0-8111-f4ce462b6a48 holds `CMT1`
//!   (IFD0 as a whole TIFF), `CMT2` (the Exif IFD as a whole TIFF), `CMT3`
//!   (Canon maker notes), `CMT4` (GPS) and `THMB` (a 160×120 JPEG).
//! * `moov` / `trak` 1 points (`stsz`, `co64`) at a full-size JPEG in `mdat`;
//!   the other tracks are the RAW data and metadata.
//! * A top-level `uuid` eaf42b5e-1c98-4b88-b9fb-b7dc406e4d16 holds `PRVW`,
//!   a 1620×1080 JPEG.
//!
//! Everything is bounds-checked; a hostile or truncated file yields fewer
//! results, never a panic.

/// Canon's metadata box inside `moov`.
pub const CANON_UUID: [u8; 16] = [
    0x85, 0xc0, 0xb6, 0x87, 0x82, 0x0f, 0x11, 0xe0, 0x81, 0x11, 0xf4, 0xce, 0x46, 0x2b, 0x6a, 0x48,
];
/// The top-level box that holds the PRVW preview.
pub const PREVIEW_UUID: [u8; 16] = [
    0xea, 0xf4, 0x2b, 0x5e, 0x1c, 0x98, 0x4b, 0x88, 0xb9, 0xfb, 0xb7, 0xdc, 0x40, 0x6e, 0x4d, 0x16,
];

/// Upper bound on boxes read at one level, so a crafted file cannot make the
/// walk expensive.
const MAX_BOXES: usize = 1024;

/// One box: its type, its uuid for `uuid` boxes, and its payload.
#[derive(Clone, Copy, Debug)]
pub struct BoxRef<'a> {
    pub kind: [u8; 4],
    pub uuid: Option<[u8; 16]>,
    /// Offset of the payload in the outermost buffer.
    pub offset: usize,
    pub payload: &'a [u8],
}

/// The boxes directly inside `data`, where `data` starts at `base` in the
/// outermost buffer. Stops at the first malformed header.
pub fn children(data: &[u8], base: usize) -> Vec<BoxRef<'_>> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while out.len() < MAX_BOXES && data.len().saturating_sub(pos) >= 8 {
        let size32 = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as u64;
        let kind: [u8; 4] = data[pos + 4..pos + 8].try_into().unwrap();
        let rest = (data.len() - pos) as u64;
        let mut header = 8usize;
        let size = match size32 {
            0 => rest,
            1 => {
                let Some(b) = data.get(pos + 8..pos + 16) else {
                    break;
                };
                header = 16;
                u64::from_be_bytes(b.try_into().unwrap())
            }
            n => n,
        };
        if size < header as u64 || size > rest {
            break;
        }
        let size = size as usize;
        let mut uuid = None;
        if &kind == b"uuid" {
            let Some(u) = data.get(pos + header..pos + header + 16) else {
                break;
            };
            if size < header + 16 {
                break;
            }
            uuid = Some(u.try_into().unwrap());
            header += 16;
        }
        out.push(BoxRef {
            kind,
            uuid,
            offset: base + pos + header,
            payload: &data[pos + header..pos + size],
        });
        pos += size;
    }
    out
}

/// True for a Canon CR3 (an ISO-BMFF file whose major brand is `crx `).
pub fn is_cr3(data: &[u8]) -> bool {
    data.len() >= 12 && &data[4..8] == b"ftyp" && &data[8..12] == b"crx "
}

/// What a CR3 holds that Apature reads: the TIFF metadata blocks and the
/// byte ranges where JPEG previews may start (each still to be checked).
#[derive(Debug, Default)]
pub struct Cr3<'a> {
    /// `CMT1`: IFD0 (make, model, orientation) as a whole TIFF.
    pub ifd0: Option<&'a [u8]>,
    /// `CMT2`: the Exif IFD (capture time, exposure, lens) as a whole TIFF.
    pub exif: Option<&'a [u8]>,
    /// (offset, length, where) of each JPEG candidate in the file.
    pub jpegs: Vec<(usize, usize, &'static str)>,
}

/// Index a CR3. `None` when `data` is not one.
pub fn cr3(data: &[u8]) -> Option<Cr3<'_>> {
    if !is_cr3(data) {
        return None;
    }
    let mut out = Cr3::default();
    for top in children(data, 0) {
        match &top.kind {
            b"moov" => {
                for b in children(top.payload, top.offset) {
                    if b.uuid == Some(CANON_UUID) {
                        canon_box(&b, &mut out);
                    } else if &b.kind == b"trak" {
                        if let Some(r) = first_sample(&b) {
                            out.jpegs.push((r.0, r.1, "CR3:trak"));
                        }
                    }
                }
            }
            b"uuid" if top.uuid == Some(PREVIEW_UUID) => {
                // 8 bytes Canon does not document, then the PRVW box.
                if let Some(inner) = top.payload.get(8..) {
                    for b in children(inner, top.offset + 8) {
                        if &b.kind == b"PRVW" {
                            if let Some(r) = jpeg_in(&b) {
                                out.jpegs.push((r.0, r.1, "CR3:PRVW"));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Some(out)
}

fn canon_box<'a>(b: &BoxRef<'a>, out: &mut Cr3<'a>) {
    for c in children(b.payload, b.offset) {
        match &c.kind {
            b"CMT1" => out.ifd0 = Some(c.payload),
            b"CMT2" => out.exif = Some(c.payload),
            b"THMB" => {
                if let Some(r) = jpeg_in(&c) {
                    out.jpegs.push((r.0, r.1, "CR3:THMB"));
                }
            }
            _ => {}
        }
    }
}

/// PRVW and THMB start with a few header words (size, dimensions) before
/// the JPEG; find its start marker near the front and run to the box end.
fn jpeg_in(b: &BoxRef) -> Option<(usize, usize)> {
    let head = &b.payload[..b.payload.len().min(64)];
    let at = head.windows(3).position(|w| w == [0xFF, 0xD8, 0xFF])?;
    Some((b.offset + at, b.payload.len() - at))
}

/// trak / mdia / minf / stbl: the first sample's file offset and size.
fn first_sample(trak: &BoxRef) -> Option<(usize, usize)> {
    let mdia = find(trak, b"mdia")?;
    let minf = find(&mdia, b"minf")?;
    let stbl = find(&minf, b"stbl")?;
    let kids = children(stbl.payload, stbl.offset);
    let stsz = kids.iter().find(|b| &b.kind == b"stsz")?;
    // version/flags, sample_size, sample_count, then sizes when sample_size == 0.
    let fixed = be32(stsz.payload, 4)?;
    let size = if fixed != 0 {
        fixed
    } else {
        be32(stsz.payload, 12)?
    };
    let offset = if let Some(co64) = kids.iter().find(|b| &b.kind == b"co64") {
        // version/flags, entry_count, then u64 offsets.
        let b = co64.payload.get(8..16)?;
        u64::from_be_bytes(b.try_into().ok()?)
    } else {
        let stco = kids.iter().find(|b| &b.kind == b"stco")?;
        be32(stco.payload, 8)? as u64
    };
    Some((usize::try_from(offset).ok()?, size as usize))
}

fn find<'a>(b: &BoxRef<'a>, kind: &[u8; 4]) -> Option<BoxRef<'a>> {
    children(b.payload, b.offset)
        .into_iter()
        .find(|c| &c.kind == kind)
}

fn be32(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

/// Builders for synthetic CR3 files, for tests in this and other crates.
pub mod testutil {
    use super::{CANON_UUID, PREVIEW_UUID};

    pub fn bx(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut v = ((payload.len() + 8) as u32).to_be_bytes().to_vec();
        v.extend_from_slice(kind);
        v.extend_from_slice(payload);
        v
    }

    pub fn uuid_box(uuid: &[u8; 16], payload: &[u8]) -> Vec<u8> {
        let mut p = uuid.to_vec();
        p.extend_from_slice(payload);
        bx(b"uuid", &p)
    }

    /// A CR3 laid out the way Canon writes one: ftyp, moov (Canon uuid with
    /// CMT1, CMT2 and THMB; one trak pointing into mdat), the PRVW uuid,
    /// then mdat holding `full`.
    pub fn cr3(cmt1: &[u8], cmt2: &[u8], thmb: &[u8], prvw: &[u8], full: &[u8]) -> Vec<u8> {
        let build = |mdat_payload_at: u64| {
            let mut ftyp = b"crx ".to_vec();
            ftyp.extend_from_slice(&[0, 0, 0, 1]);
            ftyp.extend_from_slice(b"crx isom");
            let mut canon = bx(b"CMT1", cmt1);
            canon.extend(bx(b"CMT2", cmt2));
            // THMB: version/flags, width, height, jpeg size, then the JPEG.
            let mut th = vec![0, 0, 0, 0, 0, 160, 0, 120];
            th.extend_from_slice(&(thmb.len() as u32).to_be_bytes());
            th.extend_from_slice(thmb);
            canon.extend(bx(b"THMB", &th));
            let mut stsz = vec![0u8; 4];
            stsz.extend_from_slice(&(full.len() as u32).to_be_bytes());
            stsz.extend_from_slice(&1u32.to_be_bytes());
            let mut co64 = vec![0, 0, 0, 0, 0, 0, 0, 1];
            co64.extend_from_slice(&mdat_payload_at.to_be_bytes());
            let mut stbl = bx(b"stsz", &stsz);
            stbl.extend(bx(b"co64", &co64));
            let trak = bx(b"trak", &bx(b"mdia", &bx(b"minf", &bx(b"stbl", &stbl))));
            let mut moov = uuid_box(&CANON_UUID, &canon);
            moov.extend(trak);
            // PRVW: an unknown u32, width, height, an unknown u16, jpeg size, then the JPEG.
            let mut pv = vec![0, 0, 0, 0, 0x06, 0x54, 0x04, 0x38, 0, 1];
            pv.extend_from_slice(&(prvw.len() as u32).to_be_bytes());
            pv.extend_from_slice(prvw);
            let mut prvw_uuid = vec![0u8; 8];
            prvw_uuid.extend(bx(b"PRVW", &pv));
            let mut v = bx(b"ftyp", &ftyp);
            v.extend(bx(b"moov", &moov));
            v.extend(uuid_box(&PREVIEW_UUID, &prvw_uuid));
            let at = v.len() as u64 + 8;
            v.extend(bx(b"mdat", full));
            (v, at)
        };
        let (_, at) = build(0);
        build(at).0
    }
}

#[cfg(test)]
mod tests {
    use super::testutil::{bx, cr3 as build};
    use super::*;

    #[test]
    fn reads_canon_layout() {
        let data = build(
            b"II*\0one",
            b"II*\0two",
            b"\xFF\xD8\xFFthumb",
            b"\xFF\xD8\xFFpreview",
            b"\xFF\xD8\xFFfull",
        );
        let c = cr3(&data).expect("a CR3");
        assert_eq!(c.ifd0, Some(&b"II*\0one"[..]));
        assert_eq!(c.exif, Some(&b"II*\0two"[..]));
        let got: Vec<(&[u8], &str)> = c
            .jpegs
            .iter()
            .map(|&(o, l, w)| (&data[o..o + l], w))
            .collect();
        assert!(
            got.contains(&(&b"\xFF\xD8\xFFthumb"[..], "CR3:THMB")),
            "{got:?}"
        );
        assert!(
            got.contains(&(&b"\xFF\xD8\xFFpreview"[..], "CR3:PRVW")),
            "{got:?}"
        );
        assert!(
            got.contains(&(&b"\xFF\xD8\xFFfull"[..], "CR3:trak")),
            "{got:?}"
        );
    }

    #[test]
    fn other_files_are_not_cr3() {
        assert!(cr3(b"").is_none());
        assert!(cr3(b"II*\0\x08\0\0\0").is_none());
        let mut mp4 = bx(b"ftyp", b"isom\0\0\0\x01isom");
        mp4.extend(bx(b"moov", b""));
        assert!(cr3(&mp4).is_none());
    }

    #[test]
    fn truncated_and_lying_sizes_do_not_panic() {
        let data = build(
            b"II*\0",
            b"II*\0",
            b"\xFF\xD8\xFFt",
            b"\xFF\xD8\xFFp",
            b"\xFF\xD8\xFFf",
        );
        for cut in 0..data.len() {
            let _ = cr3(&data[..cut]);
        }
        // A box that claims to be larger than the file, and a 64-bit size of 0.
        let mut bad = bx(b"ftyp", b"crx \0\0\0\x01");
        bad.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        bad.extend_from_slice(b"moov");
        let _ = cr3(&bad);
        let mut big = bx(b"ftyp", b"crx \0\0\0\x01");
        big.extend_from_slice(&[0, 0, 0, 1]);
        big.extend_from_slice(b"moov");
        big.extend_from_slice(&0u64.to_be_bytes());
        let _ = cr3(&big);
    }

    #[test]
    fn reported_ranges_start_inside_the_file() {
        // The preview locator still checks each range is a JPEG before use.
        let data = build(b"", b"", b"", b"", b"\xFF\xD8\xFFf");
        let c = cr3(&data).unwrap();
        assert!(c.jpegs.iter().all(|&(o, _, _)| o <= data.len()));
    }
}
