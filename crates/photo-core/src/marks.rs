//! Culling marks: star rating, Photo Mechanic-style colour class, and tag.
//!
//! These map onto the XMP properties Photo Mechanic and Lightroom both read:
//! `xmp:Rating`, `xmp:Label`, `photomechanic:ColorClass`, `photomechanic:Tagged`.

use serde::{Deserialize, Serialize};

/// Star rating. `-1` is "rejected" (Lightroom's X), `0` is unrated, `1..=5` stars.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rating(pub i8);

impl Rating {
    pub fn clamp(v: i8) -> Self {
        Rating(v.clamp(-1, 5))
    }
    pub fn stars(self) -> u8 {
        self.0.max(0) as u8
    }
    pub fn is_rejected(self) -> bool {
        self.0 < 0
    }
}

/// Photo Mechanic colour class, `0` = none, `1..=8`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColorClass(pub u8);

impl ColorClass {
    pub const NONE: ColorClass = ColorClass(0);
    pub fn clamp(v: u8) -> Self {
        ColorClass(v.min(8))
    }
    pub fn is_set(self) -> bool {
        self.0 != 0
    }
    pub fn index(self) -> Option<usize> {
        if self.is_set() {
            Some((self.0 - 1) as usize)
        } else {
            None
        }
    }
}

/// Names and swatch colours for the eight colour classes.
///
/// Photo Mechanic writes the class *name* into `xmp:Label` so other apps can
/// show something sensible; the names here must match what your copy of Photo
/// Mechanic is configured with for round-tripping to be exact. The defaults
/// are the names Photo Mechanic 2026.6 shows in Image › Set Color Class
/// (verified 2026-10-13: it wrote `xmp:Label="Yellow"` for class 2). The
/// older "Winner / Winner alt / Superior…" names are accepted as aliases
/// when reading.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorClassScheme {
    pub names: [String; 8],
    pub colors: [[u8; 3]; 8],
}

impl Default for ColorClassScheme {
    fn default() -> Self {
        ColorClassScheme {
            names: [
                "Red".into(),
                "Yellow".into(),
                "Green".into(),
                "Blue".into(),
                "Purple".into(),
                "Cyan".into(),
                "Brown".into(),
                "Trash".into(),
            ],
            colors: [
                [220, 50, 47],  // 1 red
                [233, 185, 30], // 2 yellow
                [66, 160, 66],  // 3 green
                [55, 120, 220], // 4 blue
                [150, 80, 200], // 5 purple
                [60, 180, 200], // 6 cyan
                [150, 100, 50], // 7 brown
                [60, 60, 60],   // 8 trash
            ],
        }
    }
}

impl ColorClassScheme {
    pub fn name(&self, c: ColorClass) -> Option<&str> {
        c.index().map(|i| self.names[i].as_str())
    }
    pub fn color(&self, c: ColorClass) -> Option<[u8; 3]> {
        c.index().map(|i| self.colors[i])
    }
    /// Reverse lookup from a label string (case-insensitive), also accepting
    /// Photo Mechanic's classic names and Lightroom's colour names.
    pub fn class_for_label(&self, label: &str) -> Option<ColorClass> {
        let l = label.trim().to_lowercase();
        if l.is_empty() {
            return None;
        }
        if let Some(i) = self.names.iter().position(|n| n.to_lowercase() == l) {
            return Some(ColorClass(i as u8 + 1));
        }
        match l.as_str() {
            "red" | "winner" => Some(ColorClass(1)),
            "yellow" | "winner alt" => Some(ColorClass(2)),
            "green" | "superior" => Some(ColorClass(3)),
            "blue" | "superior alt" => Some(ColorClass(4)),
            "purple" | "excellent" => Some(ColorClass(5)),
            "cyan" | "excellent alt" => Some(ColorClass(6)),
            "brown" | "review" => Some(ColorClass(7)),
            "trash" => Some(ColorClass(8)),
            _ => None,
        }
    }
}

/// Everything a cull pass records about one item.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Marks {
    pub rating: Rating,
    pub color_class: ColorClass,
    pub tagged: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rating_clamps() {
        assert_eq!(Rating::clamp(9).0, 5);
        assert_eq!(Rating::clamp(-4).0, -1);
        assert!(Rating(-1).is_rejected());
        assert_eq!(Rating(3).stars(), 3);
    }

    #[test]
    fn label_round_trip() {
        let s = ColorClassScheme::default();
        assert_eq!(s.name(ColorClass(1)), Some("Red"));
        assert_eq!(s.class_for_label("winner alt"), Some(ColorClass(2)));
        assert_eq!(s.class_for_label("Yellow"), Some(ColorClass(2)));
        assert_eq!(s.class_for_label("Green"), Some(ColorClass(3)));
        assert_eq!(s.class_for_label("trash"), Some(ColorClass(8)));
        assert_eq!(s.class_for_label(""), None);
        assert_eq!(s.name(ColorClass::NONE), None);
    }
}
