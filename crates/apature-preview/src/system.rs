//! Pictures Apature cannot decode itself, handed to the operating system.
//!
//! HEIC/HEIF (iPhones, and Canon's and Sony's HEIF modes) stores the picture
//! as HEVC, a video codec with patent and licence strings attached, so
//! Apature does not ship a decoder (ADR-002 keeps the graph permissive).
//! Every desktop it runs on already has one:
//!
//! * Windows: WIC, through Microsoft's HEIF and HEVC Video Extensions (the
//!   HEVC one is a Store install on some PCs).
//! * macOS: ImageIO, through `/usr/bin/sips`.
//! * Linux: libheif's `heif-dec` (or the older `heif-convert`) when installed.
//!
//! What comes back is upright: the rotation Windows reports
//! (`System.Photo.Orientation`) and the orientation in the JPEG that `sips`
//! or libheif writes are applied here.

use std::path::Path;

use crate::decode::Rgb;

pub use apature_core::bmff::is_heif;

/// Decode `path` with the operating system, at most `long_side` pixels on
/// the long side (0 = full size), upright.
pub fn decode_with_os(path: &Path, long_side: u32) -> Result<Rgb, String> {
    imp::decode(path, long_side)
}

#[cfg(windows)]
mod imp {
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    use windows::core::{GUID, PCWSTR};
    use windows::Win32::Foundation::GENERIC_READ;
    use windows::Win32::Graphics::Imaging::*;
    use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PROPVARIANT};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_MULTITHREADED,
    };
    use windows::Win32::System::Variant::VT_UI2;

    use crate::decode::{rotate, Rgb};

    /// Balances a successful CoInitializeEx on this thread.
    struct Com(bool);
    impl Drop for Com {
        fn drop(&mut self) {
            if self.0 {
                unsafe { CoUninitialize() };
            }
        }
    }

    pub fn decode(path: &Path, long_side: u32) -> Result<Rgb, String> {
        // S_OK and S_FALSE both need a matching uninit; RPC_E_CHANGED_MODE
        // (the thread is already apartment-threaded) works as it is.
        let _com = Com(unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok());
        unsafe { decode_inner(path, long_side) }.map_err(|e| {
            format!(
                "Windows could not decode it ({}). HEIC needs the HEIF Image Extensions and \
                 HEVC Video Extensions from the Microsoft Store",
                e.message()
            )
        })
    }

    unsafe fn decode_inner(path: &Path, long_side: u32) -> windows::core::Result<Rgb> {
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        let decoder = factory.CreateDecoderFromFilename(
            PCWSTR(wide.as_ptr()),
            None,
            GENERIC_READ,
            WICDecodeMetadataCacheOnDemand,
        )?;
        let frame = decoder.GetFrame(0)?;
        let (mut w, mut h) = (0u32, 0u32);
        frame.GetSize(&mut w, &mut h)?;
        let orientation = photo_orientation(&frame);

        let source: IWICBitmapSource = if long_side > 0 && w.max(h) > long_side {
            let s = long_side as f64 / w.max(h) as f64;
            let (sw, sh) = (
                ((w as f64 * s).round() as u32).max(1),
                ((h as f64 * s).round() as u32).max(1),
            );
            let scaler = factory.CreateBitmapScaler()?;
            scaler.Initialize(&frame, sw, sh, WICBitmapInterpolationModeFant)?;
            (w, h) = (sw, sh);
            scaler.into()
        } else {
            frame.into()
        };
        let converter = factory.CreateFormatConverter()?;
        converter.Initialize(
            &source,
            &GUID_WICPixelFormat24bppRGB as *const GUID,
            WICBitmapDitherTypeNone,
            None,
            0.0,
            WICBitmapPaletteTypeCustom,
        )?;
        let stride = w * 3;
        let mut data = vec![0u8; (stride * h) as usize];
        converter.CopyPixels(std::ptr::null(), stride, &mut data)?;
        Ok(rotate(
            Rgb {
                width: w,
                height: h,
                data,
            },
            orientation,
        ))
    }

    /// The orientation WIC reports for the frame, 1 when it has none. WIC's
    /// HEIF decoder hands back pixels as stored and the rotation as this
    /// property, like a JPEG's EXIF tag.
    unsafe fn photo_orientation(frame: &IWICBitmapFrameDecode) -> u32 {
        let Ok(reader) = frame.GetMetadataQueryReader() else {
            return 1;
        };
        let name: Vec<u16> = "System.Photo.Orientation"
            .encode_utf16()
            .chain([0])
            .collect();
        let mut v = PROPVARIANT::default();
        let mut o = 1;
        if reader
            .GetMetadataByName(PCWSTR(name.as_ptr()), &mut v)
            .is_ok()
            && v.Anonymous.Anonymous.vt == VT_UI2
        {
            let ui = v.Anonymous.Anonymous.Anonymous.uiVal as u32;
            if (1..=8).contains(&ui) {
                o = ui;
            }
        }
        let _ = PropVariantClear(&mut v);
        o
    }

    /// Write `rgb` as a HEIF with WIC's encoder, for tests. The error names
    /// the step that failed (a PC without the HEIF extension has no encoder).
    #[cfg(test)]
    pub fn write_heif(path: &Path, w: u32, h: u32, rgb: &[u8]) -> Result<(), String> {
        let _com = Com(unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok());
        let step =
            |what: &'static str| move |e: windows::core::Error| format!("{what}: {}", e.message());
        unsafe {
            let factory: IWICImagingFactory =
                CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
                    .map_err(step("factory"))?;
            let stream = factory.CreateStream().map_err(step("stream"))?;
            let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
            stream
                .InitializeFromFilename(PCWSTR(wide.as_ptr()), 0x4000_0000)
                .map_err(step("open for writing"))?;
            let encoder = factory
                .CreateEncoder(&GUID_ContainerFormatHeif, std::ptr::null())
                .map_err(step("HEIF encoder"))?;
            encoder
                .Initialize(&stream, WICBitmapEncoderNoCache)
                .map_err(step("encoder init"))?;
            let mut frame = None;
            encoder
                .CreateNewFrame(&mut frame, std::ptr::null_mut())
                .map_err(step("new frame"))?;
            let frame = frame.ok_or("no frame")?;
            frame.Initialize(None).map_err(step("frame init"))?;
            frame.SetSize(w, h).map_err(step("size"))?;
            // WriteSource converts to whatever format the encoder wants.
            let bgr: Vec<u8> = rgb
                .chunks_exact(3)
                .flat_map(|p| [p[2], p[1], p[0]])
                .collect();
            let bitmap = factory
                .CreateBitmapFromMemory(w, h, &GUID_WICPixelFormat24bppBGR, w * 3, &bgr)
                .map_err(step("bitmap"))?;
            frame
                .WriteSource(&bitmap, std::ptr::null())
                .map_err(step("write pixels"))?;
            frame.Commit().map_err(step("frame commit"))?;
            encoder.Commit().map_err(step("encoder commit"))?;
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use std::path::Path;

    use crate::decode::Rgb;

    pub fn decode(path: &Path, long_side: u32) -> Result<Rgb, String> {
        let tmp = super::temp_jpeg();
        let mut cmd = std::process::Command::new("/usr/bin/sips");
        cmd.args(["-s", "format", "jpeg", "-s", "formatOptions", "95"]);
        if long_side > 0 {
            cmd.args(["--resampleHeightWidthMax", &long_side.to_string()]);
        }
        cmd.arg(path).arg("--out").arg(&tmp);
        super::run_to_jpeg(cmd, &tmp, "sips")
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod imp {
    use std::path::Path;

    use crate::decode::Rgb;

    pub fn decode(path: &Path, long_side: u32) -> Result<Rgb, String> {
        let mut last = String::new();
        for tool in ["heif-dec", "heif-convert"] {
            let tmp = super::temp_jpeg();
            let mut cmd = std::process::Command::new(tool);
            cmd.args(["-q", "92"]).arg(path).arg(&tmp);
            match super::run_to_jpeg(cmd, &tmp, tool) {
                Ok(rgb) => return Ok(super::fit(rgb, long_side)),
                Err(e) => last = e,
            }
        }
        Err(format!(
            "{last}. HEIC needs libheif's heif-dec (Debian and Ubuntu: apt install libheif-examples)"
        ))
    }
}

#[cfg(not(any(windows, unix)))]
mod imp {
    pub fn decode(_: &std::path::Path, _: u32) -> Result<crate::decode::Rgb, String> {
        Err("no system HEIC decoder on this platform".into())
    }
}

/// A fresh temporary JPEG path, unique across threads.
#[cfg(unix)]
fn temp_jpeg() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "apature-heic-{}-{}.jpg",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ))
}

/// Run a converter that writes a JPEG to `tmp`, decode it upright, and
/// remove it.
#[cfg(unix)]
fn run_to_jpeg(
    mut cmd: std::process::Command,
    tmp: &std::path::Path,
    tool: &str,
) -> Result<Rgb, String> {
    let out = cmd.output().map_err(|e| format!("{tool}: {e}"));
    let result = out.and_then(|out| {
        if out.status.success() && tmp.is_file() {
            let bytes = std::fs::read(tmp).map_err(|e| e.to_string())?;
            let rgb = crate::decode::decode_jpeg(&bytes, None).map_err(|e| e.to_string())?;
            Ok(crate::decode::rotate(
                rgb,
                crate::jpeg_info::orientation(&bytes),
            ))
        } else {
            Err(format!(
                "{tool}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ))
        }
    });
    let _ = std::fs::remove_file(tmp);
    result
}

/// Shrink to `long_side` when the converter could not.
#[cfg(all(unix, not(target_os = "macos")))]
fn fit(rgb: Rgb, long_side: u32) -> Rgb {
    if long_side == 0 || rgb.long_side() <= long_side {
        rgb
    } else {
        crate::decode::thumbnail(&rgb, long_side)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round trip through Windows' own HEIF encoder and decoder: size,
    /// colour and the downscale. Skipped (passes) on a PC without the HEIF
    /// and HEVC extensions.
    #[cfg(windows)]
    #[test]
    fn windows_decodes_a_heif_it_wrote() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("red.heic");
        let (w, h) = (64u32, 32u32);
        // Left half red, right half blue.
        let rgb: Vec<u8> = (0..w * h)
            .flat_map(|i| {
                if i % w < w / 2 {
                    [220, 20, 20]
                } else {
                    [20, 20, 220]
                }
            })
            .collect();
        if let Err(e) = imp::write_heif(&path, w, h, &rgb) {
            eprintln!("skipped, Windows could not write a HEIF here: {e}");
            return;
        }
        let data = std::fs::read(&path).unwrap();
        assert!(is_heif(&data), "WIC wrote something that is not HEIF");

        let full = decode_with_os(&path, 0).unwrap();
        assert_eq!((full.width, full.height), (w, h));
        let px = |img: &Rgb, x: u32, y: u32| {
            let i = ((y * img.width + x) * 3) as usize;
            [img.data[i], img.data[i + 1], img.data[i + 2]]
        };
        let left = px(&full, 8, 16);
        let right = px(&full, 56, 16);
        assert!(left[0] > 150 && left[2] < 100, "left {left:?}");
        assert!(right[2] > 150 && right[0] < 100, "right {right:?}");

        let small = decode_with_os(&path, 16).unwrap();
        assert_eq!((small.width, small.height), (16, 8));
    }

    #[test]
    fn a_missing_file_is_an_error_not_a_panic() {
        assert!(decode_with_os(Path::new("no/such/photo.heic"), 100).is_err());
    }
}
