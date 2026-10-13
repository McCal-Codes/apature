//! Metadata in and out.
//!
//! * [`exif`] reads the handful of EXIF fields a contact sheet shows (camera,
//!   capture time, exposure, orientation) straight from the TIFF structure of
//!   a RAW or the APP1 segment of a JPEG, touching only the pages it needs.
//! * [`xmp`] reads and writes `.xmp` sidecars for culling marks, editing the
//!   existing document in place so anything Photo Mechanic or Lightroom wrote
//!   survives untouched.

pub mod exif;
pub mod xmp;

pub use exif::ExifSummary;
pub use xmp::XmpMarks;
