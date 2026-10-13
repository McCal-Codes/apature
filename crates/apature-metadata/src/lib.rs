//! Metadata in and out.
//!
//! * [`exif`] reads the handful of EXIF fields a contact sheet shows (camera,
//!   capture time, exposure, orientation) straight from the TIFF structure of
//!   a RAW or the APP1 segment of a JPEG, touching only the pages it needs.
//! * [`xmp`] reads and writes `.xmp` sidecars for culling marks, editing the
//!   existing document in place so anything Photo Mechanic or Lightroom wrote
//!   survives untouched.
//! * [`jpeg_xmp`] does the same for the packet embedded in a JPEG, by
//!   splicing one APP1 segment and copying every other byte.
//! * [`items`] is the policy layer: RAW → sidecar, JPEG → embedded, pair →
//!   both, with conflict reporting.

pub mod exif;
pub mod items;
pub mod jpeg_xmp;
pub mod xmp;

pub use exif::ExifSummary;
pub use xmp::XmpMarks;
