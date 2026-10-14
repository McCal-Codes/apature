//! Filters and sort orders over the open folder. Pure functions over marks
//! and metadata so they can be tested without a window.

use std::collections::BTreeSet;

use apature_core::{Marks, PhotoItem};
use apature_metadata::ExifSummary;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    /// Minimum star rating, 1..=5. `None` = any.
    pub min_rating: Option<u8>,
    /// Keep only these classes (empty = any). `0` means "no class".
    pub classes: BTreeSet<u8>,
    /// `Some(true)` tagged only, `Some(false)` untagged only.
    pub tagged: Option<bool>,
    pub hide_rejected: bool,
    pub pairs_only: bool,
}

impl Filter {
    pub fn is_active(&self) -> bool {
        *self != Filter::default()
    }

    pub fn matches(&self, item: &PhotoItem, m: &Marks) -> bool {
        if self.hide_rejected && m.rating.is_rejected() {
            return false;
        }
        if let Some(min) = self.min_rating {
            if m.rating.stars() < min {
                return false;
            }
        }
        if !self.classes.is_empty() && !self.classes.contains(&m.color_class.0) {
            return false;
        }
        if let Some(t) = self.tagged {
            if m.tagged != t {
                return false;
            }
        }
        if self.pairs_only && !item.is_pair() {
            return false;
        }
        true
    }

    /// Human chips for the toolbar, one per active clause.
    pub fn chips(&self, scheme: &apature_core::ColorClassScheme) -> Vec<(String, Clause)> {
        let mut out = Vec::new();
        if let Some(r) = self.min_rating {
            out.push((format!("★ ≥ {r}"), Clause::Rating));
        }
        if !self.classes.is_empty() {
            let names: Vec<String> = self
                .classes
                .iter()
                .map(|&c| {
                    if c == 0 {
                        "no class".to_string()
                    } else {
                        scheme
                            .name(apature_core::ColorClass(c))
                            .unwrap_or("?")
                            .to_string()
                    }
                })
                .collect();
            out.push((names.join(", "), Clause::Classes));
        }
        if let Some(t) = self.tagged {
            out.push((
                if t { "tagged" } else { "untagged" }.to_string(),
                Clause::Tagged,
            ));
        }
        if self.hide_rejected {
            out.push(("hide rejected".into(), Clause::Rejected));
        }
        if self.pairs_only {
            out.push(("RAW+JPEG only".into(), Clause::Pairs));
        }
        out
    }

    pub fn clear(&mut self, clause: Clause) {
        match clause {
            Clause::Rating => self.min_rating = None,
            Clause::Classes => self.classes.clear(),
            Clause::Tagged => self.tagged = None,
            Clause::Rejected => self.hide_rejected = false,
            Clause::Pairs => self.pairs_only = false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clause {
    Rating,
    Classes,
    Tagged,
    Rejected,
    Pairs,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Sort {
    #[default]
    Name,
    CaptureTime,
    Rating,
    ColorClass,
}

impl Sort {
    pub const ALL: [Sort; 4] = [
        Sort::Name,
        Sort::CaptureTime,
        Sort::Rating,
        Sort::ColorClass,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Sort::Name => "name",
            Sort::CaptureTime => "capture time",
            Sort::Rating => "rating",
            Sort::ColorClass => "color class",
        }
    }
}

/// The visible, ordered list of item indices.
pub fn build_view(
    items: &[PhotoItem],
    marks: &[Marks],
    exif: &[Option<ExifSummary>],
    filter: &Filter,
    sort: Sort,
    reversed: bool,
) -> Vec<usize> {
    let mut view: Vec<usize> = (0..items.len())
        .filter(|&i| filter.matches(&items[i], &marks[i]))
        .collect();
    let name = |i: usize| items[i].primary.path.to_string_lossy().to_lowercase();
    match sort {
        Sort::Name => {} // scan order is already by name
        Sort::CaptureTime => view.sort_by(|&a, &b| {
            let ka = exif[a].as_ref().and_then(|e| e.date_time_original.clone());
            let kb = exif[b].as_ref().and_then(|e| e.date_time_original.clone());
            ka.cmp(&kb).then_with(|| name(a).cmp(&name(b)))
        }),
        Sort::Rating => view.sort_by(|&a, &b| {
            marks[b]
                .rating
                .0
                .cmp(&marks[a].rating.0)
                .then_with(|| name(a).cmp(&name(b)))
        }),
        Sort::ColorClass => view.sort_by(|&a, &b| {
            let ka = if marks[a].color_class.is_set() {
                marks[a].color_class.0
            } else {
                9
            };
            let kb = if marks[b].color_class.is_set() {
                marks[b].color_class.0
            } else {
                9
            };
            ka.cmp(&kb).then_with(|| name(a).cmp(&name(b)))
        }),
    }
    if reversed {
        view.reverse();
    }
    view
}

#[cfg(test)]
mod tests {
    use super::*;
    use apature_core::{ColorClass, Kind, Rating, Variant};
    use std::path::PathBuf;

    fn item(name: &str, pair: bool) -> PhotoItem {
        let v = |n: &str, k: Kind| Variant {
            path: PathBuf::from(n),
            kind: k,
            size: 1,
            modified: None,
        };
        PhotoItem {
            id: 0,
            stem: name.into(),
            primary: v(&format!("{name}.NEF"), Kind::Raw),
            secondary: if pair {
                vec![v(&format!("{name}.JPG"), Kind::Jpeg)]
            } else {
                vec![]
            },
        }
    }

    fn marks(r: i8, c: u8, t: bool) -> Marks {
        Marks {
            rating: Rating(r),
            color_class: ColorClass(c),
            tagged: t,
        }
    }

    #[test]
    fn filter_clauses_combine_with_and() {
        let it = item("a", true);
        let f = Filter {
            min_rating: Some(3),
            classes: [1, 2].into_iter().collect(),
            tagged: Some(true),
            hide_rejected: true,
            pairs_only: true,
        };
        assert!(f.matches(&it, &marks(4, 1, true)));
        assert!(!f.matches(&it, &marks(2, 1, true)), "rating too low");
        assert!(!f.matches(&it, &marks(4, 3, true)), "class not selected");
        assert!(!f.matches(&it, &marks(4, 1, false)), "untagged");
        assert!(
            !f.matches(&item("b", false), &marks(4, 1, true)),
            "not a pair"
        );
        assert!(!Filter {
            hide_rejected: true,
            ..Default::default()
        }
        .matches(&it, &marks(-1, 0, false)));
        assert!(Filter::default().matches(&it, &marks(-1, 0, false)));
        assert!(!Filter::default().is_active());
        assert_eq!(f.chips(&apature_core::ColorClassScheme::default()).len(), 5);
    }

    #[test]
    fn view_sorts_and_filters() {
        let items = vec![item("c", false), item("a", true), item("b", false)];
        let marks = vec![marks(1, 2, false), marks(5, 0, true), marks(3, 1, false)];
        let e2 = ExifSummary {
            date_time_original: Some("2026:10:06 14:00:00".into()),
            ..Default::default()
        };
        let e0 = ExifSummary {
            date_time_original: Some("2026:10:06 13:00:00".into()),
            ..Default::default()
        };
        let exif = vec![Some(e0), None, Some(e2)];
        assert_eq!(
            build_view(&items, &marks, &exif, &Filter::default(), Sort::Name, false),
            vec![0, 1, 2]
        );
        assert_eq!(
            build_view(
                &items,
                &marks,
                &exif,
                &Filter::default(),
                Sort::Rating,
                false
            ),
            vec![1, 2, 0]
        );
        assert_eq!(
            build_view(
                &items,
                &marks,
                &exif,
                &Filter::default(),
                Sort::ColorClass,
                false
            ),
            vec![2, 0, 1]
        );
        // Unknown capture time sorts first (None < Some), then by time.
        assert_eq!(
            build_view(
                &items,
                &marks,
                &exif,
                &Filter::default(),
                Sort::CaptureTime,
                false
            ),
            vec![1, 0, 2]
        );
        assert_eq!(
            build_view(&items, &marks, &exif, &Filter::default(), Sort::Name, true),
            vec![2, 1, 0]
        );
        let f = Filter {
            min_rating: Some(3),
            ..Default::default()
        };
        assert_eq!(
            build_view(&items, &marks, &exif, &f, Sort::Name, false),
            vec![1, 2]
        );
    }
}
