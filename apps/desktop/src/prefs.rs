//! Per-user preferences and panel memory, stored as JSON under the app's
//! Application Support directory. Everything here is a convenience; losing
//! the file loses nothing photographic.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub sidebar_visible: bool,
    pub sidebar_width: f32,
    pub inspector_visible: bool,
    pub inspector_width: f32,
    pub filmstrip_visible: bool,
    pub cell: f32,
    pub favorites: Vec<PathBuf>,
    pub recent: Vec<PathBuf>,
    pub sort: crate::filter::Sort,
    pub sort_reversed: bool,
    pub advance_after_mark: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            sidebar_visible: true,
            sidebar_width: 240.0,
            inspector_visible: true,
            inspector_width: 320.0,
            filmstrip_visible: false,
            cell: 180.0,
            favorites: Vec::new(),
            recent: Vec::new(),
            sort: crate::filter::Sort::Name,
            sort_reversed: false,
            advance_after_mark: true,
        }
    }
}

impl Prefs {
    pub fn path() -> Option<PathBuf> {
        directories::ProjectDirs::from("dev", "mccal", "Apature")
            .map(|d| d.config_dir().join("prefs.json"))
    }

    pub fn load() -> Prefs {
        Self::path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(p) = Self::path() else { return };
        if let Some(dir) = p.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let tmp = p.with_extension("json.tmp");
            if fs::write(&tmp, text).is_ok() {
                let _ = fs::rename(tmp, p);
            }
        }
    }

    pub fn note_recent(&mut self, folder: &Path) {
        self.recent.retain(|r| r != folder);
        self.recent.insert(0, folder.to_path_buf());
        self.recent.truncate(10);
    }
}
