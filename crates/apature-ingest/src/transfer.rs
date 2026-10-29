//! Copy or move photos between folders and drives, the way ingest copies from
//! a card: every file is checked by SHA-256 before it counts, nothing is
//! overwritten, and a *move* only sends the originals to the Trash after every
//! file of that photo (RAW, JPEG, sidecar) is verified at the destination.
//!
//! A cloud drive can list a file it has not downloaded (an online-only
//! placeholder); reading it would block. Such a file is taken from an
//! alternate source instead (for example the camera card) when exactly one
//! file there has the same size, and reported as failed otherwise.

use std::fs;
use std::path::{Path, PathBuf};

use apature_core::fileops::{self, is_online_only};
use apature_core::PhotoItem;

use crate::copy::{copy_verified, sha256_file};

/// What to do with the originals once the copies are verified.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Leave the originals where they are.
    Copy,
    /// Send the originals of a photo to the Trash after all its files verify.
    Move,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileResult {
    /// Copied and verified.
    Copied { bytes: u64, from_alternate: bool },
    /// Already at the destination with identical content.
    AlreadyThere,
    /// Not copied; the photo's originals stay where they are.
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhotoResult {
    pub name: String,
    pub files: Vec<(PathBuf, FileResult)>,
    /// Originals were sent to the Trash (Move only, and only when every file verified).
    pub trashed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub photos: Vec<PhotoResult>,
}

impl Report {
    pub fn failed(&self) -> usize {
        self.photos
            .iter()
            .filter(|p| {
                p.files
                    .iter()
                    .any(|(_, r)| matches!(r, FileResult::Failed(_)))
            })
            .count()
    }
    pub fn files_copied(&self) -> usize {
        self.photos
            .iter()
            .flat_map(|p| &p.files)
            .filter(|(_, r)| matches!(r, FileResult::Copied { .. }))
            .count()
    }
    pub fn trashed(&self) -> usize {
        self.photos.iter().filter(|p| p.trashed).count()
    }
}

/// Every file under `roots`, for matching an online-only file by size.
pub fn index_alternates(roots: &[PathBuf]) -> Vec<(u64, PathBuf)> {
    fn walk(dir: &Path, out: &mut Vec<(u64, PathBuf)>) {
        let Ok(rd) = fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            match e.metadata() {
                Ok(m) if m.is_dir() => walk(&p, out),
                Ok(m) if m.is_file() => out.push((m.len(), p)),
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    for r in roots {
        walk(r, &mut out);
    }
    out
}

/// Transfer `items` into `dest_dir` (created if needed). `alternates` is the
/// size index of other places the same files might be read from.
/// `trash` is how originals are removed in `Mode::Move` (the system Trash in
/// the app; a fake in tests). Sequential: a card or cloud drive is the
/// bottleneck.
pub fn transfer(
    items: &[PhotoItem],
    dest_dir: &Path,
    mode: Mode,
    alternates: &[(u64, PathBuf)],
    trash: &dyn Fn(&[PathBuf]) -> Result<(), String>,
    on_file: &mut dyn FnMut(&Path, &FileResult),
) -> Report {
    let mut report = Report::default();
    for item in items {
        let mut files: Vec<(PathBuf, FileResult)> = Vec::new();
        let mut verified_sources: Vec<PathBuf> = Vec::new();
        for src in fileops::item_files(item) {
            let r = one(&src, dest_dir, alternates);
            on_file(&src, &r);
            if matches!(r, FileResult::Copied { .. } | FileResult::AlreadyThere) {
                verified_sources.push(src.clone());
            }
            files.push((src, r));
        }
        let all_ok = files
            .iter()
            .all(|(_, r)| !matches!(r, FileResult::Failed(_)));
        let mut trashed = false;
        if mode == Mode::Move && all_ok && !verified_sources.is_empty() {
            // Online-only originals have nothing local to trash; the cloud
            // drive's own copy goes with the others when it syncs.
            trashed = trash(&verified_sources).is_ok();
        }
        report.photos.push(PhotoResult {
            name: item.display_name(),
            files,
            trashed,
        });
    }
    report
}

fn one(src: &Path, dest_dir: &Path, alternates: &[(u64, PathBuf)]) -> FileResult {
    let Some(name) = src.file_name() else {
        return FileResult::Failed("no file name".into());
    };
    let dst = dest_dir.join(name);
    let (read_from, from_alternate) = if is_online_only(src) {
        let Ok(meta) = fs::symlink_metadata(src) else {
            return FileResult::Failed("online-only and unreadable".into());
        };
        let same: Vec<&PathBuf> = alternates
            .iter()
            .filter(|(len, _)| *len == meta.len())
            .map(|(_, p)| p)
            .collect();
        match same.as_slice() {
            [one] => ((*one).clone(), true),
            [] => {
                return FileResult::Failed(
                    "online-only (not downloaded) and no file of the same size on the card".into(),
                )
            }
            _ => {
                return FileResult::Failed(
                    "online-only (not downloaded) and several card files have the same size".into(),
                )
            }
        }
    } else {
        (src.to_path_buf(), false)
    };
    if dst.exists() {
        return match (sha256_file(&read_from), sha256_file(&dst)) {
            (Ok(a), Ok(b)) if a == b => FileResult::AlreadyThere,
            (Ok(_), Ok(_)) => FileResult::Failed(format!(
                "{} already exists with different content; nothing was overwritten",
                dst.display()
            )),
            (Err(e), _) | (_, Err(e)) => FileResult::Failed(e.to_string()),
        };
    }
    match copy_verified(&read_from, &dst) {
        Ok(r) => FileResult::Copied {
            bytes: r.bytes,
            from_alternate,
        },
        Err(e) => FileResult::Failed(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apature_core::{item, scan, ScanOptions};
    use std::cell::RefCell;

    fn photo(dir: &Path, stem: &str, raw: &str, xmp: Option<&str>) {
        fs::write(dir.join(format!("{stem}.NEF")), raw).unwrap();
        if let Some(x) = xmp {
            fs::write(dir.join(format!("{stem}.XMP")), x).unwrap();
        }
    }

    fn items_of(dir: &Path) -> Vec<PhotoItem> {
        scan(dir, &ScanOptions::default()).unwrap()
    }

    #[test]
    fn copy_brings_raw_and_sidecar_and_leaves_the_originals() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        photo(src.path(), "A001", "raw bytes", Some("xmp"));
        let trashed = RefCell::new(Vec::new());
        let r = transfer(
            &items_of(src.path()),
            dst.path(),
            Mode::Copy,
            &[],
            &|f| {
                trashed.borrow_mut().extend_from_slice(f);
                Ok(())
            },
            &mut |_, _| {},
        );
        assert_eq!(r.files_copied(), 2);
        assert_eq!(
            fs::read_to_string(dst.path().join("A001.NEF")).unwrap(),
            "raw bytes"
        );
        assert_eq!(
            fs::read_to_string(dst.path().join("A001.XMP")).unwrap(),
            "xmp"
        );
        assert!(src.path().join("A001.NEF").exists());
        assert!(trashed.borrow().is_empty(), "copy never trashes");
    }

    #[test]
    fn move_trashes_only_after_every_file_verified_and_is_repeatable() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        photo(src.path(), "A001", "raw bytes", Some("xmp"));
        let items = items_of(src.path());
        let trashed = RefCell::new(Vec::<PathBuf>::new());
        let fake = |f: &[PathBuf]| -> Result<(), String> {
            trashed.borrow_mut().extend_from_slice(f);
            Ok(())
        };
        let r = transfer(&items, dst.path(), Mode::Move, &[], &fake, &mut |_, _| {});
        assert_eq!(r.trashed(), 1);
        assert_eq!(trashed.borrow().len(), 2);
        // Run again (the fake did not really remove them): already there, no new copies.
        let r2 = transfer(&items, dst.path(), Mode::Copy, &[], &fake, &mut |_, _| {});
        assert_eq!(r2.files_copied(), 0);
        assert_eq!(r2.failed(), 0);
    }

    #[test]
    fn a_different_file_at_the_destination_is_never_overwritten_and_blocks_the_trash() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        photo(src.path(), "A001", "new", Some("xmp"));
        fs::write(dst.path().join("A001.NEF"), "someone else's file").unwrap();
        let trashed = RefCell::new(Vec::<PathBuf>::new());
        let r = transfer(
            &items_of(src.path()),
            dst.path(),
            Mode::Move,
            &[],
            &|f| {
                trashed.borrow_mut().extend_from_slice(f);
                Ok(())
            },
            &mut |_, _| {},
        );
        assert_eq!(r.failed(), 1);
        assert_eq!(r.trashed(), 0);
        assert!(trashed.borrow().is_empty());
        assert_eq!(
            fs::read_to_string(dst.path().join("A001.NEF")).unwrap(),
            "someone else's file"
        );
    }

    #[test]
    fn alternates_are_matched_by_size_and_must_be_unique() {
        let card = tempfile::tempdir().unwrap();
        fs::write(card.path().join("CAL_0001.NEF"), "12345").unwrap();
        fs::write(card.path().join("CAL_0002.NEF"), "abcdefgh").unwrap();
        let idx = index_alternates(&[card.path().to_path_buf()]);
        assert_eq!(idx.iter().filter(|(l, _)| *l == 5).count(), 1);
        assert_eq!(idx.len(), 2);
        let _ = item::Kind::Raw; // the item module is part of the public API used here
    }
}
