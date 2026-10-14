//! Undo for marks: one entry per user action, however many items it touched.

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
}

#[derive(Clone, Debug, Default)]
pub struct UndoStack {
    undo: Vec<Entry>,
    redo: Vec<Entry>,
}

const DEPTH: usize = 200;

impl UndoStack {
    pub fn push(&mut self, entry: Entry) {
        if entry.changes.iter().all(|c| c.before == c.after) {
            return;
        }
        self.undo.push(entry);
        if self.undo.len() > DEPTH {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Pops an entry and returns the (idx, marks) pairs to apply.
    pub fn undo(&mut self) -> Option<(String, Vec<(usize, Marks)>)> {
        let e = self.undo.pop()?;
        let apply = e.changes.iter().map(|c| (c.idx, c.before)).collect();
        let label = e.label.clone();
        self.redo.push(e);
        Some((label, apply))
    }

    pub fn redo(&mut self) -> Option<(String, Vec<(usize, Marks)>)> {
        let e = self.redo.pop()?;
        let apply = e.changes.iter().map(|c| (c.idx, c.after)).collect();
        let label = e.label.clone();
        self.undo.push(e);
        Some((label, apply))
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
        });
        s.push(Entry {
            label: "no-op".into(),
            changes: vec![Change {
                idx: 1,
                before: m(4),
                after: m(4),
            }],
        });
        let (label, apply) = s.undo().unwrap();
        assert_eq!(label, "rate 3 photos");
        assert_eq!(apply, vec![(1, m(0)), (2, m(2))]);
        let (_, apply) = s.redo().unwrap();
        assert_eq!(apply, vec![(1, m(4)), (2, m(4))]);
        s.push(Entry {
            label: "x".into(),
            changes: vec![Change {
                idx: 0,
                before: m(0),
                after: m(1),
            }],
        });
        assert!(s.redo().is_none(), "a new action clears redo");
    }
}
