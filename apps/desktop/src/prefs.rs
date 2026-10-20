//! Per-user preferences and panel memory, stored as JSON under the app's
//! Application Support directory. Everything here is a convenience; losing
//! the file loses nothing photographic.

use std::collections::BTreeMap;
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
    /// Write JPEG XL instead of JPEG (lossless transcode of the JPEG via cjxl).
    pub jxl: bool,
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
            jxl: false,
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
    /// Group frames shot within a second of the previous one behind one
    /// thumbnail (bursts); S expands or collapses the focused stack.
    pub stack_bursts: bool,
    /// What the bare 1–8 keys set: colour class (Photo Mechanic) or star rating.
    pub number_keys: NumberKeys,
    pub assignment_file: Option<PathBuf>,
    pub codes_file: Option<PathBuf>,
    pub ingest: Option<apature_ingest::Options>,
    pub scheme: apature_core::ColorClassScheme,
    pub appearance: Appearance,
    pub open_tabs: Vec<PathBuf>,
    pub active_tab: usize,
    pub code_delimiter: char,
    /// Expand \code\ pairs while typing captions (off = type delimiters literally).
    pub codes_enabled: bool,
    pub open_after_ingest: bool,
    /// When a volume with a DCIM folder mounts, switch to Ingest with it as the source.
    pub open_ingest_on_card: bool,
    /// `{seqn}` keeps counting across ingests with the same headline
    /// (Photo Mechanic's persistent sequence variable, one counter per slug).
    pub seq_continue: bool,
    /// Next sequence number per headline key (lower-cased, trimmed; "" = none).
    pub seq_counters: BTreeMap<String, u32>,
    /// Recent values per caption field (headline, credit, city…), newest first,
    /// offered from the ▾ next to the field like Photo Mechanic's autocomplete.
    pub caption_history: BTreeMap<String, Vec<String>>,
    /// Accessibility: whole-UI scale (text and controls), 1.0 = default.
    pub ui_scale: f32,
    /// Accessibility: follow the system Reduce Motion setting, or force it.
    pub motion: Motion,
    /// Accessibility: stronger text and borders.
    pub high_contrast: bool,
    pub export: ExportPrefs,
    /// Export presets per outlet: size, quality, naming, format and the
    /// delivery target in one chip.
    pub export_presets: Vec<ExportPreset>,
    /// The preset last applied (by name); cleared when the values drift.
    pub export_preset: Option<String>,
    /// Delivery targets (FTP, FTPS, SFTP). Passwords live in the Keychain.
    pub delivery: Vec<apature_deliver::Target>,
    /// Send every finished export to this target (by name) automatically.
    pub send_after_export: Option<String>,
    /// Last template used in Rename Photos.
    pub rename_template: String,
    /// Eject the card after an ingest where every file verified.
    pub eject_after_ingest: bool,
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
            stack_bursts: false,
            number_keys: NumberKeys::Rating,
            assignment_file: None,
            codes_file: None,
            ingest: None,
            scheme: apature_core::ColorClassScheme::default(),
            appearance: Appearance::System,
            open_tabs: Vec::new(),
            active_tab: 0,
            code_delimiter: '\\',
            codes_enabled: true,
            open_after_ingest: true,
            open_ingest_on_card: true,
            seq_continue: true,
            seq_counters: BTreeMap::new(),
            caption_history: BTreeMap::new(),
            ui_scale: 1.0,
            motion: Motion::System,
            high_contrast: false,
            export: ExportPrefs::default(),
            export_presets: Vec::new(),
            export_preset: None,
            delivery: Vec::new(),
            send_after_export: None,
            rename_template: String::new(),
            eject_after_ingest: false,
        }
    }
}

/// The app's ingest defaults: the owner's TribLive convention without the
/// section prefix, one folder per shoot and files `Headline001-MMDDYY.ext`.
pub fn default_ingest_options() -> apature_ingest::Options {
    apature_ingest::Options {
        folder_template: "{year4}/{month0} {monthname}/{mnn3} {day0} {headline}".into(),
        rename_template: "{headline}{seqn:3}-{month0}{day0}{yr2}".into(),
        ..Default::default()
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

/// Motion preference: follow macOS Reduce Motion, or override it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Motion {
    #[default]
    System,
    Reduce,
    Full,
}

/// Key for the per-headline sequence counter.
pub fn seq_key(headline: Option<&str>) -> String {
    headline.unwrap_or("").trim().to_lowercase()
}

/// What the bare number keys do on the sheet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberKeys {
    /// 1–5 set the star rating, 0 clears it; ⌘1–⌘8 set the colour class.
    #[default]
    Rating,
    /// 1–8 set the colour class, 0 clears it; ⌘0–⌘5 set the rating (Photo Mechanic).
    ColorClass,
}

impl Prefs {
    /// Remember a field value for the ▾ menu: newest first, no duplicates, 12 kept.
    pub fn remember_caption_value(&mut self, field: &str, value: &str) {
        let v = value.trim();
        if v.is_empty() {
            return;
        }
        let list = self.caption_history.entry(field.to_string()).or_default();
        list.retain(|x| x != v);
        list.insert(0, v.to_string());
        list.truncate(12);
    }
}

/// One outlet's export spec. `dest` is kept too, so a preset can point at a
/// fixed folder (an Eagle outbox) or leave the last folder alone (`None`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportPreset {
    pub name: String,
    pub export: ExportPrefs,
    pub send_to: Option<String>,
}

impl Prefs {
    /// True when the current export settings still match the named preset.
    pub fn export_matches_preset(&self, name: &str) -> bool {
        self.export_presets
            .iter()
            .find(|p| p.name == name)
            .is_some_and(|p| {
                let mut e = self.export.clone();
                if p.export.dest.is_none() {
                    e.dest = None;
                }
                e == p.export && self.send_after_export == p.send_to
            })
    }

    pub fn apply_export_preset(&mut self, name: &str) {
        if let Some(p) = self.export_presets.iter().find(|p| p.name == name).cloned() {
            let keep_dest = self.export.dest.clone();
            self.export = p.export.clone();
            if self.export.dest.is_none() {
                self.export.dest = keep_dest;
            }
            self.send_after_export = p.send_to.clone();
            self.export_preset = Some(name.to_string());
        }
    }
}
