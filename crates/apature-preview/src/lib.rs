//! Fast previews without decoding RAW data.
//!
//! The architectural rule of the whole project lives here: **never demosaic a
//! RAW to show a thumbnail when the camera already embedded a JPEG.** Every
//! RAW format we care about (NEF, CR2, ARW, DNG, ...) is a TIFF container with
//! one or more full-size or reduced JPEG previews inside; we locate them by
//! walking IFDs, decode the smallest one that satisfies the requested size
//! with DCT-domain downscaling, and cache the result on disk. Canon CR3 is
//! ISO-BMFF rather than TIFF and is read through `apature_core::bmff`. HEIC
//! has no JPEG to find: its HEVC picture goes to the operating system's own
//! decoder (`system`).

pub mod cache;
pub mod decode;
pub mod embedded;
pub mod jpeg_info;
pub mod source;
pub mod system;

pub use cache::ThumbCache;
pub use decode::{decode_jpeg, rotate, thumbnail, DecodeError, Rgb};
pub use embedded::{find_embedded_jpegs, EmbeddedJpeg};
pub use source::{Source, SourceError};
