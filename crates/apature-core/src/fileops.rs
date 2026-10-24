//! Renaming photos in place and moving them to the Trash. Both act on a
//! whole [`PhotoItem`]: every variant (RAW, JPEG) and the XMP sidecar move
//! together, so a pair never splits. Rules, like ingest: plan first, refuse
//! any collision, never overwrite, never delete (the Trash is the system's
//! and Finder › Put Back restores).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::PhotoItem;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenamePlan {
    /// (from, to) for every file that changes name.
    pub moves: Vec<(PathBuf, PathBuf)>,
    /// Targets that cannot be used, with the reason.
    pub collisions: Vec<(PathBuf, String)>,
    /// Items whose name does not change (template gave the same stem).
    pub unchanged: usize,
}

impl RenamePlan {
    pub fn can_run(&self) -> bool {
        self.collisions.is_empty() && !self.moves.is_empty()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FileOpError {
    #[error("the plan has collisions; nothing was renamed")]
    Collisions,
    #[error("new name {0:?} is not a plain file name")]
    BadName(String),
    #[error("{0}: {1}")]
    Io(PathBuf, std::io::Error),
    #[error("could not move to the Trash: {0}")]
    Trash(String),
}

/// The files of `item` that move with it: every variant plus the sidecar
/// when one exists.
pub fn item_files(item: &PhotoItem) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = item.variants().map(|x| x.path.clone()).collect();
    let sc = item.sidecar_path();
    if sc.is_file() {
        v.push(sc);
    }
    v
}

fn with_stem(path: &Path, stem: &str) -> PathBuf {
    match path.extension() {
        Some(ext) => path.with_file_name(format!("{stem}.{}", ext.to_string_lossy())),
        None => path.with_file_name(stem),
    }
}

/// Plan renaming each item to its new stem (no extension, no separators).
pub fn plan_rename(items: &[(&PhotoItem, String)]) -> Result<RenamePlan, FileOpError> {
    let mut plan = RenamePlan::default();
    let mut sources: HashSet<PathBuf> = HashSet::new();
    for (it, _) in items {
        for f in item_files(it) {
            sources.insert(f);
        }
    }
    let mut targets: HashSet<PathBuf> = HashSet::new();
    for (it, stem) in items {
        let stem = stem.trim();
        if stem.is_empty()
            || stem.contains('/')
            || stem.contains('\\')
            || stem.contains(':')
            || stem == "."
            || stem == ".."
        {
            return Err(FileOpError::BadName(stem.to_string()));
        }
        if stem == it.stem {
            plan.unchanged += 1;
            continue;
        }
        for from in item_files(it) {
            let to = with_stem(&from, stem);
            if to == from {
                continue;
            }
            if !targets.insert(to.clone()) {
                plan.collisions
                    .push((to.clone(), "two photos would get this name".into()));
                continue;
            }
            // Case-only renames on a case-insensitive disk report the target
            // as existing; the two-phase rename below handles those, so only
            // a target that is not also a source counts as a collision.
            if to.exists() && !sources.contains(&to) {
                plan.collisions
                    .push((to.clone(), "a file with this name already exists".into()));
                continue;
            }
            plan.moves.push((from, to));
        }
    }
    Ok(plan)
}

/// Execute a plan. Two phases (every source to a temporary name, then every
/// temporary to its target) so swaps and case changes work; any failure rolls
/// back what was done. Returns the completed moves.
pub fn rename(plan: &RenamePlan) -> Result<Vec<(PathBuf, PathBuf)>, FileOpError> {
    if !plan.collisions.is_empty() {
        return Err(FileOpError::Collisions);
    }
    let tmp_of = |p: &Path, i: usize| -> PathBuf {
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        p.with_file_name(format!(".{name}.apature-rename-{i}"))
    };
    let mut staged: Vec<(PathBuf, PathBuf)> = Vec::new(); // (from, tmp)
    for (i, (from, _)) in plan.moves.iter().enumerate() {
        let tmp = tmp_of(from, i);
        if let Err(e) = std::fs::rename(from, &tmp) {
            for (f, t) in staged.iter().rev() {
                let _ = std::fs::rename(t, f);
            }
            return Err(FileOpError::Io(from.clone(), e));
        }
        staged.push((from.clone(), tmp));
    }
    let mut done: Vec<(PathBuf, PathBuf)> = Vec::new(); // (tmp, to)
    for ((_, tmp), (_, to)) in staged.iter().zip(plan.moves.iter()) {
        if to.exists() {
            // Appeared since planning: roll everything back.
            for (t, d) in done.iter().rev() {
                let _ = std::fs::rename(d, t);
            }
            for (f, t) in staged.iter().rev() {
                let _ = std::fs::rename(t, f);
            }
            return Err(FileOpError::Io(
                to.clone(),
                std::io::Error::new(std::io::ErrorKind::AlreadyExists, "appeared during rename"),
            ));
        }
        if let Err(e) = std::fs::rename(tmp, to) {
            for (t, d) in done.iter().rev() {
                let _ = std::fs::rename(d, t);
            }
            for (f, t) in staged.iter().rev() {
                let _ = std::fs::rename(t, f);
            }
            return Err(FileOpError::Io(to.clone(), e));
        }
        done.push((tmp.clone(), to.clone()));
    }
    Ok(plan.moves.clone())
}

/// Move files to the system Trash. On macOS this goes through
/// NSFileManager rather than Finder scripting: no automation prompt, no
/// Finder time-outs; the files sit in the Trash until it is emptied.
pub fn trash(paths: &[PathBuf]) -> Result<(), FileOpError> {
    if paths.is_empty() {
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        use trash::macos::{DeleteMethod, TrashContextExtMacos};
        let mut ctx = trash::TrashContext::default();
        ctx.set_delete_method(DeleteMethod::NsFileManager);
        ctx.delete_all(paths)
            .map_err(|e| FileOpError::Trash(e.to_string()))
    }
    #[cfg(not(target_os = "macos"))]
    {
        trash::delete_all(paths).map_err(|e| FileOpError::Trash(e.to_string()))
    }
}

/// A file the Trash now holds, with where it came from, so it can go back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trashed {
    pub original: PathBuf,
    pub in_trash: PathBuf,
}

/// The user's Trash folder, where the platform keeps it as plain files.
#[cfg(not(target_os = "macos"))]
fn trash_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    #[cfg(unix)]
    {
        Some(home.join(".local/share/Trash/files"))
    }
    #[cfg(not(unix))]
    {
        let _ = home;
        None
    }
}

/// [`trash`], and find where each file landed so [`restore`] can put it back.
/// On macOS the system says where each item went (`trashItemAtURL` returns the
/// resulting URL; the Trash folder itself is not readable without Full Disk
/// Access). Elsewhere the Trash folder is searched for the new entries; where
/// that is not possible the list is empty and the files stay in the Trash.
#[cfg(target_os = "macos")]
pub fn trash_tracked(paths: &[PathBuf]) -> Result<Vec<Trashed>, FileOpError> {
    use objc2_foundation::{NSFileManager, NSString, NSURL};
    let fm = NSFileManager::defaultManager();
    let mut out = Vec::new();
    for p in paths {
        let Some(s) = p.to_str() else {
            trash(std::slice::from_ref(p))?;
            continue;
        };
        let url = NSURL::fileURLWithPath(&NSString::from_str(s));
        let mut landed = None;
        fm.trashItemAtURL_resultingItemURL_error(&url, Some(&mut landed))
            .map_err(|e| FileOpError::Trash(e.to_string()))?;
        if let Some(in_trash) = landed.and_then(|u| u.path()).map(|n| PathBuf::from(n.to_string())) {
            out.push(Trashed {
                original: p.clone(),
                in_trash,
            });
        }
    }
    Ok(out)
}

#[cfg(not(target_os = "macos"))]
pub fn trash_tracked(paths: &[PathBuf]) -> Result<Vec<Trashed>, FileOpError> {
    let dir = trash_dir();
    let names = |d: &Path| -> Vec<std::ffi::OsString> {
        std::fs::read_dir(d)
            .map(|r| r.filter_map(|e| e.ok().map(|e| e.file_name())).collect())
            .unwrap_or_default()
    };
    let before: HashSet<std::ffi::OsString> = dir.as_deref().map(names).unwrap_or_default().into_iter().collect();
    trash(paths)?;
    let Some(dir) = dir else {
        return Ok(Vec::new());
    };
    let mut fresh: Vec<std::ffi::OsString> = names(&dir).into_iter().filter(|n| !before.contains(n)).collect();
    let mut out = Vec::new();
    for p in paths {
        let Some(name) = p.file_name() else { continue };
        let stem = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let ext = p.extension().map(|e| e.to_os_string());
        let pick = fresh.iter().position(|n| n == name).or_else(|| {
            fresh.iter().position(|n| {
                let np = Path::new(n);
                let s = np.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                np.extension().map(|e| e.to_os_string()) == ext && s.starts_with(&format!("{stem} "))
            })
        });
        if let Some(i) = pick {
            out.push(Trashed {
                original: p.clone(),
                in_trash: dir.join(fresh.swap_remove(i)),
            });
        }
    }
    Ok(out)
}

/// Put trashed files back where they were. Refuses to overwrite: a file that
/// already sits at the original path is left alone and counted as skipped.
/// Returns (restored, skipped).
pub fn restore(items: &[Trashed]) -> Result<(usize, usize), FileOpError> {
    let (mut done, mut skipped) = (0, 0);
    for t in items {
        if t.original.exists() || !t.in_trash.exists() {
            skipped += 1;
            continue;
        }
        if let Some(parent) = t.original.parent() {
            std::fs::create_dir_all(parent).map_err(|e| FileOpError::Io(parent.to_path_buf(), e))?;
        }
        if std::fs::rename(&t.in_trash, &t.original).is_err() {
            // Another volume (a synced drive): copy, then drop the Trash copy.
            std::fs::copy(&t.in_trash, &t.original).map_err(|e| FileOpError::Io(t.original.clone(), e))?;
            let _ = std::fs::remove_file(&t.in_trash);
        }
        done += 1;
    }
    Ok((done, skipped))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::{Kind, Variant};

    fn item(dir: &Path, stem: &str, exts: &[&str]) -> PhotoItem {
        let mut vs: Vec<Variant> = exts
            .iter()
            .map(|e| {
                let p = dir.join(format!("{stem}.{e}"));
                std::fs::write(&p, e).unwrap();
                Variant {
                    path: p,
                    kind: if *e == "NEF" { Kind::Raw } else { Kind::Jpeg },
                    size: 1,
                    modified: None,
                }
            })
            .collect();
        let primary = vs.remove(0);
        PhotoItem {
            id: 0,
            stem: stem.into(),
            primary,
            secondary: vs,
        }
    }

    #[test]
    fn pair_and_sidecar_move_together_and_collisions_refuse() {
        let d = tempfile::tempdir().unwrap();
        let a = item(d.path(), "DSC_0001", &["NEF", "JPG"]);
        std::fs::write(d.path().join("DSC_0001.xmp"), "x").unwrap();
        let b = item(d.path(), "DSC_0002", &["NEF"]);
        let plan = plan_rename(&[(&a, "BUTLR-001".into()), (&b, "BUTLR-002".into())]).unwrap();
        assert_eq!(plan.moves.len(), 4, "NEF+JPG+xmp, NEF");
        assert!(plan.can_run());
        let done = rename(&plan).unwrap();
        assert_eq!(done.len(), 4);
        for n in [
            "BUTLR-001.NEF",
            "BUTLR-001.JPG",
            "BUTLR-001.xmp",
            "BUTLR-002.NEF",
        ] {
            assert!(d.path().join(n).is_file(), "{n}");
        }
        assert!(!d.path().join("DSC_0001.NEF").exists());
        assert!(std::fs::read_dir(d.path()).unwrap().all(|e| !e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("apature-rename")));

        // Collision with an existing file, and two items to one name.
        let c = item(d.path(), "DSC_0003", &["NEF"]);
        let e = item(d.path(), "DSC_0004", &["NEF"]);
        let plan = plan_rename(&[(&c, "BUTLR-001".into()), (&e, "BUTLR-001".into())]).unwrap();
        assert_eq!(plan.collisions.len(), 2);
        assert!(!plan.can_run());
        assert!(matches!(rename(&plan), Err(FileOpError::Collisions)));
        assert!(d.path().join("DSC_0003.NEF").is_file(), "nothing moved");
    }

    #[test]
    fn swap_and_unchanged_and_bad_names() {
        let d = tempfile::tempdir().unwrap();
        let a = item(d.path(), "A", &["JPG"]);
        let b = item(d.path(), "B", &["JPG"]);
        let plan = plan_rename(&[(&a, "B".into()), (&b, "A".into())]).unwrap();
        assert!(
            plan.can_run(),
            "swap is not a collision: {:?}",
            plan.collisions
        );
        rename(&plan).unwrap();
        assert_eq!(
            std::fs::read_to_string(d.path().join("A.JPG")).unwrap(),
            "JPG"
        );
        let plan = plan_rename(&[(&a, "A".into())]).unwrap();
        assert_eq!(plan.unchanged, 1);
        assert!(!plan.can_run());
        assert!(matches!(
            plan_rename(&[(&a, "x/y".into())]),
            Err(FileOpError::BadName(_))
        ));
    }

    #[test]
    fn restore_puts_files_back_and_never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let (home, trash) = (dir.path().join("photos"), dir.path().join("trash"));
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&trash).unwrap();
        let (a, b) = (home.join("a.NEF"), home.join("b.NEF"));
        let (ta, tb) = (trash.join("a.NEF"), trash.join("b 2.NEF"));
        std::fs::write(&ta, "a").unwrap();
        std::fs::write(&tb, "b").unwrap();
        std::fs::write(&b, "new b").unwrap();
        let items = vec![
            Trashed { original: a.clone(), in_trash: ta },
            Trashed { original: b.clone(), in_trash: tb.clone() },
        ];
        assert_eq!(restore(&items).unwrap(), (1, 1));
        assert_eq!(std::fs::read_to_string(&a).unwrap(), "a");
        assert_eq!(std::fs::read_to_string(&b).unwrap(), "new b");
        assert!(tb.exists(), "the clash stays in the Trash");
    }

    /// Touches the real Trash, so it only runs on request:
    /// `cargo test -p apature-core -- --ignored trash_round_trip`.
    #[test]
    #[ignore]
    fn trash_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let name = format!("apature-trash-test-{}.NEF", std::process::id());
        let (a, side) = (dir.path().join(&name), dir.path().join(name.replace(".NEF", ".XMP")));
        std::fs::write(&a, "raw").unwrap();
        std::fs::write(&side, "xmp").unwrap();
        let t = trash_tracked(&[a.clone(), side.clone()]).unwrap();
        assert_eq!(t.len(), 2, "both files located in the Trash: {t:?}");
        assert!(!a.exists() && !side.exists());
        assert_eq!(restore(&t).unwrap(), (2, 0));
        assert_eq!(std::fs::read_to_string(&a).unwrap(), "raw");
        assert_eq!(std::fs::read_to_string(&side).unwrap(), "xmp");
    }
}

