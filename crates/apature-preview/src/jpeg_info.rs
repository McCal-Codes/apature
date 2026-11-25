//! Read a JPEG's pixel dimensions from its headers without decoding.

/// Scan JPEG markers up to the first SOF and return `(width, height)`.
/// Returns `None` if `bytes` isn't a JPEG or no SOF is found before the
/// first scan. The walk skips whole segments, so even a camera JPEG with a
/// multi-megabyte APP1 (Nikon MakerNote plus embedded preview) costs a
/// handful of reads.
pub fn dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return None;
    }
    let limit = bytes.len();
    let mut i = 2;
    while i + 4 <= limit {
        if bytes[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = bytes[i + 1];
        // Padding bytes.
        if marker == 0xFF {
            i += 1;
            continue;
        }
        // Standalone markers without a length.
        if matches!(marker, 0xD8 | 0x01 | 0xD0..=0xD7) {
            i += 2;
            continue;
        }
        let len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
        if len < 2 {
            return None;
        }
        // SOF0..SOF15, excluding DHT (C4), JPG (C8), DAC (CC).
        if (0xC0..=0xCF).contains(&marker) && !matches!(marker, 0xC4 | 0xC8 | 0xCC) {
            if i + 9 > bytes.len() {
                return None;
            }
            let h = u16::from_be_bytes([bytes[i + 5], bytes[i + 6]]) as u32;
            let w = u16::from_be_bytes([bytes[i + 7], bytes[i + 8]]) as u32;
            return Some((w, h));
        }
        if marker == 0xDA {
            // Start of scan: no SOF seen, give up.
            return None;
        }
        i += 2 + len;
    }
    None
}

/// True if `bytes` starts with a JPEG SOI marker.
pub fn is_jpeg(bytes: &[u8]) -> bool {
    bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF
}

/// The EXIF orientation (1..=8) in a JPEG's APP1 segment, 1 when there is
/// none. Used for JPEGs another program wrote, whose pixels may be stored
/// sideways.
pub fn orientation(bytes: &[u8]) -> u32 {
    use apature_core::tiff::{tag, Tiff};
    if !is_jpeg(bytes) {
        return 1;
    }
    let mut i = 2;
    while i + 4 <= bytes.len() && bytes[i] == 0xFF {
        let marker = bytes[i + 1];
        if marker == 0xDA || marker == 0xD9 {
            break;
        }
        let len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
        let Some(seg) = bytes.get(i + 4..i + 2 + len) else {
            break;
        };
        if marker == 0xE1 && seg.starts_with(b"Exif\0\0") {
            let o = Tiff::parse(&seg[6..])
                .ok()
                .and_then(|(t, first)| {
                    let ifd = t.read_ifd(first).ok()?;
                    t.uint(ifd.get(tag::ORIENTATION)?, 0)
                })
                .unwrap_or(1) as u32;
            return if (1..=8).contains(&o) { o } else { 1 };
        }
        i += 2 + len;
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_sof0_dimensions() {
        // SOI, APP0 (len 16), SOF0 (len 17: precision, h=480, w=640, 3 comps...), then junk.
        let mut v = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        v.extend_from_slice(&[0u8; 14]);
        v.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x11, 8, 0x01, 0xE0, 0x02, 0x80, 3]);
        v.extend_from_slice(&[0u8; 9]);
        assert_eq!(dimensions(&v), Some((640, 480)));
        assert!(is_jpeg(&v));
        assert_eq!(dimensions(b"II*\0"), None);
    }
}
