//! Undo for marks: one entry per user action, however many items it touched.

use apature_core::fileops::Trashed;
use apature_core::Marks;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub idx: usize,
    pub before: Marks,
    pub after: Marks,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub label: String,
    pub changes: Vec<Change>,
    /// Files moved to the Trash by this action; undo puts them back.
    pub trashed: Vec<Trashed>,
}

#[derive(Clone, Debug, Default)]
pub struct UndoStack {
    undo: Vec<Entry>,
    redo: Vec<Entry>,
}

const DEPTH: usize = 200;

impl UndoStack {
    pub fn push(&mut self, entry: Entry) {
        if entry.trashed.is_empty() && entry.changes.iter().all(|c| c.before == c.after) {
            return;
        }
        self.undo.push(entry);
        if self.undo.len() > DEPTH {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Pops an entry and returns the (idx, marks) pairs to apply and the
    /// files to bring back from the Trash.
    pub fn undo(&mut self) -> Option<(String, Vec<(usize, Marks)>, Vec<Trashed>)> {
        let e = self.undo.pop()?;
        let apply = e.changes.iter().map(|c| (c.idx, c.before)).collect();
        let (label, trashed) = (e.label.clone(), e.trashed.clone());
        self.redo.push(e);
        Some((label, apply, trashed))
    }

    /// The files to move to the Trash again come back as `trashed`.
    pub fn redo(&mut self) -> Option<(String, Vec<(usize, Marks)>, Vec<Trashed>)> {
        let e = self.redo.pop()?;
        let apply = e.changes.iter().map(|c| (c.idx, c.after)).collect();
        let (label, trashed) = (e.label.clone(), e.trashed.clone());
        self.undo.push(e);
        Some((label, apply, trashed))
    }

    /// After a redo trashed the files again, record where they went.
    pub fn set_top_trashed(&mut self, trashed: Vec<Trashed>) {
        if let Some(e) = self.undo.last_mut() {
            e.trashed = trashed;
        }
    }

    /// The photo list changed (a delete, a restore): re-point every entry at
    /// the photo's new index, and drop changes to photos no longer listed.
    pub fn remap(&mut self, f: impl Fn(usize) -> Option<usize>) {
        for stack in [&mut self.undo, &mut self.redo] {
            for e in stack.iter_mut() {
                e.changes = e
                    .changes
                    .iter()
                    .filter_map(|c| f(c.idx).map(|idx| Change { idx, ..c.clone() }))
                    .collect();
            }
            stack.retain(|e| !e.changes.is_empty() || !e.trashed.is_empty());
        }
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apature_core::{ColorClass, Rating};

    fn m(r: i8) -> Marks {
        Marks {
            rating: Rating(r),
            color_class: ColorClass(0),
            tagged: false,
        }
    }

    #[test]
    fn batch_is_one_entry_and_round_trips() {
        let mut s = UndoStack::default();
        s.push(Entry {
            label: "rate 3 photos".into(),
            changes: vec![
                Change {
                    idx: 1,
                    before: m(0),
                    after: m(4),
                },
                Change {
                    idx: 2,
                    before: m(2),
                    after: m(4),
                },
            ],
            trashed: vec![],
        });
        s.push(Entry {
            label: "no-op".into(),
            changes: vec![Change {
                idx: 1,
                before: m(4),
                after: m(4),
            }],
            trashed: vec![],
        });
        let (label, apply, _) = s.undo().unwrap();
        assert_eq!(label, "rate 3 photos");
        assert_eq!(apply, vec![(1, m(0)), (2, m(2))]);
        let (_, apply, _) = s.redo().unwrap();
        assert_eq!(apply, vec![(1, m(4)), (2, m(4))]);
        s.push(Entry {
            label: "x".into(),
            changes: vec![Change {
                idx: 0,
                before: m(0),
                after: m(1),
            }],
            trashed: vec![],
        });
        assert!(s.redo().is_none(), "a new action clears redo");
    }

    #[test]
    fn remap_follows_photos_and_drops_the_ones_that_left() {
        let mut s = UndoStack::default();
        let ch = |idx| Change { idx, before: m(0), after: m(3) };
        s.push(Entry { label: "rate".into(), changes: vec![ch(1), ch(4)], trashed: vec![] });
        s.push(Entry { label: "rate one".into(), changes: vec![ch(1)], trashed: vec![] });
        // Photo 1 was deleted: 0 stays, 2.. shift down by one.
        s.remap(|i| match i {
            1 => None,
            i if i > 1 => Some(i - 1),
            i => Some(i),
        });
        let (label, apply, _) = s.undo().unwrap();
        assert_eq!(label, "rate", "the entry that only touched the deleted photo is gone");
        assert_eq!(apply, vec![(3, m(0))]);
        assert!(s.undo().is_none());
    }
}

