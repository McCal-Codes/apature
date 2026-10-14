//! Selection, kept apart from focus (the cursor) on purpose: the keyboard
//! moves focus, and selection follows it unless ⇧ extends a range or ⌘
//! keeps the existing set. See docs/design/INTERACTION_SPEC.md.

use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    set: BTreeSet<usize>,
    /// Range anchor: where a ⇧-range starts. Set by plain clicks/moves.
    anchor: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub shift: bool,
    pub command: bool,
}

impl Selection {
    pub fn contains(&self, idx: usize) -> bool {
        self.set.contains(&idx)
    }
    pub fn len(&self) -> usize {
        self.set.len()
    }
    pub fn clear(&mut self) {
        self.set.clear();
        self.anchor = None;
    }

    /// Select exactly `idx`.
    pub fn only(&mut self, idx: usize) {
        self.set.clear();
        self.set.insert(idx);
        self.anchor = Some(idx);
    }

    /// Replace the set with every index in `view`.
    pub fn all(&mut self, view: &[usize]) {
        self.set = view.iter().copied().collect();
    }

    /// A click on `idx` with modifiers, where `view` is the visible order.
    pub fn click(&mut self, idx: usize, mods: Mods, view: &[usize]) {
        if mods.shift {
            self.range_to(idx, view);
        } else if mods.command {
            if !self.set.remove(&idx) {
                self.set.insert(idx);
            }
            self.anchor = Some(idx);
        } else {
            self.only(idx);
        }
    }

    /// Focus moved to `idx` by keyboard: plain moves collapse the selection
    /// to the new focus, ⇧ extends the range from the anchor, ⌘ leaves the
    /// selection alone.
    pub fn focus_moved(&mut self, idx: usize, mods: Mods, view: &[usize]) {
        if mods.command {
            return;
        }
        if mods.shift {
            self.range_to(idx, view);
        } else {
            self.only(idx);
        }
    }

    fn range_to(&mut self, idx: usize, view: &[usize]) {
        let anchor = self.anchor.unwrap_or(idx);
        let pa = view.iter().position(|&i| i == anchor);
        let pb = view.iter().position(|&i| i == idx);
        match (pa, pb) {
            (Some(a), Some(b)) => {
                let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                self.set.clear();
                self.set.extend(view[lo..=hi].iter().copied());
                self.anchor = Some(anchor);
            }
            _ => self.only(idx),
        }
    }

    /// Which items a mark applies to: the selection when the focused item is
    /// part of it, otherwise just the focused item.
    pub fn targets(&self, cursor: usize) -> Vec<usize> {
        if self.set.contains(&cursor) && self.set.len() > 1 {
            self.set.iter().copied().collect()
        } else {
            vec![cursor]
        }
    }

    /// Drop indices that are no longer visible.
    pub fn retain_visible(&mut self, visible: &[bool]) {
        self.set
            .retain(|&i| visible.get(i).copied().unwrap_or(false));
        if let Some(a) = self.anchor {
            if !visible.get(a).copied().unwrap_or(false) {
                self.anchor = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW: [usize; 6] = [0, 1, 2, 3, 4, 5];

    #[test]
    fn plain_click_and_move_collapse_to_one() {
        let mut s = Selection::default();
        s.click(2, Mods::default(), &VIEW);
        assert_eq!(s.set.iter().copied().collect::<Vec<_>>(), vec![2]);
        s.focus_moved(3, Mods::default(), &VIEW);
        assert_eq!(s.set.iter().copied().collect::<Vec<_>>(), vec![3]);
        assert_eq!(s.targets(3), vec![3]);
    }

    #[test]
    fn shift_extends_from_anchor_in_view_order() {
        let mut s = Selection::default();
        s.click(1, Mods::default(), &VIEW);
        s.focus_moved(
            4,
            Mods {
                shift: true,
                command: false,
            },
            &VIEW,
        );
        assert_eq!(s.set.iter().copied().collect::<Vec<_>>(), vec![1, 2, 3, 4]);
        // Shrinking the range keeps the anchor.
        s.focus_moved(
            2,
            Mods {
                shift: true,
                command: false,
            },
            &VIEW,
        );
        assert_eq!(s.set.iter().copied().collect::<Vec<_>>(), vec![1, 2]);
        // Range runs over a filtered view, not raw indices.
        let view = [5, 3, 1];
        s.click(5, Mods::default(), &view);
        s.click(
            1,
            Mods {
                shift: true,
                command: false,
            },
            &view,
        );
        assert_eq!(s.set.iter().copied().collect::<Vec<_>>(), vec![1, 3, 5]);
    }

    #[test]
    fn command_toggles_and_keeps_on_move() {
        let mut s = Selection::default();
        s.click(0, Mods::default(), &VIEW);
        s.click(
            2,
            Mods {
                shift: false,
                command: true,
            },
            &VIEW,
        );
        assert_eq!(s.set.iter().copied().collect::<Vec<_>>(), vec![0, 2]);
        s.focus_moved(
            4,
            Mods {
                shift: false,
                command: true,
            },
            &VIEW,
        );
        assert_eq!(s.set.iter().copied().collect::<Vec<_>>(), vec![0, 2]);
        // Focus outside the selection: marks apply to focus only.
        assert_eq!(s.targets(4), vec![4]);
        assert_eq!(s.targets(2), vec![0, 2]);
        s.click(
            2,
            Mods {
                shift: false,
                command: true,
            },
            &VIEW,
        );
        assert_eq!(s.set.iter().copied().collect::<Vec<_>>(), vec![0]);
    }

    #[test]
    fn retain_visible_drops_hidden() {
        let mut s = Selection::default();
        s.all(&VIEW);
        s.retain_visible(&[true, false, true, false, true, false]);
        assert_eq!(s.set.iter().copied().collect::<Vec<_>>(), vec![0, 2, 4]);
    }
}
