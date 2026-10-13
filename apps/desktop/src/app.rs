use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::Instant;

use egui::{
    Align2, Color32, ColorImage, CornerRadius, FontId, Key, Pos2, Rect, Sense, Stroke, StrokeKind,
    TextureHandle, TextureOptions, Vec2,
};
use rayon::prelude::*;

use photo_core::{ColorClass, ColorClassScheme, Marks, PhotoItem, Rating, ScanOptions};
use photo_metadata::xmp::{self, XmpMarks};
use photo_metadata::ExifSummary;
use photo_preview::{Rgb, ThumbCache};

use crate::worker::{Job, JobKind, Pool};

const THUMB_PX: u32 = 320;
const GAP: f32 = 8.0;
const LABEL_H: f32 = 18.0;
const PREFETCH_ROWS: usize = 3;
const KEEP_ROWS: usize = 12;
const PREVIEW_LRU: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Sheet,
    Preview,
}

#[derive(Default)]
struct Stats {
    thumbs_done: usize,
    cache_hits: usize,
    decode_ms_total: f32,
    scan_ms: f32,
    /// Exponential moving average of frame time in ms (UI thread).
    frame_ms_ema: f32,
    frame_ms_max: f32,
    textures_uploaded: usize,
}

pub struct App {
    folder: Option<PathBuf>,
    items: Vec<PhotoItem>,
    marks: Vec<Marks>,
    exif: Vec<Option<ExifSummary>>,
    thumbs: Vec<Option<TextureHandle>>,
    thumb_err: Vec<Option<String>>,
    previews: HashMap<usize, TextureHandle>,
    preview_lru: VecDeque<usize>,
    pool: Pool,
    generation: u64,
    cursor: usize,
    mode: Mode,
    cell: f32,
    cols: usize,
    last_visible_rows: Range<usize>,
    scroll_to_cursor: bool,
    scheme: ColorClassScheme,
    stats: Stats,
    status: String,
    opened_at: Option<Instant>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, folder: Option<PathBuf>) -> Self {
        let cache = directories::ProjectDirs::from("", "", "photo-workflow")
            .and_then(|d| ThumbCache::new(d.cache_dir().join("thumbs")).ok());
        let threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let threads = (threads.saturating_sub(1)).clamp(2, 12);
        let pool = Pool::new(cc.egui_ctx.clone(), cache, threads);
        let mut app = App {
            folder: None,
            items: Vec::new(),
            marks: Vec::new(),
            exif: Vec::new(),
            thumbs: Vec::new(),
            thumb_err: Vec::new(),
            previews: HashMap::new(),
            preview_lru: VecDeque::new(),
            pool,
            generation: 0,
            cursor: 0,
            mode: Mode::Sheet,
            cell: 180.0,
            cols: 1,
            last_visible_rows: 0..0,
            scroll_to_cursor: false,
            scheme: ColorClassScheme::default(),
            stats: Stats::default(),
            status: "Open a folder (⌘O) or pass one on the command line.".into(),
            opened_at: None,
        };
        if let Some(f) = folder {
            app.open_folder(&f);
        }
        app
    }

    fn open_folder(&mut self, dir: &Path) {
        let t0 = Instant::now();
        match photo_core::scan(dir, &ScanOptions::default()) {
            Ok(items) => {
                self.pool.clear_pending();
                self.generation += 1;
                let scheme = self.scheme.clone();
                let marks: Vec<Marks> = items
                    .par_iter()
                    .map(|it| {
                        xmp::read_sidecar(&it.sidecar_path())
                            .ok()
                            .flatten()
                            .map(|m| m.to_marks(&scheme))
                            .unwrap_or_default()
                    })
                    .collect();
                let n = items.len();
                self.items = items;
                self.marks = marks;
                self.exif = vec![None; n];
                self.thumbs = vec![None; n];
                self.thumb_err = vec![None; n];
                self.previews.clear();
                self.preview_lru.clear();
                self.cursor = 0;
                self.mode = Mode::Sheet;
                self.folder = Some(dir.to_path_buf());
                self.stats = Stats {
                    scan_ms: t0.elapsed().as_secs_f32() * 1000.0,
                    ..Default::default()
                };
                self.opened_at = Some(Instant::now());
                self.status = format!(
                    "{} photos in {} ({:.0} ms scan)",
                    n,
                    dir.display(),
                    self.stats.scan_ms
                );
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
                            self.previews.insert(done.idx, tex);
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

    fn request_thumb(&self, idx: usize, prio: u32) {
        if self.thumbs[idx].is_some() || self.thumb_err[idx].is_some() {
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

    fn request_preview(&self, idx: usize, long_side: u32, prio: u32) {
        if idx >= self.items.len() || self.previews.contains_key(&idx) {
            return;
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

    /// Request thumbnails for visible rows (priority by distance), prefetch
    /// a few rows around them, and drop textures far away so GPU memory stays
    /// bounded no matter how big the folder is.
    fn schedule_thumbs(&mut self, visible: Range<usize>) {
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
                if self.thumbs[idx].is_some() {
                    self.thumbs[idx] = None;
                    self.stats.thumbs_done = self.stats.thumbs_done.saturating_sub(1);
                }
            }
        }
    }

    // ----- marks -------------------------------------------------------------

    fn persist(&mut self, idx: usize) {
        let it = &self.items[idx];
        let xm = XmpMarks::from_marks(&self.marks[idx], &self.scheme);
        match xmp::write_marks(&it.sidecar_path(), &xm) {
            Ok(()) => {
                let m = self.marks[idx];
                self.status = format!(
                    "{}: {} · {} · {}",
                    it.display_name(),
                    rating_text(m.rating),
                    self.scheme.name(m.color_class).unwrap_or("no class"),
                    if m.tagged { "tagged" } else { "untagged" }
                );
            }
            Err(e) => self.status = format!("Could not write {}: {e}", it.sidecar_path().display()),
        }
    }

    fn set_rating(&mut self, r: i8) {
        if self.items.is_empty() {
            return;
        }
        let i = self.cursor;
        self.marks[i].rating = Rating::clamp(r);
        self.persist(i);
    }

    fn set_class(&mut self, c: u8) {
        if self.items.is_empty() {
            return;
        }
        let i = self.cursor;
        self.marks[i].color_class = ColorClass::clamp(c);
        self.persist(i);
    }

    fn toggle_tag(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = self.cursor;
        self.marks[i].tagged = !self.marks[i].tagged;
        self.persist(i);
    }

    // ----- navigation --------------------------------------------------------

    fn move_cursor(&mut self, delta: isize) {
        if self.items.is_empty() {
            return;
        }
        let n = self.items.len() as isize;
        let next = (self.cursor as isize + delta).clamp(0, n - 1);
        self.cursor = next as usize;
        self.scroll_to_cursor = true;
    }

    fn row_count(&self) -> usize {
        if self.cols == 0 {
            0
        } else {
            self.items.len().div_ceil(self.cols)
        }
    }

    fn row_height(&self) -> f32 {
        self.cell + LABEL_H + GAP
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        let (cmd, shift) = ctx.input(|i| (i.modifiers.command, i.modifiers.shift));
        let pressed = |k: Key| ctx.input(|i| i.key_pressed(k));

        if cmd && pressed(Key::O) {
            self.pick_folder();
            return;
        }
        if self.items.is_empty() {
            return;
        }
        let cols = self.cols.max(1) as isize;
        let page = (self.last_visible_rows.len().max(1) as isize) * cols;

        match self.mode {
            Mode::Sheet => {
                if pressed(Key::ArrowLeft) {
                    self.move_cursor(-1);
                }
                if pressed(Key::ArrowRight) {
                    self.move_cursor(1);
                }
                if pressed(Key::ArrowUp) {
                    self.move_cursor(-cols);
                }
                if pressed(Key::ArrowDown) {
                    self.move_cursor(cols);
                }
                if pressed(Key::PageUp) {
                    self.move_cursor(-page);
                }
                if pressed(Key::PageDown) {
                    self.move_cursor(page);
                }
                if pressed(Key::Enter) || pressed(Key::Space) {
                    self.mode = Mode::Preview;
                }
                if !cmd && pressed(Key::Plus) || pressed(Key::Equals) {
                    self.cell = (self.cell * 1.2).min(600.0);
                    self.scroll_to_cursor = true;
                }
                if !cmd && pressed(Key::Minus) {
                    self.cell = (self.cell / 1.2).max(80.0);
                    self.scroll_to_cursor = true;
                }
            }
            Mode::Preview => {
                if pressed(Key::ArrowLeft) || pressed(Key::ArrowUp) || pressed(Key::PageUp) {
                    self.move_cursor(-1);
                }
                if pressed(Key::ArrowRight)
                    || pressed(Key::ArrowDown)
                    || pressed(Key::PageDown)
                    || pressed(Key::Space)
                {
                    self.move_cursor(1);
                }
                if pressed(Key::Escape) || pressed(Key::Enter) {
                    self.mode = Mode::Sheet;
                    self.scroll_to_cursor = true;
                }
            }
        }
        if pressed(Key::Home) {
            self.cursor = 0;
            self.scroll_to_cursor = true;
        }
        if pressed(Key::End) {
            self.cursor = self.items.len() - 1;
            self.scroll_to_cursor = true;
        }

        // Photo Mechanic conventions: 1–8 colour class, 0 clears it,
        // ⌘0–⌘5 star rating, T toggles tag, X rejects.
        let digits = [
            Key::Num0,
            Key::Num1,
            Key::Num2,
            Key::Num3,
            Key::Num4,
            Key::Num5,
            Key::Num6,
            Key::Num7,
            Key::Num8,
        ];
        for (n, k) in digits.iter().enumerate() {
            if pressed(*k) {
                if cmd {
                    if n <= 5 {
                        self.set_rating(n as i8);
                    }
                } else {
                    self.set_class(n as u8);
                }
            }
        }
        if pressed(Key::T) && !cmd {
            self.toggle_tag();
        }
        if pressed(Key::X) && !cmd && !shift {
            self.set_rating(-1);
        }
    }

    // ----- drawing -----------------------------------------------------------

    fn draw_sheet(&mut self, ui: &mut egui::Ui) {
        let avail_w = ui.available_width();
        let avail_h = ui.available_height();
        self.cols = (((avail_w - GAP) / (self.cell + GAP)).floor() as usize).max(1);
        let rows = self.row_count();
        let row_h = self.row_height();

        let mut area = egui::ScrollArea::vertical().auto_shrink([false, false]);
        if self.scroll_to_cursor && self.cols > 0 {
            let row = self.cursor / self.cols;
            let vis = &self.last_visible_rows;
            // Only jump when the cursor row is outside the rows seen last frame.
            if row < vis.start || row + 1 > vis.end.saturating_sub(1) {
                let target = if row < vis.start {
                    row as f32 * row_h
                } else {
                    ((row + 1) as f32 * row_h - avail_h + GAP).max(0.0)
                };
                area = area.vertical_scroll_offset(target);
            }
            self.scroll_to_cursor = false;
        }

        let mut clicked: Option<usize> = None;
        let mut opened: Option<usize> = None;
        let mut visible = 0..0;
        let cols = self.cols;
        let cell = self.cell;
        let n = self.items.len();

        area.show_rows(ui, row_h, rows, |ui, range| {
            visible = range.clone();
            for row in range {
                ui.horizontal(|ui| {
                    ui.add_space(GAP);
                    for col in 0..cols {
                        let idx = row * cols + col;
                        if idx >= n {
                            break;
                        }
                        let (rect, resp) =
                            ui.allocate_exact_size(Vec2::new(cell, cell + LABEL_H), Sense::click());
                        self.draw_cell(ui, rect, idx);
                        if resp.double_clicked() {
                            opened = Some(idx);
                        } else if resp.clicked() {
                            clicked = Some(idx);
                        }
                        ui.add_space(GAP);
                    }
                });
                ui.add_space(GAP);
            }
        });

        self.last_visible_rows = visible.clone();
        if let Some(i) = clicked {
            self.cursor = i;
        }
        if let Some(i) = opened {
            self.cursor = i;
            self.mode = Mode::Preview;
        }
        self.schedule_thumbs(visible);
    }

    fn draw_cell(&self, ui: &egui::Ui, rect: Rect, idx: usize) {
        let p = ui.painter();
        let selected = idx == self.cursor;
        let img_rect = Rect::from_min_size(rect.min, Vec2::splat(self.cell));
        p.rect_filled(
            img_rect,
            CornerRadius::same(3),
            Color32::from_gray(if selected { 58 } else { 34 }),
        );

        if let Some(tex) = &self.thumbs[idx] {
            let size = tex.size_vec2();
            let s = (self.cell / size.x).min(self.cell / size.y);
            let fitted = Rect::from_center_size(img_rect.center(), size * s);
            p.image(
                tex.id(),
                fitted,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else if let Some(err) = &self.thumb_err[idx] {
            p.text(
                img_rect.center(),
                Align2::CENTER_CENTER,
                "no preview",
                FontId::proportional(12.0),
                Color32::from_gray(140),
            );
            let _ = err;
        } else {
            p.text(
                img_rect.center(),
                Align2::CENTER_CENTER,
                "…",
                FontId::proportional(18.0),
                Color32::from_gray(110),
            );
        }

        let m = &self.marks[idx];
        if let Some(c) = self.scheme.color(m.color_class) {
            let bar = Rect::from_min_max(
                Pos2::new(img_rect.left(), img_rect.bottom() - 6.0),
                img_rect.right_bottom(),
            );
            p.rect_filled(bar, CornerRadius::ZERO, Color32::from_rgb(c[0], c[1], c[2]));
        }
        if selected {
            p.rect_stroke(
                img_rect.expand(1.5),
                CornerRadius::same(4),
                Stroke::new(3.0, Color32::from_rgb(255, 200, 40)),
                StrokeKind::Outside,
            );
        }
        if m.tagged {
            let r = Rect::from_min_size(
                Pos2::new(img_rect.right() - 20.0, img_rect.top() + 4.0),
                Vec2::new(16.0, 16.0),
            );
            p.rect_filled(r, CornerRadius::same(3), Color32::from_rgb(40, 140, 255));
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                "T",
                FontId::proportional(11.0),
                Color32::WHITE,
            );
        }
        if self.items[idx].is_pair() {
            p.text(
                Pos2::new(img_rect.left() + 4.0, img_rect.top() + 3.0),
                Align2::LEFT_TOP,
                "R+J",
                FontId::proportional(10.0),
                Color32::from_gray(200),
            );
        }

        let name = self.items[idx].display_name();
        let label_y = img_rect.bottom() + 3.0;
        p.text(
            Pos2::new(rect.left() + 2.0, label_y),
            Align2::LEFT_TOP,
            name,
            FontId::proportional(11.0),
            Color32::from_gray(210),
        );
        let stars = rating_text(m.rating);
        if !stars.is_empty() {
            p.text(
                Pos2::new(rect.right() - 2.0, label_y),
                Align2::RIGHT_TOP,
                stars,
                FontId::proportional(11.0),
                Color32::from_rgb(255, 200, 40),
            );
        }
    }

    fn draw_preview(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let ppp = ui.ctx().pixels_per_point();
        let long_side = ((rect.width().max(rect.height())) * ppp).ceil() as u32;
        self.request_preview(self.cursor, long_side, 0);
        self.request_preview(self.cursor + 1, long_side, 1);
        if self.cursor > 0 {
            self.request_preview(self.cursor - 1, long_side, 2);
        }
        let idx = self.cursor;
        let p = ui.painter();
        p.rect_filled(rect, CornerRadius::ZERO, Color32::from_gray(18));
        let tex = self.previews.get(&idx).or(self.thumbs[idx].as_ref());
        if let Some(tex) = tex {
            let size = tex.size_vec2();
            let s = (rect.width() / size.x)
                .min(rect.height() / size.y)
                .min(ppp.max(1.0));
            let fitted = Rect::from_center_size(rect.center(), size * s);
            p.image(
                tex.id(),
                fitted,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            if !self.previews.contains_key(&idx) {
                p.text(
                    rect.center_bottom() - Vec2::new(0.0, 12.0),
                    Align2::CENTER_BOTTOM,
                    "loading full preview…",
                    FontId::proportional(12.0),
                    Color32::from_gray(160),
                );
            }
        } else {
            p.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "loading…",
                FontId::proportional(16.0),
                Color32::from_gray(140),
            );
        }
        let m = self.marks[idx];
        let overlay = format!(
            "{}  {}/{}   {}   {}",
            self.items[idx].display_name(),
            idx + 1,
            self.items.len(),
            rating_text(m.rating),
            self.scheme.name(m.color_class).unwrap_or("")
        );
        p.text(
            rect.left_top() + Vec2::new(12.0, 10.0),
            Align2::LEFT_TOP,
            overlay,
            FontId::proportional(13.0),
            Color32::from_gray(230),
        );
        if let Some(c) = self.scheme.color(m.color_class) {
            let bar = Rect::from_min_size(rect.left_top(), Vec2::new(6.0, rect.height()));
            p.rect_filled(bar, CornerRadius::ZERO, Color32::from_rgb(c[0], c[1], c[2]));
        }
        // Keep the sheet's thumbnails warm around the cursor for when we return.
        if self.cols > 0 {
            let row = idx / self.cols;
            let rows = self.row_count();
            self.schedule_thumbs(row.saturating_sub(1)..(row + 2).min(rows));
        }
    }

    fn status_line(&self) -> String {
        let Some(it) = self.items.get(self.cursor) else {
            return self.status.clone();
        };
        let m = self.marks[self.cursor];
        let mut parts = vec![
            format!("{}/{}", self.cursor + 1, self.items.len()),
            it.display_name(),
        ];
        if let Some(e) = &self.exif[self.cursor] {
            if let Some(d) = &e.date_time_original {
                parts.push(d.clone());
            }
            let cam = e.camera_line();
            if !cam.is_empty() {
                parts.push(cam);
            }
            let ex = e.exposure_line();
            if !ex.is_empty() {
                parts.push(ex);
            }
        }
        let stars = rating_text(m.rating);
        if !stars.is_empty() {
            parts.push(stars);
        }
        if let Some(n) = self.scheme.name(m.color_class) {
            parts.push(n.to_string());
        }
        if m.tagged {
            parts.push("tagged".into());
        }
        parts.join("   ")
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
        self.drain_results(ctx);
        self.handle_keys(ctx);

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open folder…").clicked() {
                    self.pick_folder();
                }
                if let Some(f) = &self.folder {
                    ui.label(f.display().to_string());
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.cell, 80.0..=600.0)
                            .show_value(false)
                            .text("size"),
                    );
                    let pending = self.pool.pending();
                    if pending > 0 {
                        ui.label(format!("decoding {pending}…"));
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(self.status_line());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let avg = if self.stats.thumbs_done > 0 {
                        self.stats.decode_ms_total / self.stats.thumbs_done as f32
                    } else {
                        0.0
                    };
                    ui.label(format!(
                        "thumbs {} ({} cached, {avg:.0} ms avg) · uploads {} · frame {:.1} ms (max {:.0})",
                        self.stats.thumbs_done, self.stats.cache_hits, self.stats.textures_uploaded,
                        self.stats.frame_ms_ema, self.stats.frame_ms_max
                    ));
                });
            });
            ui.horizontal(|ui| {
                ui.small(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small("←→↑↓ move · ⏎/space preview · esc back · 1–8 class · 0 clear · ⌘0–5 stars · X reject · T tag · +/- size");
                });
            });
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::from_gray(24)))
            .show(ctx, |ui| {
                if self.items.is_empty() {
                    ui.centered_and_justified(|ui| ui.label(&self.status));
                    return;
                }
                match self.mode {
                    Mode::Sheet => self.draw_sheet(ui),
                    Mode::Preview => self.draw_preview(ui),
                }
            });
    }
}

fn upload(ctx: &egui::Context, name: &str, rgb: &Rgb) -> TextureHandle {
    let img = ColorImage::from_rgb([rgb.width as usize, rgb.height as usize], &rgb.data);
    ctx.load_texture(name, img, TextureOptions::LINEAR)
}

fn rating_text(r: Rating) -> String {
    if r.is_rejected() {
        "✕".into()
    } else {
        "★".repeat(r.stars() as usize)
    }
}
