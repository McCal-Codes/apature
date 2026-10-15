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
- [x] multi-select and shift-range marking (done; ticked 2026-10-15)
- [x] filter the sheet by rating / class / tag (done; ticked 2026-10-15)
- [ ] HEIC and CR3 previews

## 0.2 — ingest and metadata
- [x] IPTC stationery: headline, caption, byline, credit, copyright, location, keywords (done; ticked 2026-10-15)
- [x] code replacements and `{variables}` compatible with the existing Butler Eagle files (done; ticked 2026-10-15)
- [x] caption editor with the "type subject, append saved tail" flow (done; ticked 2026-10-15)
- [x] ingest UI on top of `apature-ingest`: card detection, two destinations, rename template, report (done; ticked 2026-10-15)
- [x] write IPTC into JPEG exports (sidecar stays the source for RAW) (done; ticked 2026-10-15)

## Requested 2026-10-15 (owner): Photo Mechanic parity, in order of value
After the first real-card ingest (Ryan's birthday, 134 NEF, verified) the owner
confirmed this order. Readability, no overlapping UI, polish and accessibility
settings apply to every item.
1. [x] Persistent `{seqn}` counter per headline (2026-10-15): continues across
   ingests, "Start over at 1" on the sheet, Settings › Ingest switch.
2. [x] Preserve the card's folder structure (option); copy WAV voice memos
   (2026-10-15): XMP sidecars on the card travel too and follow a `-N` suffix.
3. [ ] Open the contact sheet while the ingest is still running (Ingest is its
   own window since 2026-10-15, so the main window is free for it).
4. [~] Lossless rotate (orientation tag) and a crop stored in XMP, applied on export.
   2026-10-15: rotate done (Image › Rotate Left/Right, ⌘[ ⌘], `tiff:Orientation`
   in the sidecar, sheet and preview follow it). Crop stored as `crs:Crop*`
   (Lightroom/Photo Mechanic compatible) and applied on export; set from the
   CLI today (`apature-cli crop`). Still to do: the on-screen crop tool.
5. [ ] Keywords panel, GPS, sRGB conversion on export.
6. [ ] FTP / SFTP delivery after export.
7. [ ] Live Ingest (watched folder) → the camera-FTP idea.
Apature-only: verified eject (refuse until every hash is clean; per-file
ledger), assignment-aware defaults, collision and other-day warnings before
the copy. Cross-cutting: easy to read, nothing overlapping, polished, and
Settings › Accessibility (text size, motion, contrast; 2026-10-15).

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

## From the Photo Mechanic gap analysis (docs/research/pm-feature-gap.md, 2026-10-14)
1. ~~Export / Save As (M7)~~ done 2026-10-14: resize, quality, keep EXIF/XMP/IIM/ICC, rename, batch. Left: sRGB conversion for JPEG sources, presets per outlet, watermark. RAW → JPEG: opt-in system RAW decode on macOS (sips/ImageIO) done; a cross-platform demosaic (permissive licence) later.
2. ~~Preview 1:1 + pan, compare view~~ done 2026-10-14 (M8). Left: pan inertia, loupe.
3. ~~Rename in place, move to trash, copy/paste metadata~~ done (M9, M10). Left: persistent {seqn} counters.
4. Ingest polish: ~~card-mount trigger, unmount, auto-suffix~~ done; left: preserve structure, WAV companions, open sheet while ingesting, multi-source.
5. Rotate, ICC previews.
6. Live Ingest (watched folders) → camera FTP.
7. FTP/SFTP delivery.

## 0.3 — professional workflow
- [ ] export presets (long side, quality, sRGB, keep metadata, naming)
- [ ] saved assignment presets (the `generator/` text files become first-class)
- [ ] folder watching for live ingest
- [ ] Windows and Linux builds
- [ ] Installers (requested 2026-10-15): a signed, notarised macOS DMG and a
      Windows installer (MSI via cargo-wix or an Inno Setup .exe), both built
      by CI from a version tag and attached to a GitHub release

## 0.4 — catalogs
- [ ] SQLite index across drives, metadata search, collections
