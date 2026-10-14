//! Delivery export: one photo in, one JPEG out, sized for the desk, with
//! EXIF, XMP, IPTC-IIM and the ICC profile carried across.
//!
//! Pixels come from the JPEG of a pair or a plain JPEG; a RAW alone uses its
//! largest embedded preview (never a RAW decode: there is no copyleft-free
//! demosaic in this workspace and the camera's own rendering is what the
//! photographer saw). The image is rotated upright, resized so the long side
//! is `Settings::long_side`, and re-encoded at `Settings::quality`.
//!
//! Colour: the source's ICC profile is copied (an Adobe RGB JPEG stays
//! Adobe RGB and tagged); no conversion to sRGB happens for JPEG sources.
//!
//! RAW decode: with [`RawPixels::SystemDecode`] a RAW alone is rendered by
//! the operating system's RAW engine (macOS ImageIO through `sips`,
//! matched to sRGB) instead of its embedded preview. Slower, full quality,
//! macOS only; other platforms fall back to the preview and say so.

use std::path::{Path, PathBuf};

use apature_core::{Kind, PhotoItem};
use apature_metadata::jpeg_xmp::{header_segments, is_exif_segment};
use apature_metadata::{exif, exif_write, iim, iptc, Iptc};
use apature_preview::{decode_jpeg, find_embedded_jpegs, rotate, Rgb};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RawPixels {
    /// The camera's embedded full-size JPEG (fast, the camera's rendering).
    #[default]
    EmbeddedPreview,
    /// Demosaic with the system RAW engine (macOS), matched to sRGB.
    SystemDecode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Long side in pixels; 0 keeps the source size.
    pub long_side: u32,
    /// JPEG quality 1..=100.
    pub quality: u8,
    pub keep_exif: bool,
    pub write_xmp: bool,
    pub write_iim: bool,
    pub keep_icc: bool,
    pub overwrite: bool,
    pub raw: RawPixels,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            long_side: 2048,
            quality: 90,
            keep_exif: true,
            write_xmp: true,
            write_iim: true,
            keep_icc: true,
            overwrite: false,
            raw: RawPixels::EmbeddedPreview,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExifSource {
    /// Copied from the source JPEG, orientation and size patched.
    Copied,
    /// Built from the EXIF summary (RAW-only source).
    Minimal,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelSource {
    Jpeg,
    EmbeddedPreview,
    SystemRawDecode,
}

#[derive(Clone, Debug)]
pub struct Outcome {
    pub source: PathBuf,
    pub pixels: PixelSource,
    pub out: PathBuf,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
    pub exif: ExifSource,
    pub millis: u128,
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("{0}: no JPEG and no embedded preview to export")]
    NoPixels(PathBuf),
    #[error("{0}: HEIC export is not supported yet")]
    Heic(PathBuf),
    #[error("{0} exists; export does not overwrite")]
    Exists(PathBuf),
    #[error("{0}: {1}")]
    Io(PathBuf, std::io::Error),
    #[error("decode {0}: {1}")]
    Decode(PathBuf, apature_preview::DecodeError),
    #[error("encode: {0}")]
    Encode(image::ImageError),
    #[error("metadata: {0}")]
    Xmp(#[from] apature_metadata::xmp::XmpError),
    #[error("{0}")]
    Jpeg(#[from] apature_metadata::jpeg_xmp::JpegXmpError),
    #[error("system RAW decode of {0} failed: {1}")]
    RawDecode(PathBuf, String),
}

/// Render a RAW with the macOS RAW engine (ImageIO via `sips`) to a JPEG at
/// quality 100, matched to sRGB, optionally resampled so the long side is
/// `long_side` (0 = full size). Returns the JPEG bytes.
#[cfg(target_os = "macos")]
pub fn system_decode_raw(path: &Path, long_side: u32) -> Result<Vec<u8>, String> {
    let tmp = std::env::temp_dir().join(format!(
        "apature-raw-{}-{}.jpg",
        std::process::id(),
        path.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    ));
    let mut cmd = std::process::Command::new("/usr/bin/sips");
    cmd.args(["-s", "format", "jpeg", "-s", "formatOptions", "100"]);
    cmd.args([
        "--matchTo",
        "/System/Library/ColorSync/Profiles/sRGB Profile.icc",
    ]);
    if long_side > 0 {
        cmd.args(["--resampleHeightWidthMax", &long_side.to_string()]);
    }
    cmd.arg(path).arg("--out").arg(&tmp);
    let out = cmd.output().map_err(|e| e.to_string())?;
    let result = if out.status.success() && tmp.is_file() {
        std::fs::read(&tmp).map_err(|e| e.to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    };
    let _ = std::fs::remove_file(&tmp);
    result
}

#[cfg(not(target_os = "macos"))]
pub fn system_decode_raw(_path: &Path, _long_side: u32) -> Result<Vec<u8>, String> {
    Err("system RAW decode is only available on macOS".into())
}

/// Which file supplies the pixels, and whether it is a real JPEG (with its
/// own EXIF/ICC) or a preview inside a RAW.
fn pixel_source(item: &PhotoItem) -> Result<(PathBuf, bool), ExportError> {
    if let Some(v) = item.variants().find(|v| v.kind == Kind::Jpeg) {
        return Ok((v.path.clone(), true));
    }
    match item.primary.kind {
        Kind::Raw => Ok((item.primary.path.clone(), false)),
        Kind::Heic => Err(ExportError::Heic(item.primary.path.clone())),
        Kind::Jpeg => Ok((item.primary.path.clone(), true)),
    }
}

fn resize_to(img: Rgb, long_side: u32) -> Rgb {
    if long_side == 0 || img.long_side() <= long_side {
        return img;
    }
    let s = long_side as f64 / img.long_side() as f64;
    let w = ((img.width as f64) * s).round().max(1.0) as u32;
    let h = ((img.height as f64) * s).round().max(1.0) as u32;
    let Some(src) = img.into_image() else {
        return Rgb {
            width: 0,
            height: 0,
            data: Vec::new(),
        };
    };
    let out = image::imageops::resize(&src, w, h, image::imageops::FilterType::Lanczos3);
    Rgb::from_image(out)
}

fn app1(sig: &[u8], body: &[u8]) -> Vec<u8> {
    let len = 2 + sig.len() + body.len();
    let mut v = Vec::with_capacity(len + 2);
    v.extend_from_slice(&[0xFF, 0xE1]);
    v.extend_from_slice(&(len as u16).to_be_bytes());
    v.extend_from_slice(sig);
    v.extend_from_slice(body);
    v
}

/// Export `item` to `out` with `settings`, embedding `iptc` as XMP and IIM.
pub fn export(
    item: &PhotoItem,
    iptc: &Iptc,
    settings: &Settings,
    out: &Path,
) -> Result<Outcome, ExportError> {
    let t0 = std::time::Instant::now();
    if out.exists() && !settings.overwrite {
        return Err(ExportError::Exists(out.to_path_buf()));
    }
    let (src_path, mut is_jpeg) = pixel_source(item)?;
    let mut pixels = if is_jpeg {
        PixelSource::Jpeg
    } else {
        PixelSource::EmbeddedPreview
    };
    let mut data = std::fs::read(&src_path).map_err(|e| ExportError::Io(src_path.clone(), e))?;
    if !is_jpeg && settings.raw == RawPixels::SystemDecode {
        match system_decode_raw(&src_path, settings.long_side) {
            Ok(rendered) => {
                data = rendered;
                is_jpeg = true;
                pixels = PixelSource::SystemRawDecode;
            }
            Err(e) => {
                log::warn!("{}: {e}; using the embedded preview", src_path.display());
            }
        }
    }
    let jpeg_bytes: &[u8] = if is_jpeg {
        &data
    } else {
        let best = find_embedded_jpegs(&data)
            .into_iter()
            .max_by_key(|e| e.long_side())
            .ok_or_else(|| ExportError::NoPixels(src_path.clone()))?;
        best.bytes(&data)
    };
    let summary = if pixels == PixelSource::SystemRawDecode {
        exif::from_bytes(&data).unwrap_or_default()
    } else {
        exif::read(&src_path).unwrap_or_else(|_| exif::from_bytes(&data).unwrap_or_default())
    };
    let target = if settings.long_side == 0 {
        None
    } else {
        Some(settings.long_side)
    };
    let img =
        decode_jpeg(jpeg_bytes, target).map_err(|e| ExportError::Decode(src_path.clone(), e))?;
    let img = rotate(img, summary.orientation);
    let img = resize_to(img, settings.long_side);
    let (width, height) = (img.width, img.height);
    let encoded = apature_preview::decode::encode_jpeg(&img, settings.quality.clamp(1, 100))
        .map_err(ExportError::Encode)?;

    // ---- metadata segments ----
    let mut segs: Vec<Vec<u8>> = Vec::new();
    let mut exif_source = ExifSource::None;
    let src_segs = if is_jpeg {
        header_segments(&data)?
    } else {
        Vec::new()
    };
    if settings.keep_exif {
        // A system RAW render carries the RAW's TIFF-flavoured IFD0, which
        // validators reject in a JPEG; build a clean block from the summary.
        let copied = if pixels == PixelSource::SystemRawDecode {
            None
        } else {
            src_segs
                .iter()
                .find(|s| is_exif_segment(&data, s))
                .map(|s| s.payload(&data)[6..].to_vec())
        };
        match copied {
            Some(mut block) => {
                exif_write::patch_for_export(&mut block, width, height);
                segs.push(app1(b"Exif\0\0", &block));
                exif_source = ExifSource::Copied;
            }
            None => {
                let block = exif_write::minimal(
                    &summary,
                    width,
                    height,
                    iptc.creator
                        .as_ref()
                        .and_then(|c| c.first())
                        .map(String::as_str),
                    iptc.copyright.as_deref(),
                );
                segs.push(app1(b"Exif\0\0", &block));
                exif_source = ExifSource::Minimal;
            }
        }
    }
    if settings.write_xmp {
        let packet = iptc::apply("", iptc)?;
        segs.push(app1(b"http://ns.adobe.com/xap/1.0/\0", packet.as_bytes()));
    }
    if settings.keep_icc {
        for s in src_segs
            .iter()
            .filter(|s| s.marker == 0xE2 && s.payload(&data).starts_with(b"ICC_PROFILE\0"))
        {
            segs.push(data[s.start..s.end].to_vec());
        }
    }
    if settings.write_iim && !iptc.is_empty() {
        segs.push(iim::app13_segment(iptc));
    }

    // ---- splice after SOI (and the encoder's APP0 when present) ----
    let mut ins = 2;
    if encoded.len() > 4 && encoded[2] == 0xFF && encoded[3] == 0xE0 {
        let len = u16::from_be_bytes([encoded[4], encoded[5]]) as usize;
        ins = 2 + 2 + len;
    }
    let mut file = Vec::with_capacity(encoded.len() + segs.iter().map(Vec::len).sum::<usize>());
    file.extend_from_slice(&encoded[..ins]);
    for s in &segs {
        file.extend_from_slice(s);
    }
    file.extend_from_slice(&encoded[ins..]);

    // ---- atomic write ----
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ExportError::Io(parent.to_path_buf(), e))?;
    }
    let tmp = out.with_extension("apature-export.tmp");
    let write = || -> std::io::Result<()> {
        use std::io::Write;
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(&file)?;
        f.sync_all()?;
        if out.exists() && !settings.overwrite {
            return Err(std::io::Error::other("destination appeared during export"));
        }
        std::fs::rename(&tmp, out)
    };
    if let Err(e) = write() {
        let _ = std::fs::remove_file(&tmp);
        return Err(ExportError::Io(out.to_path_buf(), e));
    }
    Ok(Outcome {
        source: src_path,
        pixels,
        out: out.to_path_buf(),
        width,
        height,
        bytes: file.len() as u64,
        exif: exif_source,
        millis: t0.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use apature_core::item::Variant;

    fn item_for(path: &Path) -> PhotoItem {
        PhotoItem {
            id: 0,
            stem: path.file_stem().unwrap().to_string_lossy().into_owned(),
            primary: Variant {
                path: path.to_path_buf(),
                kind: Kind::Jpeg,
                size: 0,
                modified: None,
            },
            secondary: Vec::new(),
        }
    }

    fn sample_iptc() -> Iptc {
        Iptc {
            description: Some("A caption. Photog/Butler Eagle".into()),
            credit: Some("Butler Eagle".into()),
            creator: Some(vec!["Caleb".into()]),
            copyright: Some("© 2026 Butler Eagle".into()),
            headline: Some("100726 SPO BGLF".into()),
            ..Default::default()
        }
    }

    #[test]
    fn synthetic_jpeg_resizes_and_carries_every_block() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("wide.jpg");
        let img = Rgb {
            width: 400,
            height: 200,
            data: (0..400 * 200 * 3).map(|i| (i % 251) as u8).collect(),
        };
        std::fs::write(
            &src,
            apature_preview::decode::encode_jpeg(&img, 85).unwrap(),
        )
        .unwrap();
        let out = dir.path().join("out/wide-export.jpg");
        let o = export(
            &item_for(&src),
            &sample_iptc(),
            &Settings {
                long_side: 100,
                ..Default::default()
            },
            &out,
        )
        .unwrap();
        assert_eq!((o.width, o.height), (100, 50));
        assert_eq!(
            o.exif,
            ExifSource::Minimal,
            "a plain encoder JPEG has no EXIF"
        );
        let bytes = std::fs::read(&out).unwrap();
        let segs = header_segments(&bytes).unwrap();
        let markers: Vec<u8> = segs.iter().map(|s| s.marker).collect();
        assert!(
            markers.contains(&0xE1) && markers.contains(&0xED),
            "{markers:?}"
        );
        assert!(segs.iter().any(|s| is_exif_segment(&bytes, s)));
        let packet = apature_metadata::xmp::embedded_packet_in_jpeg(&bytes).expect("xmp");
        let back = iptc::parse(packet);
        assert_eq!(
            back.description.as_deref(),
            Some("A caption. Photog/Butler Eagle")
        );
        let app13 = segs.iter().find(|s| s.marker == 0xED).unwrap();
        let sets = iim::parse(iim::from_app13(app13.payload(&bytes)).unwrap());
        assert!(sets
            .iter()
            .any(|(r, s, v)| *r == 2 && *s == 110 && v == b"Butler Eagle"));
        // Decodes again at the new size.
        let back = decode_jpeg(&bytes, None).unwrap();
        assert_eq!((back.width, back.height), (100, 50));
        // Refuses to overwrite.
        assert!(matches!(
            export(&item_for(&src), &sample_iptc(), &Settings::default(), &out),
            Err(ExportError::Exists(_))
        ));
        assert!(!dir
            .path()
            .join("out/wide-export.apature-export.tmp")
            .exists());
    }
}
