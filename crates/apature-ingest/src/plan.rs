//! Build the plan: which files, where they go, what collides.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use apature_core::{Kind, PhotoItem, ScanOptions};

use crate::ledger::{fingerprint, Ledger};
use crate::template::{expand, sanitize_component, Facts};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FileFilter {
    #[default]
    All,
    RawOnly,
    JpegOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub source: PathBuf,
    pub include_subfolders: bool,
    pub primary: PathBuf,
    pub secondary: Option<PathBuf>,
    /// Folder template under each destination, e.g. `{date}-{headline}` or
    /// `{year}/{month}/{day}`. Empty = copy straight into the destination.
    pub folder_template: String,
    /// File name template without extension, e.g. `{headline}-{seqn}`.
    /// Empty = keep original names.
    pub rename_template: String,
    pub seq_start: u32,
    pub filter: FileFilter,
    /// Skip files whose fingerprint is in the ledger.
    pub only_new: bool,
    pub headline: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            source: PathBuf::new(),
            include_subfolders: true,
            primary: PathBuf::new(),
            secondary: None,
            folder_template: "{date}-{headline}".into(),
            rename_template: String::new(),
            seq_start: 1,
            filter: FileFilter::All,
            only_new: true,
            headline: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub source: PathBuf,
    pub size: u64,
    pub primary: PathBuf,
    pub secondary: Option<PathBuf>,
    /// Index of the photo item this file belongs to (pairs share one).
    pub item: usize,
    pub fingerprint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collision {
    pub destination: PathBuf,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skip {
    pub source: PathBuf,
    pub reason: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    pub entries: Vec<Entry>,
    pub skipped: Vec<Skip>,
    pub collisions: Vec<Collision>,
    pub total_bytes: u64,
    pub items: usize,
}

impl Plan {
    pub fn can_run(&self) -> bool {
        self.collisions.is_empty() && !self.entries.is_empty()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlanError {
    #[error("source folder: {0}")]
    Source(#[from] apature_core::folder::ScanError),
    #[error("destination is inside the source, or the source inside the destination")]
    Nested,
    #[error("folder template produced an unusable name for {0}")]
    BadFolder(PathBuf),
    #[error("rename template produced an unusable name for {0}")]
    BadName(PathBuf),
}

/// Naming facts for one file of `item` (capture date/time from EXIF).
pub fn facts_for(item: &PhotoItem, variant_path: &Path, headline: Option<&str>) -> Facts {
    let exif = apature_metadata::exif::read(item.path()).ok();
    let (date, time) = exif
        .and_then(|e| e.date_time_original)
        .map(|d| {
            // "YYYY:MM:DD HH:MM:SS"
            let date = apature_metadata::codes::parse_date(&d);
            let time = (|| {
                let h: u32 = d.get(11..13)?.parse().ok()?;
                let m: u32 = d.get(14..16)?.parse().ok()?;
                let s: u32 = d.get(17..19)?.parse().ok()?;
                Some((h, m, s))
            })();
            (date, time)
        })
        .unwrap_or((None, None));
    Facts {
        stem: variant_path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        ext: variant_path
            .extension()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        date,
        time,
        headline: headline.map(|h| h.to_string()),
    }
}

/// Build the plan. Fast enough to run on every option change for a card
/// (one EXIF read per item, one 64 KiB fingerprint per file when `only_new`).
pub fn build_plan(opts: &Options, ledger: &Ledger) -> Result<Plan, PlanError> {
    if opts.primary.starts_with(&opts.source) || opts.source.starts_with(&opts.primary) {
        return Err(PlanError::Nested);
    }
    if let Some(s) = &opts.secondary {
        if s.starts_with(&opts.source) || opts.source.starts_with(s) {
            return Err(PlanError::Nested);
        }
    }
    let items = apature_core::scan(
        &opts.source,
        &ScanOptions {
            recursive: opts.include_subfolders,
        },
    )?;
    let mut plan = Plan::default();
    let mut seen_dest: BTreeMap<PathBuf, PathBuf> = BTreeMap::new();
    let mut seq = opts.seq_start;
    let mut item_count = 0usize;

    for (i, item) in items.iter().enumerate() {
        let variants: Vec<_> = item
            .variants()
            .filter(|v| match opts.filter {
                FileFilter::All => true,
                FileFilter::RawOnly => v.kind == Kind::Raw,
                FileFilter::JpegOnly => v.kind == Kind::Jpeg,
            })
            .collect();
        if variants.is_empty() {
            continue;
        }
        let facts0 = facts_for(item, &variants[0].path, opts.headline.as_deref());
        // Folder: one per item so a pair never splits.
        let folder_rel: PathBuf = if opts.folder_template.trim().is_empty() {
            PathBuf::new()
        } else {
            let mut rel = PathBuf::new();
            for part in expand(&opts.folder_template, &facts0, seq).split('/') {
                let Some(c) = sanitize_component(part) else {
                    return Err(PlanError::BadFolder(variants[0].path.clone()));
                };
                rel.push(c);
            }
            rel
        };
        let mut added_any = false;
        for v in &variants {
            if opts.only_new {
                match fingerprint(&v.path) {
                    Ok(fp) if ledger.contains(&fp) => {
                        plan.skipped.push(Skip {
                            source: v.path.clone(),
                            reason: "already ingested".into(),
                        });
                        continue;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        plan.skipped.push(Skip {
                            source: v.path.clone(),
                            reason: format!("unreadable: {e}"),
                        });
                        continue;
                    }
                }
            }
            let facts = Facts {
                stem: facts0.stem.clone(),
                ext: v
                    .path
                    .extension()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                ..facts0.clone()
            };
            let name = if opts.rename_template.trim().is_empty() {
                v.file_name()
            } else {
                let stem = sanitize_component(&expand(&opts.rename_template, &facts, seq))
                    .ok_or_else(|| PlanError::BadName(v.path.clone()))?;
                if facts.ext.is_empty() {
                    stem
                } else {
                    format!("{stem}.{}", facts.ext)
                }
            };
            let primary = opts.primary.join(&folder_rel).join(&name);
            let secondary = opts
                .secondary
                .as_ref()
                .map(|s| s.join(&folder_rel).join(&name));
            for dest in std::iter::once(&primary).chain(secondary.iter()) {
                if dest.exists() {
                    plan.collisions.push(Collision {
                        destination: dest.clone(),
                        reason: "already exists at the destination".into(),
                    });
                } else if let Some(other) = seen_dest.get(dest) {
                    plan.collisions.push(Collision {
                        destination: dest.clone(),
                        reason: format!(
                            "two sources map to it: {} and {}",
                            other.display(),
                            v.path.display()
                        ),
                    });
                }
                seen_dest.insert(dest.clone(), v.path.clone());
            }
            // Always record the fingerprint so a later "only new" pass knows this
            // file, even when this run copied everything.
            let fp = fingerprint(&v.path).ok();
            plan.total_bytes += v.size;
            plan.entries.push(Entry {
                source: v.path.clone(),
                size: v.size,
                primary,
                secondary,
                item: i,
                fingerprint: fp,
            });
            added_any = true;
        }
        if added_any {
            item_count += 1;
            seq += 1;
        }
    }
    // Deduplicate identical collision reports.
    let mut seen = BTreeSet::new();
    plan.collisions
        .retain(|c| seen.insert((c.destination.clone(), c.reason.clone())));
    plan.items = item_count;
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn card(dir: &Path) {
        for n in ["DSC_0001.NEF", "DSC_0001.JPG", "DSC_0002.NEF", "notes.txt"] {
            fs::write(dir.join(n), format!("data-{n}")).unwrap();
        }
    }

    #[test]
    fn plans_pairs_rename_and_two_destinations() {
        let t = tempfile::tempdir().unwrap();
        let src = t.path().join("CARD");
        fs::create_dir_all(&src).unwrap();
        card(&src);
        let opts = Options {
            source: src.clone(),
            primary: t.path().join("Work"),
            secondary: Some(t.path().join("Backup")),
            folder_template: "{headline}".into(),
            rename_template: "{headline}-{seqn}".into(),
            headline: Some("100726 SPO BGLF".into()),
            only_new: false,
            ..Default::default()
        };
        let plan = build_plan(&opts, &Ledger::in_memory()).unwrap();
        assert!(plan.can_run());
        assert_eq!(plan.items, 2);
        assert_eq!(plan.entries.len(), 3);
        let names: Vec<String> = plan
            .entries
            .iter()
            .map(|e| {
                e.primary
                    .strip_prefix(t.path())
                    .unwrap()
                    .display()
                    .to_string()
            })
            .collect();
        assert_eq!(
            names,
            vec![
                "Work/100726 SPO BGLF/100726 SPO BGLF-001.NEF",
                "Work/100726 SPO BGLF/100726 SPO BGLF-001.JPG",
                "Work/100726 SPO BGLF/100726 SPO BGLF-002.NEF"
            ]
        );
        assert!(plan.entries[0]
            .secondary
            .as_ref()
            .unwrap()
            .starts_with(t.path().join("Backup")));
        assert_eq!(plan.total_bytes, 3 * "data-DSC_0001.NEF".len() as u64);
    }

    #[test]
    fn refuses_collisions_nesting_and_honours_ledger() {
        let t = tempfile::tempdir().unwrap();
        let src = t.path().join("CARD");
        fs::create_dir_all(&src).unwrap();
        card(&src);
        // Existing file at the destination collides.
        let dest = t.path().join("Work");
        fs::create_dir_all(&dest).unwrap();
        fs::write(dest.join("DSC_0001.NEF"), b"old").unwrap();
        let opts = Options {
            source: src.clone(),
            primary: dest.clone(),
            folder_template: String::new(),
            only_new: false,
            ..Default::default()
        };
        let plan = build_plan(&opts, &Ledger::in_memory()).unwrap();
        assert!(!plan.can_run());
        assert_eq!(plan.collisions.len(), 1);
        // Two sources to one name collide within the plan.
        let opts2 = Options {
            rename_template: "same".into(),
            primary: t.path().join("Other"),
            ..opts.clone()
        };
        let plan2 = build_plan(&opts2, &Ledger::in_memory()).unwrap();
        assert!(plan2
            .collisions
            .iter()
            .any(|c| c.reason.starts_with("two sources")));
        // Nested destination refused.
        let nested = Options {
            primary: src.join("sub"),
            ..opts.clone()
        };
        assert!(matches!(
            build_plan(&nested, &Ledger::in_memory()),
            Err(PlanError::Nested)
        ));
        // Ledger skips already-ingested files.
        let mut ledger = Ledger::in_memory();
        ledger.insert(fingerprint(&src.join("DSC_0002.NEF")).unwrap());
        let opts3 = Options {
            only_new: true,
            primary: t.path().join("Fresh"),
            ..opts.clone()
        };
        let plan3 = build_plan(&opts3, &ledger).unwrap();
        assert_eq!(plan3.entries.len(), 2);
        assert_eq!(plan3.skipped.len(), 1);
        assert_eq!(plan3.skipped[0].reason, "already ingested");
        // RAW-only filter.
        let opts4 = Options {
            filter: FileFilter::RawOnly,
            primary: t.path().join("RawOnly"),
            ..opts.clone()
        };
        assert_eq!(
            build_plan(&opts4, &Ledger::in_memory())
                .unwrap()
                .entries
                .len(),
            2
        );
    }
}
