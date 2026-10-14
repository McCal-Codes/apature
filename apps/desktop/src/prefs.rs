//! Per-user preferences and panel memory, stored as JSON under the app's
//! Application Support directory. Everything here is a convenience; losing
//! the file loses nothing photographic.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Appearance {
    #[default]
    System,
    Dark,
    Light,
}

/// Export workspace defaults (the desk's delivery spec).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportPrefs {
    /// Long side in pixels, 0 = full size.
    pub long_side: u32,
    pub quality: u8,
    pub dest: Option<PathBuf>,
    /// Naming template; empty keeps the source names.
    pub rename: String,
    pub keep_exif: bool,
    pub write_iim: bool,
    pub open_after: bool,
    /// Render RAW-only photos with the system RAW engine (macOS) instead of the preview.
    pub decode_raw: bool,
}

impl Default for ExportPrefs {
    fn default() -> Self {
        ExportPrefs {
            long_side: 2048,
            quality: 90,
            dest: None,
            rename: String::new(),
            keep_exif: true,
            write_iim: true,
            open_after: true,
            decode_raw: false,
        }
    }
}

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
    pub assignment_file: Option<PathBuf>,
    pub codes_file: Option<PathBuf>,
    pub ingest: Option<apature_ingest::Options>,
    pub scheme: apature_core::ColorClassScheme,
    pub appearance: Appearance,
    pub open_tabs: Vec<PathBuf>,
    pub active_tab: usize,
    pub code_delimiter: char,
    pub open_after_ingest: bool,
    /// When a volume with a DCIM folder mounts, switch to Ingest with it as the source.
    pub open_ingest_on_card: bool,
    pub export: ExportPrefs,
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
            assignment_file: None,
            codes_file: None,
            ingest: None,
            scheme: apature_core::ColorClassScheme::default(),
            appearance: Appearance::System,
            open_tabs: Vec::new(),
            active_tab: 0,
            code_delimiter: '\\',
            open_after_ingest: true,
            open_ingest_on_card: true,
            export: ExportPrefs::default(),
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
