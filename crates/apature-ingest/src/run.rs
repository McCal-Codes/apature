//! Execute a plan sequentially with progress events and cancellation.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crate::copy::copy_verified;
use crate::ledger::Ledger;
use crate::plan::Plan;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Started {
        index: usize,
        source: PathBuf,
    },
    /// One destination verified.
    Verified {
        index: usize,
        destination: PathBuf,
    },
    Failed {
        index: usize,
        destination: PathBuf,
        error: String,
    },
    Finished,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileOutcome {
    pub source: PathBuf,
    pub verified: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
}

impl FileOutcome {
    pub fn fully_verified(&self, wanted: usize) -> bool {
        self.failed.is_empty() && self.verified.len() == wanted
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub outcomes: Vec<FileOutcome>,
    pub bytes_copied: u64,
    pub elapsed_secs: f64,
    pub cancelled: bool,
}

impl Report {
    pub fn files_ok(&self) -> usize {
        self.outcomes.iter().filter(|o| o.failed.is_empty()).count()
    }
    pub fn files_failed(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| !o.failed.is_empty())
            .count()
    }
    /// True only when every file reached every destination.
    pub fn all_verified(&self, plan: &Plan) -> bool {
        !self.cancelled
            && self.outcomes.len() == plan.entries.len()
            && self.outcomes.iter().all(|o| o.failed.is_empty())
    }
}

/// Copy every entry: primary first (from the source), then the secondary
/// from the fresh primary copy (local disk, faster than the card). Each copy
/// is independently verified. The ledger is updated only for files whose
/// every destination verified, and saved at the end.
pub fn run(
    plan: &Plan,
    ledger: &mut Ledger,
    cancel: &AtomicBool,
    mut on_event: impl FnMut(Event),
) -> Report {
    let t0 = Instant::now();
    let mut report = Report::default();
    for (index, e) in plan.entries.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            report.cancelled = true;
            break;
        }
        on_event(Event::Started {
            index,
            source: e.source.clone(),
        });
        let mut outcome = FileOutcome {
            source: e.source.clone(),
            verified: Vec::new(),
            failed: Vec::new(),
        };
        let wanted = 1 + e.secondary.is_some() as usize;
        match copy_verified(&e.source, &e.primary) {
            Ok(r) => {
                report.bytes_copied += r.bytes;
                outcome.verified.push(e.primary.clone());
                on_event(Event::Verified {
                    index,
                    destination: e.primary.clone(),
                });
                if let Some(sec) = &e.secondary {
                    match copy_verified(&e.primary, sec) {
                        Ok(r2) => {
                            report.bytes_copied += r2.bytes;
                            outcome.verified.push(sec.clone());
                            on_event(Event::Verified {
                                index,
                                destination: sec.clone(),
                            });
                        }
                        Err(err) => {
                            outcome.failed.push((sec.clone(), err.to_string()));
                            on_event(Event::Failed {
                                index,
                                destination: sec.clone(),
                                error: err.to_string(),
                            });
                        }
                    }
                }
            }
            Err(err) => {
                outcome.failed.push((e.primary.clone(), err.to_string()));
                on_event(Event::Failed {
                    index,
                    destination: e.primary.clone(),
                    error: err.to_string(),
                });
            }
        }
        if outcome.fully_verified(wanted) {
            if let Some(fp) = &e.fingerprint {
                ledger.insert(fp.clone());
            }
        }
        report.outcomes.push(outcome);
    }
    let _ = ledger.save();
    report.elapsed_secs = t0.elapsed().as_secs_f64();
    on_event(Event::Finished);
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{build_plan, Options};
    use std::fs;
    use std::path::Path;

    #[test]
    fn copies_to_both_destinations_verifies_and_records() {
        let t = tempfile::tempdir().unwrap();
        let src = t.path().join("CARD");
        fs::create_dir_all(&src).unwrap();
        for n in ["A.NEF", "A.JPG", "B.NEF"] {
            fs::write(src.join(n), n.repeat(1000)).unwrap();
        }
        let lp = t.path().join("ledger.json");
        let mut ledger = Ledger::load(&lp);
        let opts = Options {
            source: src.clone(),
            primary: t.path().join("P"),
            secondary: Some(t.path().join("S")),
            folder_template: "{filename}".into(),
            only_new: true,
            ..Default::default()
        };
        let plan = build_plan(&opts, &ledger).unwrap();
        assert_eq!(plan.entries.len(), 3);
        let mut events = Vec::new();
        let report = run(&plan, &mut ledger, &AtomicBool::new(false), |e| {
            events.push(e)
        });
        assert!(report.all_verified(&plan), "{report:?}");
        assert_eq!(report.files_ok(), 3);
        assert_eq!(report.bytes_copied, 2 * (5000 + 5000 + 5000));
        assert!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Verified { .. }))
                .count()
                == 6
        );
        assert!(matches!(events.last(), Some(Event::Finished)));
        assert!(Path::new(&t.path().join("S/A/A.JPG")).is_file());
        // Sources untouched, ledger persisted, second plan has nothing to do.
        assert_eq!(fs::read_dir(&src).unwrap().count(), 3);
        let ledger2 = Ledger::load(&lp);
        assert_eq!(ledger2.len(), 3);
        let again = build_plan(&opts, &ledger2).unwrap();
        assert!(again.entries.is_empty());
        assert_eq!(again.skipped.len(), 3);
    }

    #[test]
    fn a_failing_destination_is_reported_and_the_rest_continue() {
        let t = tempfile::tempdir().unwrap();
        let src = t.path().join("CARD");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("A.NEF"), b"aaaa").unwrap();
        fs::write(src.join("B.NEF"), b"bbbb").unwrap();
        // Secondary destination is a *file*, so every copy there fails.
        let bad = t.path().join("S");
        fs::write(&bad, b"not a dir").unwrap();
        let opts = Options {
            source: src.clone(),
            primary: t.path().join("P"),
            secondary: Some(bad.clone()),
            folder_template: String::new(),
            only_new: false,
            ..Default::default()
        };
        let plan = build_plan(&opts, &Ledger::in_memory()).unwrap();
        let mut ledger = Ledger::in_memory();
        let report = run(&plan, &mut ledger, &AtomicBool::new(false), |_| {});
        assert_eq!(report.files_failed(), 2);
        assert!(!report.all_verified(&plan));
        assert!(
            t.path().join("P/A.NEF").is_file(),
            "primary still succeeded"
        );
        assert!(
            ledger.is_empty(),
            "nothing recorded as ingested when a destination failed"
        );
        // Cancellation stops before the next file.
        let cancel = AtomicBool::new(true);
        let plan2 = build_plan(
            &Options {
                primary: t.path().join("P2"),
                secondary: None,
                ..opts.clone()
            },
            &Ledger::in_memory(),
        )
        .unwrap();
        let r2 = run(&plan2, &mut Ledger::in_memory(), &cancel, |_| {});
        assert!(r2.cancelled && r2.outcomes.is_empty());
    }
}
