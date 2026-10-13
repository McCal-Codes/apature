//! A memory-mapped photo file plus its embedded-JPEG index.

use std::fs::File;
use std::path::{Path, PathBuf};

use memmap2::Mmap;

use crate::decode::{decode_jpeg, DecodeError, Rgb};
use crate::embedded::{find_embedded_jpegs, EmbeddedJpeg};

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("{path}: {err}")]
    Io { path: PathBuf, err: std::io::Error },
    #[error("{0}: no embedded JPEG preview found")]
    NoPreview(PathBuf),
    #[error("{0}: {1}")]
    Decode(PathBuf, DecodeError),
}

pub struct Source {
    path: PathBuf,
    map: Mmap,
    candidates: Vec<EmbeddedJpeg>,
}

impl Source {
    /// Memory-map the file and index its embedded JPEGs. Only the pages that
    /// hold IFDs are actually read; a 50 MB NEF costs a few page faults.
    pub fn open(path: &Path) -> Result<Self, SourceError> {
        let file = File::open(path).map_err(|err| SourceError::Io {
            path: path.into(),
            err,
        })?;
        // SAFETY: the file is opened read-only and we never write through the map.
        // If the file is truncated underneath us a SIGBUS is possible, as with any
        // mmap; the contact sheet never modifies originals, so this is accepted.
        let map = unsafe { Mmap::map(&file) }.map_err(|err| SourceError::Io {
            path: path.into(),
            err,
        })?;
        let candidates = find_embedded_jpegs(&map);
        Ok(Source {
            path: path.to_path_buf(),
            map,
            candidates,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn data(&self) -> &[u8] {
        &self.map
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// All embedded JPEGs, largest first.
    pub fn candidates(&self) -> &[EmbeddedJpeg] {
        &self.candidates
    }

    pub fn largest(&self) -> Option<&EmbeddedJpeg> {
        self.candidates.first()
    }

    /// The smallest candidate whose long side is at least `long_side`, or
    /// the largest available if none is big enough.
    pub fn best_for(&self, long_side: u32) -> Option<&EmbeddedJpeg> {
        self.candidates
            .iter()
            .filter(|c| c.long_side() >= long_side)
            .min_by_key(|c| c.long_side())
            .or_else(|| self.largest())
    }

    pub fn bytes(&self, c: &EmbeddedJpeg) -> &[u8] {
        c.bytes(&self.map)
    }

    /// Decode the best candidate for `long_side`, downscaled in the DCT domain.
    pub fn decode_for(&self, long_side: u32) -> Result<Rgb, SourceError> {
        let c = self
            .best_for(long_side)
            .ok_or_else(|| SourceError::NoPreview(self.path.clone()))?;
        decode_jpeg(self.bytes(c), Some(long_side))
            .map_err(|e| SourceError::Decode(self.path.clone(), e))
    }
}
