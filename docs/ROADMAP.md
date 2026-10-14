# Roadmap

## 0.1 — fast browser (this repo today)
- [x] folder scan with RAW+JPEG pairing
- [x] embedded-preview thumbnails, DCT-scaled decode, disk cache
- [x] viewport-prioritised background decoding, bounded GPU memory
- [x] keyboard culling: stars, colour class, tag, reject
- [x] XMP sidecars written in place, PM/Lightroom compatible
- [x] full-screen preview with prefetch of neighbours
- [x] `apature-cli` CLI and benchmarks
- [x] benchmarks recorded (docs/benchmarks/2026-10-13-m1pro.md); a real 1 000-NEF assignment is still outstanding
- [ ] multi-select and shift-range marking
- [ ] filter the sheet by rating / class / tag
- [ ] HEIC and CR3 previews

## 0.2 — ingest and metadata
- [ ] IPTC stationery: headline, caption, byline, credit, copyright, location, keywords
- [ ] code replacements and `{variables}` compatible with the existing Butler Eagle files
- [ ] caption editor with the "type subject, append saved tail" flow
- [ ] ingest UI on top of `apature-ingest`: card detection, two destinations, rename template, report
- [ ] write IPTC into JPEG exports (sidecar stays the source for RAW)

## Requested 2026-10-14 (owner)
- [x] Drag and drop a folder or file onto the window to open it (2026-10-14)
- [x] **Tabs** (2026-10-14): in-app tab strip, per-tab view/selection/filter/undo, ⌘T/⌘W/⌘⇧]/⌘⇧[, restored at launch.
- [x] **Settings window** (2026-10-14): appearance, colour classes, metadata files + delimiter, ingest defaults, cache, shortcut table (profiles still to come).

## Requested 2026-10-14 (owner): clean animations — implemented 2026-10-14 (ui/motion.rs); keep the exclusions below as the rule
Restrained motion that makes state changes legible, never slows culling,
and switches off under Reduce Motion. Candidates, in priority order:
- Sidebar / inspector / filmstrip collapse and reveal: a 160 ms width ease
  (egui already animates panel width; tune the curve and duration).
- Tab switch: a short cross-fade of the sheet (no slide) so the eye keeps its
  place; the tab highlight moves, nothing else does.
- Marks: the class bar and stars snap instantly, then a 120 ms fade of the
  "saved" status; no bounce.
- Preview open/close: cross-fade from the cell's thumbnail to the preview,
  anchored at the cell so the image appears to grow in place.
- Filter chips: fade in; the grid re-flows without animation (reflow
  animation hides what changed).
- Drop overlay: 120 ms fade; ingest progress bar eases between values.
Not: parallax, springs on photos, animated selection rings, anything on
every keystroke. Record `prefers reduced motion` in the design tokens doc.

## Idea to research (owner, 2026-10-14): auto-ingester from the camera
Frames should land on a drive already in the right folder, named and
padded, before anyone sits down to caption. Routes to evaluate:
- The Z6 II's built-in Wi-Fi/FTP upload (Nikon's "Connect to FTP server")
  into a drop folder that an always-on Apature ingester watches and runs the
  plan/run/pad pipeline on (same engine as the Ingest workspace, headless).
- A tethered/card-reader "ingest box" (Mac mini or a small Linux board)
  running `apature-cli ingest` on card insert, with the assignment file
  selecting the folder template and pad.
- Cloud: after the local verified copy, a second destination that is a
  synced folder (Drive/Dropbox) so the desk gets frames without captions.
Keep captioning separate; this is about getting bytes somewhere safe and
named. Needs: folder watching (`notify`), a daemon mode for the CLI, and
the erase/unmount gates designed first.

## 0.3 — professional workflow
- [ ] export presets (long side, quality, sRGB, keep metadata, naming)
- [ ] saved assignment presets (the `generator/` text files become first-class)
- [ ] folder watching for live ingest
- [ ] Windows and Linux builds

## 0.4 — catalogs
- [ ] SQLite index across drives, metadata search, collections
