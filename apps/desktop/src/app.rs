//! Application state and the single command dispatcher. Drawing lives in
//! `ui/`; native integration in `platform/`; decoding in `worker.rs`;
//! selection, filters and undo are pure modules with their own tests.

use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::Instant;

use egui::{ColorImage, Key, TextureHandle, TextureOptions};
use rayon::prelude::*;

use apature_core::{ColorClass, ColorClassScheme, Marks, PhotoItem, Rating, ScanOptions};
use apature_metadata::items;
use apature_metadata::ExifSummary;
use apature_preview::{Rgb, ThumbCache};

use apature_metadata::codes::{self, CodeTable, Vars};
use apature_metadata::stationery::Assignment;

use crate::commands::{Command, Workspace};
use crate::filter::{self, Filter};
use crate::platform::Platform;
use crate::prefs::Prefs;
use crate::selection::{Mods, Selection};
use crate::ui::caption::{Draft, DraftFields};
use crate::ui::{self, tokens::Theme};
use crate::undo::{Change, Entry, UndoStack};
use crate::worker::{Job, JobKind, Pool};
use crate::Launch;

const THUMB_PX: u32 = 320;
const PREFETCH_ROWS: usize = 3;
const KEEP_ROWS: usize = 12;
const PREVIEW_LRU: usize = 6;

#[derive(Default)]
pub struct Stats {
    pub thumbs_done: usize,
    pub cache_hits: usize,
    pub decode_ms_total: f32,
    pub scan_ms: f32,
    pub frame_ms_ema: f32,
    pub frame_ms_max: f32,
    pub textures_uploaded: usize,
}

pub struct App {
    pub folder: Option<PathBuf>,
    pub items: Vec<PhotoItem>,
    pub marks: Vec<Marks>,
    pub conflicts: Vec<Vec<String>>,
    pub exif: Vec<Option<ExifSummary>>,
    pub thumbs: Vec<Option<TextureHandle>>,
    pub thumb_err: Vec<Option<String>>,
    pub previews: HashMap<usize, (TextureHandle, u32)>,
    preview_lru: VecDeque<usize>,
    pub pool: Pool,
    generation: u64,
    /// Focused item (an index into `items`).
    pub cursor: usize,
    pub selection: Selection,
    /// Visible, ordered item indices after filter and sort.
    pub view: Vec<usize>,
    /// Position of each item in `view`, `None` when filtered out.
    pub pos: Vec<Option<usize>>,
    pub filter: Filter,
    pub undo: UndoStack,
    pub workspace: Workspace,
    pub cols: usize,
    pub last_visible_rows: Range<usize>,
    pub scroll_to_cursor: bool,
    pub scheme: ColorClassScheme,
    pub stats: Stats,
    pub status: String,
    pub prefs: Prefs,
    pub prefs_dirty: bool,
    prefs_saved_at: Instant,
    pub volumes: Vec<(String, PathBuf)>,
    pub filter_popup_open: bool,
    pub caption_draft: Option<Draft>,
    pub assignment: Option<Assignment>,
    pub codes: CodeTable,
    platform: Platform,
    reduce_motion: bool,
    launch: Launch,
    frames: u64,
    screenshot_requested: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, launch: Launch) -> Self {
        let cache = directories::ProjectDirs::from("dev", "mccal", "Apature")
            .and_then(|d| ThumbCache::new(d.cache_dir().join("thumbs")).ok());
        let threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let threads = (threads.saturating_sub(1)).clamp(2, 12);
        let pool = Pool::new(cc.egui_ctx.clone(), cache, threads);
        if let Some(theme) = launch.theme {
            cc.egui_ctx.set_theme(theme);
        }
        ui::tokens::install_fonts(&cc.egui_ctx);
        let prefs = Prefs::load();
        let platform = Platform::new();
        let mut app = App {
            folder: None,
            items: Vec::new(),
            marks: Vec::new(),
            conflicts: Vec::new(),
            exif: Vec::new(),
            thumbs: Vec::new(),
            thumb_err: Vec::new(),
            previews: HashMap::new(),
            preview_lru: VecDeque::new(),
            pool,
            generation: 0,
            cursor: 0,
            selection: Selection::default(),
            view: Vec::new(),
            pos: Vec::new(),
            filter: Filter::default(),
            undo: UndoStack::default(),
            workspace: Workspace::Sheet,
            cols: 1,
            last_visible_rows: 0..0,
            scroll_to_cursor: false,
            scheme: ColorClassScheme::default(),
            stats: Stats::default(),
            status: String::new(),
            prefs,
            prefs_dirty: false,
            prefs_saved_at: Instant::now(),
            volumes: Vec::new(),
            filter_popup_open: false,
            caption_draft: None,
            assignment: None,
            codes: CodeTable::default(),
            platform,
            reduce_motion: Platform::reduce_motion(),
            launch: launch.clone(),
            frames: 0,
            screenshot_requested: false,
        };
        app.sync_menu();
        app.refresh_volumes();
        if let Some(p) = app.prefs.assignment_file.clone() {
            app.load_assignment(&p);
        }
        if let Some(p) = app.prefs.codes_file.clone() {
            app.load_codes(&p);
        }
        if let Some(p) = launch.assignment.clone() {
            app.load_assignment(&p);
        }
        let first = launch
            .folder
            .clone()
            .or_else(|| app.prefs.recent.first().cloned())
            .filter(|f| f.is_dir());
        if let Some(f) = first {
            app.open_folder(&f);
        }
        if launch.demo && !app.view.is_empty() {
            app.selection.all(&app.view);
            app.cursor = app.view[0];
            app.filter_popup_open = true;
        }
        if launch.caption && !app.view.is_empty() {
            app.workspace = Workspace::Caption;
        }
        app
    }

    // ----- captions ----------------------------------------------------------

    fn load_assignment(&mut self, path: &Path) {
        match std::fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|t| Assignment::parse(&t).map_err(|e| e.to_string()))
        {
            Ok(a) => {
                self.codes.merge(&a.codes());
                self.status = format!(
                    "Assignment loaded: {} ({} codes)",
                    a.slug,
                    self.codes.codes.len()
                );
                self.assignment = Some(a);
                self.prefs.assignment_file = Some(path.to_path_buf());
                self.prefs_dirty = true;
            }
            Err(e) => self.status = format!("Could not load assignment {}: {e}", path.display()),
        }
    }

    fn load_codes(&mut self, path: &Path) {
        match std::fs::read_to_string(path) {
            Ok(t) => {
                let table = CodeTable::parse(&t);
                self.codes.merge(&table);
                self.status = format!(
                    "{} code replacements loaded from {}",
                    table.codes.len(),
                    path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
                );
                self.prefs.codes_file = Some(path.to_path_buf());
                self.prefs_dirty = true;
            }
            Err(e) => self.status = format!("Could not read {}: {e}", path.display()),
        }
    }

    fn caption_vars(&self, draft: &Draft) -> Vars {
        Vars {
            date: draft
                .date_created
                .as_deref()
                .and_then(codes::parse_date)
                .or_else(|| {
                    self.exif[draft.idx]
                        .as_ref()
                        .and_then(|e| e.date_time_original.as_deref())
                        .and_then(codes::parse_date)
                }),
            credit: if draft.fields.credit.is_empty() {
                self.assignment.as_ref().and_then(|a| a.credit_line())
            } else {
                Some(draft.fields.credit.clone())
            },
            headline: if draft.fields.headline.is_empty() {
                None
            } else {
                Some(draft.fields.headline.clone())
            },
        }
    }

    /// Keep the draft on the focused photo; save a dirty draft before moving on.
    fn sync_caption_draft(&mut self) {
        if self.items.is_empty() {
            self.caption_draft = None;
            return;
        }
        if self
            .caption_draft
            .as_ref()
            .is_some_and(|d| d.idx == self.cursor)
        {
            return;
        }
        self.save_caption(false);
        let it = &self.items[self.cursor];
        let iptc = items::read_iptc(it);
        let fields = DraftFields::from_iptc(&iptc);
        self.caption_draft = Some(Draft {
            idx: self.cursor,
            fields: fields.clone(),
            original: fields,
            date_created: iptc.date_created.clone(),
        });
    }

    /// Write the draft's changed fields. Returns true when something was written.
    fn save_caption(&mut self, announce: bool) -> bool {
        let Some(draft) = self.caption_draft.as_ref() else {
            return false;
        };
        if !draft.dirty() || draft.idx >= self.items.len() {
            return false;
        }
        let changes = draft.fields.changes_from(&draft.original);
        let idx = draft.idx;
        match items::write_iptc(&self.items[idx], &changes) {
            Ok(w) => {
                if let Some(d) = self.caption_draft.as_mut() {
                    d.original = d.fields.clone();
                }
                if announce {
                    self.status = format!(
                        "Caption saved: {} → {} file{}",
                        self.items[idx].display_name(),
                        w.len(),
                        if w.len() == 1 { "" } else { "s" }
                    );
                }
                true
            }
            Err(e) => {
                self.status = format!("Could not save caption: {e}");
                false
            }
        }
    }

    fn apply_stationery(&mut self) {
        let Some(pad) = self.assignment.as_ref().map(|a| a.iptc()) else {
            return;
        };
        let targets = self.selection.targets(self.cursor);
        let mut written = 0;
        let mut failed = None;
        for &i in &targets {
            match items::write_iptc(&self.items[i], &pad) {
                Ok(w) => written += w.len(),
                Err(e) => failed = Some(e.to_string()),
            }
        }
        self.status = match failed {
            Some(e) => format!("Pad failed: {e}"),
            None => format!(
                "Assignment pad applied to {} photo{} → {written} files",
                targets.len(),
                if targets.len() == 1 { "" } else { "s" }
            ),
        };
        self.caption_draft = None;
        self.sync_caption_draft();
    }

    fn insert_caption_template(&mut self) {
        let Some(tpl) = self.assignment.as_ref().map(|a| a.caption_template()) else {
            return;
        };
        let Some(draft) = self.caption_draft.as_ref() else {
            return;
        };
        let vars = self.caption_vars(draft);
        let text = codes::expand_variables(&tpl, &vars);
        if let Some(d) = self.caption_draft.as_mut() {
            d.fields.caption = text;
        }
    }

    fn sync_menu(&self) {
        self.platform.sync_toggles(
            self.prefs.sidebar_visible,
            self.prefs.inspector_visible,
            self.prefs.filmstrip_visible,
            self.prefs.advance_after_mark,
        );
    }

    // ----- folders -----------------------------------------------------------

    fn refresh_volumes(&mut self) {
        let mut v = Vec::new();
        if let Ok(rd) = std::fs::read_dir("/Volumes") {
            for e in rd.flatten() {
                let p = e.path();
                let name = e.file_name().to_string_lossy().into_owned();
                if p.is_dir() && !name.starts_with('.') {
                    v.push((name, p));
                }
            }
        }
        v.sort();
        self.volumes = v;
    }

    pub fn open_folder(&mut self, dir: &Path) {
        let t0 = Instant::now();
        match apature_core::scan(dir, &ScanOptions::default()) {
            Ok(items) => {
                self.pool.clear_pending();
                self.generation += 1;
                let scheme = self.scheme.clone();
                // Marks, conflicts and EXIF for every item up front: all three are
                // cheap (sidecars are tiny, EXIF is a few page faults via mmap) and
                // sorting by capture time needs them before the first frame.
                let read: Vec<(Marks, Vec<String>, Option<ExifSummary>)> = items
                    .par_iter()
                    .map(|it| {
                        let src = items::read(it);
                        let m = src.effective(it, &scheme);
                        let c = src
                            .conflicts()
                            .into_iter()
                            .map(|(p, c)| {
                                format!(
                                    "{}: sidecar {} vs {} {}",
                                    c.field,
                                    c.sidecar,
                                    p.file_name()
                                        .map(|n| n.to_string_lossy().into_owned())
                                        .unwrap_or_default(),
                                    c.embedded
                                )
                            })
                            .collect();
                        let e = apature_metadata::exif::read(it.path()).ok();
                        (m, c, e)
                    })
                    .collect();
                let n = items.len();
                self.items = items;
                self.marks = read.iter().map(|(m, _, _)| *m).collect();
                self.conflicts = read.iter().map(|(_, c, _)| c.clone()).collect();
                self.exif = read.into_iter().map(|(_, _, e)| e).collect();
                self.thumbs = vec![None; n];
                self.thumb_err = vec![None; n];
                self.previews.clear();
                self.preview_lru.clear();
                self.selection.clear();
                self.undo.clear();
                self.filter = Filter::default();
                self.rebuild_view();
                self.cursor = self.view.first().copied().unwrap_or(0);
                self.workspace = Workspace::Sheet;
                self.folder = Some(dir.to_path_buf());
                self.stats = Stats {
                    scan_ms: t0.elapsed().as_secs_f32() * 1000.0,
                    ..Default::default()
                };
                self.status = String::new();
                self.prefs.note_recent(dir);
                self.prefs_dirty = true;
                self.refresh_volumes();
            }
            Err(e) => self.status = format!("Could not open {}: {e}", dir.display()),
        }
    }

    fn pick_folder(&mut self) {
        let mut dlg = rfd::FileDialog::new();
        if let Some(f) = &self.folder {
            dlg = dlg.set_directory(f);
        }
        if let Some(dir) = dlg.pick_folder() {
            self.open_folder(&dir);
        }
    }

    // ----- view (filter + sort) ----------------------------------------------

    pub fn rebuild_view(&mut self) {
        self.view = filter::build_view(
            &self.items,
            &self.marks,
            &self.exif,
            &self.filter,
            self.prefs.sort,
            self.prefs.sort_reversed,
        );
        self.pos = vec![None; self.items.len()];
        for (p, &i) in self.view.iter().enumerate() {
            self.pos[i] = Some(p);
        }
        let visible: Vec<bool> = self.pos.iter().map(|p| p.is_some()).collect();
        self.selection.retain_visible(&visible);
        if self.pos.get(self.cursor).copied().flatten().is_none() {
            // Focused item got filtered out: move focus to the nearest visible one.
            if let Some(&first) = self.view.first() {
                self.cursor = first;
            }
        }
        self.scroll_to_cursor = true;
    }

    pub fn cursor_pos(&self) -> usize {
        self.pos.get(self.cursor).copied().flatten().unwrap_or(0)
    }

    // ----- background results ------------------------------------------------

    fn drain_results(&mut self, ctx: &egui::Context) {
        let mut n = 0;
        while let Some(done) = self.pool.try_recv() {
            if done.generation != self.generation || done.idx >= self.items.len() {
                continue;
            }
            if let (Some(e), None) = (done.exif, &self.exif[done.idx]) {
                self.exif[done.idx] = Some(e);
            }
            match done.result {
                Ok(rgb) => {
                    let tex = upload(ctx, &format!("{:?}{}", done.kind, done.idx), &rgb);
                    self.stats.textures_uploaded += 1;
                    match done.kind {
                        JobKind::Thumb => {
                            self.thumbs[done.idx] = Some(tex);
                            self.thumb_err[done.idx] = None;
                            self.stats.thumbs_done += 1;
                            if done.cache_hit {
                                self.stats.cache_hits += 1;
                            }
                            self.stats.decode_ms_total += done.millis;
                        }
                        JobKind::Preview => {
                            self.previews.insert(done.idx, (tex, rgb.long_side()));
                            self.preview_lru.retain(|&i| i != done.idx);
                            self.preview_lru.push_back(done.idx);
                            self.evict_previews();
                        }
                    }
                }
                Err(e) => {
                    if done.kind == JobKind::Thumb {
                        self.thumb_err[done.idx] = Some(e);
                    } else {
                        self.status = format!("Preview failed: {e}");
                    }
                }
            }
            n += 1;
            if n >= 48 {
                ctx.request_repaint();
                break;
            }
        }
    }

    fn evict_previews(&mut self) {
        while self.preview_lru.len() > PREVIEW_LRU {
            let keep = |i: usize| i + 1 >= self.cursor && i <= self.cursor + 1;
            let Some(p) = self.preview_lru.iter().position(|&i| !keep(i)) else {
                break;
            };
            let idx = self.preview_lru.remove(p).unwrap();
            self.previews.remove(&idx);
        }
    }

    pub fn request_thumb(&self, idx: usize, prio: u32) {
        if idx >= self.items.len() || self.thumbs[idx].is_some() || self.thumb_err[idx].is_some() {
            return;
        }
        let it = &self.items[idx];
        self.pool.request(
            Job {
                idx,
                kind: JobKind::Thumb,
                path: it.primary.path.clone(),
                size: it.primary.size,
                modified: it.primary.modified,
                long_side: THUMB_PX,
                generation: self.generation,
            },
            prio,
        );
    }

    pub fn request_preview(&self, idx: usize, long_side: u32, prio: u32) {
        if idx >= self.items.len() {
            return;
        }
        if let Some((_, have)) = self.previews.get(&idx) {
            if *have >= long_side.min(6000) || *have >= 1600 && long_side > *have {
                return;
            }
        }
        let it = &self.items[idx];
        self.pool.request(
            Job {
                idx,
                kind: JobKind::Preview,
                path: it.primary.path.clone(),
                size: it.primary.size,
                modified: it.primary.modified,
                long_side,
                generation: self.generation,
            },
            prio,
        );
    }

    /// Neighbour in view order, for preview prefetch.
    pub fn view_neighbor(&self, delta: isize) -> Option<usize> {
        let p = self.cursor_pos() as isize + delta;
        if p < 0 {
            return None;
        }
        self.view.get(p as usize).copied()
    }

    pub fn schedule_thumbs(&mut self, visible_rows: Range<usize>) {
        if self.view.is_empty() || self.cols == 0 {
            return;
        }
        let rows = self.row_count();
        let first = visible_rows.start.saturating_sub(PREFETCH_ROWS);
        let last = (visible_rows.end + PREFETCH_ROWS).min(rows);
        for row in first..last {
            let prio = if row < visible_rows.start {
                10 + (visible_rows.start - row) as u32
            } else if row >= visible_rows.end {
                10 + (row + 1 - visible_rows.end) as u32
            } else {
                (row - visible_rows.start) as u32
            };
            for p in row * self.cols..((row + 1) * self.cols).min(self.view.len()) {
                self.request_thumb(self.view[p], prio);
            }
        }
        let keep_first = visible_rows.start.saturating_sub(KEEP_ROWS);
        let keep_last = (visible_rows.end + KEEP_ROWS).min(rows);
        for row in (0..keep_first).chain(keep_last..rows) {
            for p in row * self.cols..((row + 1) * self.cols).min(self.view.len()) {
                let idx = self.view[p];
                if self.thumbs[idx].is_some() && idx != self.cursor {
                    self.thumbs[idx] = None;
                    self.stats.thumbs_done = self.stats.thumbs_done.saturating_sub(1);
                }
            }
        }
    }

    // ----- marks -------------------------------------------------------------

    /// Apply `edit` to every target, record one undo entry, write files.
    fn mark(&mut self, label: &str, edit: impl Fn(&mut Marks)) {
        if self.items.is_empty() {
            return;
        }
        let targets = self.selection.targets(self.cursor);
        let mut changes = Vec::with_capacity(targets.len());
        for &i in &targets {
            let before = self.marks[i];
            let mut after = before;
            edit(&mut after);
            if after != before {
                changes.push(Change {
                    idx: i,
                    before,
                    after,
                });
            }
        }
        if changes.is_empty() {
            return;
        }
        let apply: Vec<(usize, Marks)> = changes.iter().map(|c| (c.idx, c.after)).collect();
        self.undo.push(Entry {
            label: format!(
                "{label} ({} photo{})",
                changes.len(),
                if changes.len() == 1 { "" } else { "s" }
            ),
            changes,
        });
        self.apply_marks(&apply);
        if self.prefs.advance_after_mark
            && targets.len() == 1
            && self.workspace != Workspace::Caption
        {
            self.move_focus(1, Mods::default());
        }
    }

    fn apply_marks(&mut self, apply: &[(usize, Marks)]) {
        let mut written = 0;
        let mut failed = None;
        for &(i, m) in apply {
            self.marks[i] = m;
            match items::write(&self.items[i], &m, &self.scheme) {
                Ok(w) => {
                    written += w.len();
                    self.conflicts[i].clear();
                }
                Err(e) => failed = Some(e.to_string()),
            }
        }
        self.status = match failed {
            Some(e) => format!("Could not write marks: {e}"),
            None if apply.len() == 1 => {
                let (i, m) = apply[0];
                format!(
                    "{}: {} · {} · {} → {} file{}",
                    self.items[i].display_name(),
                    ui::rating_text(m.rating),
                    self.scheme.name(m.color_class).unwrap_or("no class"),
                    if m.tagged { "tagged" } else { "untagged" },
                    written,
                    if written == 1 { "" } else { "s" }
                )
            }
            None => format!("{} photos marked → {written} files written", apply.len()),
        };
        // Marks may change what the filter shows and the sort order.
        if self.filter.is_active()
            || matches!(
                self.prefs.sort,
                filter::Sort::Rating | filter::Sort::ColorClass
            )
        {
            self.rebuild_view();
        }
    }

    // ----- navigation --------------------------------------------------------

    pub fn row_count(&self) -> usize {
        if self.cols == 0 {
            0
        } else {
            self.view.len().div_ceil(self.cols)
        }
    }

    fn move_focus(&mut self, delta: isize, mods: Mods) {
        if self.view.is_empty() {
            return;
        }
        let n = self.view.len() as isize;
        let p = (self.cursor_pos() as isize + delta).clamp(0, n - 1) as usize;
        self.cursor = self.view[p];
        self.selection.focus_moved(self.cursor, mods, &self.view);
        self.scroll_to_cursor = true;
    }

    fn focus_to_pos(&mut self, p: usize, mods: Mods) {
        if let Some(&idx) = self.view.get(p) {
            self.cursor = idx;
            self.selection.focus_moved(idx, mods, &self.view);
            self.scroll_to_cursor = true;
        }
    }

    pub fn set_cell(&mut self, cell: f32) {
        self.prefs.cell = cell.clamp(80.0, 600.0);
        self.prefs_dirty = true;
        self.scroll_to_cursor = true;
    }

    pub fn cell_accessible_name(&self, idx: usize) -> String {
        let m = self.marks[idx];
        let mut parts = vec![self.items[idx].display_name()];
        if m.rating.is_rejected() {
            parts.push("rejected".into());
        } else if m.rating.stars() > 0 {
            parts.push(format!("{} stars", m.rating.stars()));
        }
        if let Some(n) = self.scheme.name(m.color_class) {
            parts.push(format!("class {} {n}", m.color_class.0));
        }
        if m.tagged {
            parts.push("tagged".into());
        }
        if self.selection.contains(idx) {
            parts.push("selected".into());
        }
        parts.push(format!(
            "{} of {}",
            self.pos
                .get(idx)
                .copied()
                .flatten()
                .map(|p| p + 1)
                .unwrap_or(0),
            self.view.len()
        ));
        parts.join(", ")
    }

    pub fn status_parts(&self) -> Vec<String> {
        let mut parts = Vec::new();
        match &self.folder {
            None => parts.push("No folder open · ⌘O".into()),
            Some(f) => {
                let name = f
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if self.filter.is_active() {
                    parts.push(format!(
                        "{name} · showing {} of {}",
                        self.view.len(),
                        self.items.len()
                    ));
                } else {
                    parts.push(format!("{name} · {} photos", self.items.len()));
                }
            }
        }
        if self.selection.len() > 1 {
            parts.push(format!("{} selected", self.selection.len()));
        }
        if let Some(it) = self.items.get(self.cursor) {
            parts.push(format!(
                "{}/{} {}",
                self.cursor_pos() + 1,
                self.view.len(),
                it.display_name()
            ));
            if let Some(e) = &self.exif[self.cursor] {
                let ex = e.exposure_line();
                if !ex.is_empty() {
                    parts.push(ex);
                }
            }
            let m = self.marks[self.cursor];
            let stars = ui::rating_text(m.rating);
            if !stars.is_empty() {
                parts.push(stars);
            }
            if let Some(n) = self.scheme.name(m.color_class) {
                parts.push(format!("{} {n}", m.color_class.0));
            }
            if m.tagged {
                parts.push("tagged".into());
            }
        }
        parts
    }

    // ----- commands ----------------------------------------------------------

    pub fn dispatch(&mut self, ctx: &egui::Context, cmd: Command) {
        match cmd {
            Command::OpenFolder(p) => self.open_folder(&p),
            Command::PickFolder => self.pick_folder(),
            Command::AddFavorite => {
                if let Some(f) = self.folder.clone() {
                    if !self.prefs.favorites.contains(&f) {
                        self.prefs.favorites.push(f);
                        self.prefs_dirty = true;
                    }
                }
            }
            Command::RemoveFavorite(p) => {
                self.prefs.favorites.retain(|f| f != &p);
                self.prefs_dirty = true;
            }
            Command::RevealInFinder => {
                if let Some(it) = self.items.get(self.cursor) {
                    Platform::reveal(it.path());
                }
            }
            Command::ToggleSidebar => {
                self.prefs.sidebar_visible = !self.prefs.sidebar_visible;
                self.prefs_dirty = true;
            }
            Command::ToggleInspector => {
                self.prefs.inspector_visible = !self.prefs.inspector_visible;
                self.prefs_dirty = true;
            }
            Command::ToggleFilmstrip => {
                self.prefs.filmstrip_visible = !self.prefs.filmstrip_visible;
                self.prefs_dirty = true;
            }
            Command::SetWorkspace(w) => {
                if w.available().is_ok() && !self.items.is_empty() {
                    self.workspace = w;
                    self.scroll_to_cursor = true;
                }
            }
            Command::ZoomIn => self.set_cell(self.prefs.cell * 1.2),
            Command::ZoomOut => self.set_cell(self.prefs.cell / 1.2),
            Command::Click {
                idx,
                shift,
                command,
            } => {
                if self.pos.get(idx).copied().flatten().is_some() {
                    self.cursor = idx;
                    self.selection
                        .click(idx, Mods { shift, command }, &self.view);
                }
            }
            Command::MoveFocus {
                delta,
                shift,
                command,
            } => self.move_focus(delta, Mods { shift, command }),
            Command::Move(d) => self.move_focus(d, Mods::default()),
            Command::MovePage(d) => {
                let page = (self.last_visible_rows.len().max(1) * self.cols.max(1)) as isize;
                self.move_focus(d * page, Mods::default());
            }
            Command::Home => self.focus_to_pos(0, Mods::default()),
            Command::End => self.focus_to_pos(self.view.len().saturating_sub(1), Mods::default()),
            Command::SelectAll => self.selection.all(&self.view),
            Command::SelectNone => self.selection.clear(),
            Command::Undo => {
                if let Some((label, apply)) = self.undo.undo() {
                    self.apply_marks(&apply);
                    self.status = format!("Undid {label}");
                }
            }
            Command::Redo => {
                if let Some((label, apply)) = self.undo.redo() {
                    self.apply_marks(&apply);
                    self.status = format!("Redid {label}");
                }
            }
            Command::SetFilter(f) => {
                self.filter = f;
                self.rebuild_view();
            }
            Command::ClearFilter(None) => {
                self.filter = Filter::default();
                self.rebuild_view();
            }
            Command::ClearFilter(Some(c)) => {
                self.filter.clear(c);
                self.rebuild_view();
            }
            Command::SetSort(s) => {
                self.prefs.sort = s;
                self.prefs_dirty = true;
                self.rebuild_view();
            }
            Command::ToggleSortReversed => {
                self.prefs.sort_reversed = !self.prefs.sort_reversed;
                self.prefs_dirty = true;
                self.rebuild_view();
            }
            Command::ToggleAdvanceAfterMark => {
                self.prefs.advance_after_mark = !self.prefs.advance_after_mark;
                self.prefs_dirty = true;
            }
            Command::LoadAssignment => {
                let mut dlg =
                    rfd::FileDialog::new().add_filter("Assignment", &["txt", "assignment"]);
                if let Some(dir) = self.prefs.assignment_file.as_ref().and_then(|p| p.parent()) {
                    dlg = dlg.set_directory(dir);
                }
                if let Some(p) = dlg.pick_file() {
                    self.load_assignment(&p);
                }
            }
            Command::LoadCodes => {
                if let Some(p) = rfd::FileDialog::new()
                    .add_filter("Code replacements", &["txt"])
                    .pick_file()
                {
                    self.load_codes(&p);
                }
            }
            Command::ApplyStationery => self.apply_stationery(),
            Command::InsertCaptionTemplate => self.insert_caption_template(),
            Command::SaveCaption => {
                self.save_caption(true);
            }
            Command::SaveCaptionAndNext => {
                self.save_caption(true);
                self.move_focus(1, Mods::default());
                self.sync_caption_draft();
            }
            Command::RevertCaption => {
                if let Some(d) = self.caption_draft.as_mut() {
                    d.fields = d.original.clone();
                }
            }
            Command::SetRating(r) => self.mark(if r < 0 { "reject" } else { "rate" }, |m| {
                m.rating = Rating::clamp(r)
            }),
            Command::SetClass(c) => self.mark("class", |m| m.color_class = ColorClass::clamp(c)),
            Command::ToggleTag => {
                // A batch toggle sets every target to the opposite of the focused item.
                let to = !self
                    .marks
                    .get(self.cursor)
                    .map(|m| m.tagged)
                    .unwrap_or(false);
                self.mark(if to { "tag" } else { "untag" }, |m| m.tagged = to)
            }
            Command::Escape => {
                if self.filter_popup_open {
                    self.filter_popup_open = false;
                } else if self.workspace == Workspace::Preview {
                    self.workspace = Workspace::Sheet;
                    self.scroll_to_cursor = true;
                } else if self.selection.len() > 1 {
                    self.selection.only(self.cursor);
                }
            }
            Command::Activate => {
                if self.workspace == Workspace::Sheet && !self.items.is_empty() {
                    self.workspace = Workspace::Preview;
                } else if self.workspace == Workspace::Preview {
                    self.workspace = Workspace::Sheet;
                    self.scroll_to_cursor = true;
                }
            }
        }
        self.sync_menu();
        ctx.request_repaint();
    }

    /// Keyboard → commands. Command-modified keys are the native menu's job
    /// when a menu exists, except ⌘-arrows (move focus, keep selection).
    fn keyboard_commands(&self, ctx: &egui::Context) -> Vec<Command> {
        let mut out = Vec::new();
        if ctx.wants_keyboard_input() {
            return out;
        }
        ctx.input(|i| {
            let cmd = i.modifiers.command;
            let shift = i.modifiers.shift;
            let has_menu = self.platform.has_menu();
            let cols = self.cols.max(1) as isize;
            let in_preview = self.workspace == Workspace::Preview;
            let pressed = |k: Key| i.key_pressed(k);
            let mv = |delta: isize| Command::MoveFocus {
                delta,
                shift,
                command: cmd,
            };
            if pressed(Key::ArrowLeft) {
                out.push(mv(-1));
            }
            if pressed(Key::ArrowRight) {
                out.push(mv(1));
            }
            if pressed(Key::ArrowUp) {
                out.push(mv(if in_preview { -1 } else { -cols }));
            }
            if pressed(Key::ArrowDown) {
                out.push(mv(if in_preview { 1 } else { cols }));
            }
            if cmd {
                if has_menu {
                    return;
                }
                if pressed(Key::O) {
                    out.push(Command::PickFolder);
                }
                if pressed(Key::A) {
                    out.push(if shift {
                        Command::SelectNone
                    } else {
                        Command::SelectAll
                    });
                }
                if pressed(Key::Z) {
                    out.push(if shift { Command::Redo } else { Command::Undo });
                }
                if pressed(Key::Equals) || pressed(Key::Plus) {
                    out.push(Command::ZoomIn);
                }
                if pressed(Key::Minus) {
                    out.push(Command::ZoomOut);
                }
                for (n, k) in [
                    Key::Num0,
                    Key::Num1,
                    Key::Num2,
                    Key::Num3,
                    Key::Num4,
                    Key::Num5,
                ]
                .iter()
                .enumerate()
                {
                    if pressed(*k) {
                        out.push(Command::SetRating(n as i8));
                    }
                }
                return;
            }
            if pressed(Key::PageUp) {
                out.push(Command::MovePage(-1));
            }
            if pressed(Key::PageDown) {
                out.push(Command::MovePage(1));
            }
            if pressed(Key::Home) {
                out.push(Command::Home);
            }
            if pressed(Key::End) {
                out.push(Command::End);
            }
            if pressed(Key::Enter) {
                out.push(Command::Activate);
            }
            if pressed(Key::Space) {
                out.push(if in_preview {
                    Command::Move(1)
                } else {
                    Command::Activate
                });
            }
            if pressed(Key::Escape) {
                out.push(Command::Escape);
            }
            if !has_menu {
                for (n, k) in [
                    Key::Num0,
                    Key::Num1,
                    Key::Num2,
                    Key::Num3,
                    Key::Num4,
                    Key::Num5,
                    Key::Num6,
                    Key::Num7,
                    Key::Num8,
                ]
                .iter()
                .enumerate()
                {
                    if pressed(*k) {
                        out.push(Command::SetClass(n as u8));
                    }
                }
                if pressed(Key::T) {
                    out.push(Command::ToggleTag);
                }
                if pressed(Key::X) && !shift {
                    out.push(Command::SetRating(-1));
                }
                if pressed(Key::F) && shift {
                    out.push(Command::ToggleFilmstrip);
                }
            }
        });
        out
    }

    fn maybe_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.launch.screenshot.clone() else {
            return;
        };
        self.frames += 1;
        if self.frames == self.launch.screenshot_frame && !self.screenshot_requested {
            self.screenshot_requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let shot = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = shot {
            let [w, h] = img.size;
            if let Some(buf) = image::RgbaImage::from_raw(w as u32, h as u32, img.as_raw().to_vec())
            {
                match buf.save(&path) {
                    Ok(()) => log::info!("screenshot saved to {}", path.display()),
                    Err(e) => log::error!("screenshot failed: {e}"),
                }
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.request_repaint();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt_ms = ctx.input(|i| i.stable_dt) * 1000.0;
        if dt_ms > 0.0 && dt_ms < 1000.0 {
            self.stats.frame_ms_ema = if self.stats.frame_ms_ema == 0.0 {
                dt_ms
            } else {
                self.stats.frame_ms_ema * 0.9 + dt_ms * 0.1
            };
            self.stats.frame_ms_max = self.stats.frame_ms_max.max(dt_ms);
        }
        let t = Theme::for_ctx(ctx);
        t.apply(ctx, self.reduce_motion);
        self.platform.on_first_frame();

        self.drain_results(ctx);
        for c in self.platform.drain_menu() {
            self.dispatch(ctx, c);
        }
        for c in self.keyboard_commands(ctx) {
            self.dispatch(ctx, c);
        }

        if self.workspace == Workspace::Caption {
            self.sync_caption_draft();
        }
        ui::toolbar::show(self, ctx, &t);
        ui::status::show(self, ctx, &t);
        ui::sidebar::show(self, ctx, &t);
        if self.workspace == Workspace::Caption {
            ui::caption::show_fields(self, ctx, &t);
        } else {
            ui::inspector::show(self, ctx, &t);
        }
        ui::filmstrip::show(self, ctx, &t);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(t.photo))
            .show(ctx, |ui| {
                if self.items.is_empty() {
                    ui.centered_and_justified(|ui| {
                        let msg = if self.folder.is_some() {
                            "No photos in this folder"
                        } else {
                            "Open a folder with ⌘O, or pick one in the sidebar"
                        };
                        ui.label(egui::RichText::new(msg).size(15.0).color(t.muted));
                    });
                    return;
                }
                if self.view.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new("No photos match the filter · ⌥⌘F clears it")
                                .size(15.0)
                                .color(t.muted),
                        );
                    });
                    return;
                }
                match self.workspace {
                    Workspace::Sheet => ui::sheet::show(self, ui, &t),
                    Workspace::Preview => ui::preview::show(self, ui, &t),
                    Workspace::Caption => ui::caption::show_photo(self, ui, &t),
                    Workspace::Ingest => {}
                }
            });
        let r = ctx.content_rect();
        ctx.layer_painter(egui::LayerId::background()).line_segment(
            [
                egui::pos2(r.left(), r.top() + ui::tokens::TOOLBAR_H),
                egui::pos2(r.right(), r.top() + ui::tokens::TOOLBAR_H),
            ],
            egui::Stroke::new(1.0, t.line),
        );
        ctx.layer_painter(egui::LayerId::background()).line_segment(
            [
                egui::pos2(r.left(), r.bottom() - ui::tokens::STATUS_H),
                egui::pos2(r.right(), r.bottom() - ui::tokens::STATUS_H),
            ],
            egui::Stroke::new(1.0, t.line),
        );

        if self.prefs_dirty && self.prefs_saved_at.elapsed().as_secs_f32() > 1.0 {
            self.prefs.save();
            self.prefs_dirty = false;
            self.prefs_saved_at = Instant::now();
        }
        self.maybe_screenshot(ctx);
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.prefs.save();
    }
}

fn upload(ctx: &egui::Context, name: &str, rgb: &Rgb) -> TextureHandle {
    let img = ColorImage::from_rgb([rgb.width as usize, rgb.height as usize], &rgb.data);
    ctx.load_texture(name, img, TextureOptions::LINEAR)
}
