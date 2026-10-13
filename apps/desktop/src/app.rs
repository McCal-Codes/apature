//! Application state and the single command dispatcher. Drawing lives in
//! `ui/`; native integration in `platform/`; decoding in `worker.rs`.

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

use crate::commands::{Command, Workspace};
use crate::platform::Platform;
use crate::prefs::Prefs;
use crate::ui::{self, tokens::Theme};
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
    /// Decoded previews with the long side they were decoded at.
    pub previews: HashMap<usize, (TextureHandle, u32)>,
    preview_lru: VecDeque<usize>,
    pub pool: Pool,
    generation: u64,
    pub cursor: usize,
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
        platform.sync_toggles(
            prefs.sidebar_visible,
            prefs.inspector_visible,
            prefs.filmstrip_visible,
        );
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
            platform,
            reduce_motion: Platform::reduce_motion(),
            launch: launch.clone(),
            frames: 0,
            screenshot_requested: false,
        };
        app.refresh_volumes();
        let first = launch
            .folder
            .clone()
            .or_else(|| app.prefs.recent.first().cloned())
            .filter(|f| f.is_dir());
        if let Some(f) = first {
            app.open_folder(&f);
        }
        app
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
                let read: Vec<(Marks, Vec<String>)> = items
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
                        (m, c)
                    })
                    .collect();
                let n = items.len();
                self.items = items;
                self.marks = read.iter().map(|(m, _)| *m).collect();
                self.conflicts = read.into_iter().map(|(_, c)| c).collect();
                self.exif = vec![None; n];
                self.thumbs = vec![None; n];
                self.thumb_err = vec![None; n];
                self.previews.clear();
                self.preview_lru.clear();
                self.cursor = 0;
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

    // ----- background results ------------------------------------------------

    fn drain_results(&mut self, ctx: &egui::Context) {
        let mut n = 0;
        while let Some(done) = self.pool.try_recv() {
            if done.generation != self.generation || done.idx >= self.items.len() {
                continue;
            }
            if let Some(e) = done.exif {
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
                            log::debug!(
                                "preview {} ready at {} px in {:.0} ms",
                                done.idx,
                                rgb.long_side(),
                                done.millis
                            );
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
            let Some(pos) = self.preview_lru.iter().position(|&i| !keep(i)) else {
                break;
            };
            let idx = self.preview_lru.remove(pos).unwrap();
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
                // Either big enough, or already the largest embedded preview we will get.
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

    pub fn schedule_thumbs(&mut self, visible: Range<usize>) {
        if self.items.is_empty() || self.cols == 0 {
            return;
        }
        let rows = self.row_count();
        let first = visible.start.saturating_sub(PREFETCH_ROWS);
        let last = (visible.end + PREFETCH_ROWS).min(rows);
        for row in first..last {
            let prio = if row < visible.start {
                10 + (visible.start - row) as u32
            } else if row >= visible.end {
                10 + (row + 1 - visible.end) as u32
            } else {
                (row - visible.start) as u32
            };
            for idx in row * self.cols..((row + 1) * self.cols).min(self.items.len()) {
                self.request_thumb(idx, prio);
            }
        }
        let keep_first = visible.start.saturating_sub(KEEP_ROWS);
        let keep_last = (visible.end + KEEP_ROWS).min(rows);
        for row in (0..keep_first).chain(keep_last..rows) {
            for idx in row * self.cols..((row + 1) * self.cols).min(self.items.len()) {
                if self.thumbs[idx].is_some() && idx != self.cursor {
                    self.thumbs[idx] = None;
                    self.stats.thumbs_done = self.stats.thumbs_done.saturating_sub(1);
                }
            }
        }
    }

    // ----- marks -------------------------------------------------------------

    fn persist(&mut self, idx: usize) {
        let it = &self.items[idx];
        match items::write(it, &self.marks[idx], &self.scheme) {
            Ok(written) => {
                self.conflicts[idx].clear();
                let m = self.marks[idx];
                self.status = format!(
                    "{}: {} · {} · {} → {} file{}",
                    it.display_name(),
                    ui::rating_text(m.rating),
                    self.scheme.name(m.color_class).unwrap_or("no class"),
                    if m.tagged { "tagged" } else { "untagged" },
                    written.len(),
                    if written.len() == 1 { "" } else { "s" }
                );
            }
            Err(e) => self.status = format!("Could not write marks: {e}"),
        }
    }

    // ----- navigation --------------------------------------------------------

    pub fn row_count(&self) -> usize {
        if self.cols == 0 {
            0
        } else {
            self.items.len().div_ceil(self.cols)
        }
    }

    fn move_cursor(&mut self, delta: isize) {
        if self.items.is_empty() {
            return;
        }
        let n = self.items.len() as isize;
        self.cursor = (self.cursor as isize + delta).clamp(0, n - 1) as usize;
        self.scroll_to_cursor = true;
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
        parts.push(format!("{} of {}", idx + 1, self.items.len()));
        parts.join(", ")
    }

    pub fn status_parts(&self) -> Vec<String> {
        let mut parts = Vec::new();
        match &self.folder {
            None => parts.push("No folder open · ⌘O".into()),
            Some(f) => {
                parts.push(format!(
                    "{} · {} photos",
                    f.file_name()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                    self.items.len()
                ));
            }
        }
        if let Some(it) = self.items.get(self.cursor) {
            parts.push(format!(
                "{}/{} {}",
                self.cursor + 1,
                self.items.len(),
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
            Command::Move(d) => self.move_cursor(d),
            Command::MovePage(d) => {
                let page = (self.last_visible_rows.len().max(1) * self.cols.max(1)) as isize;
                self.move_cursor(d * page);
            }
            Command::Home => {
                self.cursor = 0;
                self.scroll_to_cursor = true;
            }
            Command::End => {
                if !self.items.is_empty() {
                    self.cursor = self.items.len() - 1;
                    self.scroll_to_cursor = true;
                }
            }
            Command::SetRating(r) => {
                if !self.items.is_empty() {
                    let i = self.cursor;
                    self.marks[i].rating = Rating::clamp(r);
                    self.persist(i);
                }
            }
            Command::SetClass(c) => {
                if !self.items.is_empty() {
                    let i = self.cursor;
                    self.marks[i].color_class = ColorClass::clamp(c);
                    self.persist(i);
                }
            }
            Command::ToggleTag => {
                if !self.items.is_empty() {
                    let i = self.cursor;
                    self.marks[i].tagged = !self.marks[i].tagged;
                    self.persist(i);
                }
            }
            Command::Escape => {
                if self.workspace == Workspace::Preview {
                    self.workspace = Workspace::Sheet;
                    self.scroll_to_cursor = true;
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
        self.platform.sync_toggles(
            self.prefs.sidebar_visible,
            self.prefs.inspector_visible,
            self.prefs.filmstrip_visible,
        );
        ctx.request_repaint();
    }

    /// Keyboard → commands. Command-modified keys are the native menu's job
    /// when a menu exists; without one (other platforms) they are handled here.
    fn keyboard_commands(&self, ctx: &egui::Context) -> Vec<Command> {
        let mut out = Vec::new();
        let typing = ctx.memory(|m| m.focused().is_some()) && ctx.wants_keyboard_input();
        if typing {
            return out;
        }
        ctx.input(|i| {
            let cmd = i.modifiers.command;
            let shift = i.modifiers.shift;
            let has_menu = self.platform.has_menu();
            let cols = self.cols.max(1) as isize;
            let in_preview = self.workspace == Workspace::Preview;
            let pressed = |k: Key| i.key_pressed(k);
            if cmd {
                if has_menu {
                    return;
                }
                if pressed(Key::O) {
                    out.push(Command::PickFolder);
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
            if pressed(Key::ArrowLeft) {
                out.push(Command::Move(-1));
            }
            if pressed(Key::ArrowRight) {
                out.push(Command::Move(1));
            }
            if pressed(Key::ArrowUp) {
                out.push(Command::Move(if in_preview { -1 } else { -cols }));
            }
            if pressed(Key::ArrowDown) {
                out.push(Command::Move(if in_preview { 1 } else { cols }));
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

        ui::toolbar::show(self, ctx, &t);
        ui::status::show(self, ctx, &t);
        ui::sidebar::show(self, ctx, &t);
        ui::inspector::show(self, ctx, &t);
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
                match self.workspace {
                    Workspace::Sheet => ui::sheet::show(self, ui, &t),
                    Workspace::Preview => ui::preview::show(self, ui, &t),
                    Workspace::Caption | Workspace::Ingest => {}
                }
            });
        // Toolbar underline, drawn last so it sits above panel fills.
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
