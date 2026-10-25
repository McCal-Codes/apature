//! Hostile and damaged input: the parsers read bytes from cards, drives and
//! other people's sidecars, so none of them may panic, hang or read out of
//! bounds. Deterministic (a fixed generator), so a failure reproduces.
//! A cargo-fuzz target can replace this later; these run in every `cargo test`.

use apature_metadata::{edits, exif, iim, jpeg_xmp, xmp};

/// xorshift64: reproducible noise without a dependency.
fn noise(seed: u64, len: usize) -> Vec<u8> {
    let mut x = seed | 1;
    (0..len)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x >> 24) as u8
        })
        .collect()
}

/// A JPEG that starts like a real one: SOI, then an EXIF APP1 header.
fn jpeg_prefix() -> Vec<u8> {
    let mut v = vec![0xFF, 0xD8, 0xFF, 0xE1, 0x00, 0x40];
    v.extend_from_slice(b"Exif\0\0II*\0\x08\0\0\0");
    v.extend(noise(7, 80));
    v
}

#[test]
fn exif_survives_noise_and_every_truncation() {
    for seed in 1..200u64 {
        let _ = exif::from_bytes(&noise(seed, (seed as usize * 37) % 4096));
    }
    let base = jpeg_prefix();
    for cut in 0..base.len() {
        let _ = exif::from_bytes(&base[..cut]);
    }
}

#[test]
fn exif_header_with_hostile_offsets_does_not_panic() {
    // Little-endian TIFF whose first IFD offset points far past the end.
    for off in [0u32, 8, 0xFFFF, 0x7FFF_FFFF, 0xFFFF_FFFF] {
        let mut v = vec![0xFF, 0xD8, 0xFF, 0xE1, 0x00, 0x20];
        v.extend_from_slice(b"Exif\0\0II*\0");
        v.extend_from_slice(&off.to_le_bytes());
        v.extend(noise(off as u64, 16));
        let _ = exif::from_bytes(&v);
    }
}

#[test]
fn xmp_and_edits_parse_garbage_text() {
    let mut cases: Vec<String> = vec![
        String::new(),
        "<".into(),
        "<x:xmpmeta".into(),
        "xmp:Rating=\"".into(),
        "xmp:Rating=\"99999999999999999999\"".into(),
        "<rdf:Description crs:CropTop=\"nan\" crs:CropLeft=\"inf\" crs:CropBottom=\"-1e999\" crs:HasCrop=\"True\"/>".into(),
        "apature:Straighten=\"1e999\"".into(),
        "tiff:Orientation=\"-3\"".into(),
    ];
    for seed in 1..100u64 {
        cases.push(String::from_utf8_lossy(&noise(seed, (seed as usize * 13) % 2000)).into_owned());
    }
    for c in &cases {
        let _ = xmp::parse(c);
        let e = edits::parse(c);
        assert!(
            e.straighten.is_finite(),
            "a straighten angle is always a finite number"
        );
        if let Some(crop) = e.crop {
            for v in [crop.top, crop.left, crop.bottom, crop.right] {
                assert!(
                    (0.0..=1.0).contains(&v),
                    "a crop is always inside the frame"
                );
            }
        }
    }
}

#[test]
fn iim_and_jpeg_segments_survive_noise() {
    for seed in 1..200u64 {
        let d = noise(seed, (seed as usize * 29) % 1024);
        let _ = iim::parse(&d);
        let _ = iim::from_app13(&d);
        let _ = jpeg_xmp::header_segments(&d);
        let _ = jpeg_xmp::packet(&d);
    }
    let base = jpeg_prefix();
    for cut in 0..base.len() {
        let _ = jpeg_xmp::header_segments(&base[..cut]);
        let _ = jpeg_xmp::packet(&base[..cut]);
    }
}
