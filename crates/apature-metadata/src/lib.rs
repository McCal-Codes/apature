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
//! * [`iptc`] reads and writes the editorial IPTC Core fields with the same
//!   in-place discipline, including `rdf:Alt`/`Seq`/`Bag` structures.
//! * [`codes`] expands Photo Mechanic-style code replacements with date and
//!   credit variables; [`stationery`] turns an assignment config into an
//!   IPTC pad, a caption template and a code table.

pub mod codes;
pub mod exif;
pub mod iptc;
pub mod items;
pub mod jpeg_xmp;
pub mod stationery;
pub mod xmp;

pub use exif::ExifSummary;
pub use iptc::Iptc;
pub use xmp::XmpMarks;
