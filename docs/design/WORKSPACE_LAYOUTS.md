# Workspace layouts (Apature)

Four workspaces, switched with ⌘1–⌘4 (Finder/Photos convention) and the
View menu. Each remembers its own sidebar/inspector visibility and widths.
Panels are collapsible and resizable by divider; no docking.

## Common shell

```
┌ titlebar + unified toolbar ───────────────────────────────────────────────┐
│ [⊞ sidebar] [workspace segmented: Sheet · Preview · Caption · Ingest]      │
│                     [filter ▾] [sort ▾] [size ─●──]   [⟳ 12 decoding] [ⓘ] │
├──────────┬──────────────────────────────────────────────┬─────────────────┤
│ sidebar  │                 workspace                    │   inspector     │
│ 240 pt   │                                              │   320 pt        │
├──────────┴──────────────────────────────────────────────┴─────────────────┤
│ status: 114 photos · 3 selected · DSC_0412.NEF · 1/1600 f/2.8 ISO 400 ·  │
│         ★★★ · 1 Winner · writing sidecar…                       frame 4ms │
└───────────────────────────────────────────────────────────────────────────┘
```

Sidebar (source list, ≤ 2 levels, user-orderable, never critical at the
bottom): **Favorites** (pinned folders), **Recent**, **Cards & Volumes**
(ejectable, with an Ingest button inline), **Contact sheets** (named
working sets, later), **Assignments** (saved presets, later).

## 1. Contact Sheet (⌘1): cull

* Grid fills the workspace edge to edge on `surface.photo`; cells
  letterbox; filename below in mono 11; marks row: stars left, class chip
  and tag right; a 3 pt class bar under the image for at-a-glance scanning
  (plus the chip, so not colour-only).
* Cell size 80–600 pt via slider, ⌘= / ⌘-; `/` focuses filter; filter bar
  shows active filters as removable chips (rating ≥, class, tagged, pair).
* Inspector sections (collapsible, order saved): **Capture** (EXIF),
  **Marks** (editable), **Caption** (read-only preview, click → ⌘3),
  **Files** (RAW+JPEG pair, sidecar status, conflicts).
* States: empty folder; decoding in progress (cells show a quiet
  placeholder, never a spinner per cell); sidecar write error row.

## 2. Preview (⌘2): look

* Workspace becomes `surface.photo` full-bleed; sidebar and inspector
  collapse by default (remembered). Filmstrip 88 pt at the bottom, hidden
  with ⇧F.
* Overlay (top-left, fades after 2 s, returns on pointer move or ⌘I):
  filename, index/count, exposure line, marks. Zoom label is honest:
  "Fit", "1:1", or "Preview 1620 px (27 % of 6048)".
* Keys: ←/→ next, Space toggles 1:1 at pointer, Z fit, ⌘I inspector,
  marks keys as everywhere, Esc back to Sheet.

## 3. Caption (⌘3): write

```
┌ sidebar ┬ photo (fit, ~55 %) ──────────┬ fields (420 pt) ───────────────┐
│         │                              │ 100726 SPO BGLF WPIAL…  (serif) │
│         │  [photo]                     │ Caption  [textarea, 4 lines]    │
│         │                              │  \cf\ tees off on No. 1\tail\   │
│         │  ← 12 / 114 →  ★★★ 1 Winner  │ Headline · Keywords · City ·    │
│         │                              │ Sublocation · State · Country   │
│         │                              │ Creator · Credit · Copyright    │
│         ├──────────────────────────────┤ [Apply to 3 selected] [Next ⏎]  │
│         │ filmstrip                    │ code replacements: cf cc nc oak │
└─────────┴──────────────────────────────┴─────────────────────────────────┘
```

* Keyboard-first: ⌘⏎ saves and advances; Tab order is caption → headline
  → keywords → …; Esc leaves the field; **no culling shortcut fires while
  a field has focus** (the field owns the keyboard).
* Code replacements expand inline with a visible delimiter and an
  autocomplete list after the opening delimiter; the expansion is shown in
  a different tint until the field blurs, so a wrong code is caught.
* Stationery (assignment preset) applies to the selection with a diff
  preview: "3 photos: Credit will change from — to Butler Eagle".
  Fields that differ across the selection show "mixed" in words.

## 4. Ingest (⌘4): copy

```
┌ Source ─────────────────┐  ┌ Destinations ─────────────────────────────┐
│ ◉ NIKON Z 6_2 (card 1)  │  │ Primary   /Volumes/Work/2026/{date}-{slug} │
│ ○ Folder…               │  │ Secondary /Volumes/Backup/…  [optional]    │
│ 412 files · 23.8 GB     │  │ Rename    {date}-SPO-BGLF-{seq:3}          │
│ ☐ only new since last   │  │ Preview:  100726-SPO-BGLF-001.NEF  …       │
└─────────────────────────┘  └───────────────────────────────────────────┘
┌ Plan ───────────────────────────────────────────────────────────────────┐
│ 412 to copy · 0 collisions · 2 destinations · verify SHA-256            │
│ [Start ingest]                                                          │
├ Progress (one row per file, errors stay visible) ───────────────────────┤
│ ✓ 100726-SPO-BGLF-001.NEF   verified both   ✓ 002 …   ⚠ 118 sha mismatch│
└─────────────────────────────────────────────────────────────────────────┘
```

* The plan is computed and shown before anything is copied; collisions
  block Start with a reason. Progress is a list, never only a bar; errors
  are rows with a retry. Source erase is a separate, later step gated on
  "every file verified in every destination" and an explicit confirmation
  listing the count and the destinations.

## Panel memory

Per workspace: sidebar visible/width, inspector visible/width, filmstrip
visible, cell size, sort, filters. Stored as JSON in the app support
directory; exportable as a "workspace layout" (Folio's portable-layout
idea).
