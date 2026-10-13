# Visual identity and design tokens (Apature)

Semantic tokens only; no literal colours in UI code. Values are the
starting set derived from mccal.dev's tokens with two deliberate changes:
**photo surfaces are hue-neutral** (the site's cool near-black would tint
images) and **the light theme is designed, not inverted**.

## Colour tokens

| token | dark | light | use |
|---|---|---|---|
| `surface.window` | `#0B0B10` | `#F4F3F8` | window background behind panels |
| `surface.panel` | `#15151C` | `#FFFFFF` | sidebar, inspector, toolbar |
| `surface.raised` | `#1D1C27` | `#ECEAF4` | fields, popovers, selected rows |
| `surface.photo` | `#161616` | `#D9D9D9` | contact-sheet and preview ground; **neutral grey, no hue** |
| `surface.photo.cell` | `#202020` | `#CFCFCF` | letterbox behind a thumbnail |
| `text.primary` | `#ECEBF3` | `#16151E` | labels, filenames |
| `text.secondary` | `#B7B4CB` | `#4E4B63` | field values |
| `text.muted` | `#8F8CA6` | `#6E6B86` | metadata, hints (6.1:1 on dark, 4.6:1 on light) |
| `line` | `#25242F` | `#DAD8E6` | 1 pt dividers |
| `line.strong` | `#3B3A4B` | `#B9B6CB` | field borders |
| `accent` | `#CFC8F5` | `#5B4FA8` | focus ring, active tab, selected sidebar row text |
| `accent.wash` | `rgba(207,200,245,.10)` | `rgba(91,79,168,.10)` | selected row fill |
| `focus.photo` | `#CFC8F5` | `#5B4FA8` | 2 pt ring around the focused cell, **inset by 2 pt** so it never touches image pixels |
| `select.photo` | `#FFFFFF` at 70 % / `#000000` at 60 % | | 1 pt ring + 8 % wash on selected (not focused) cells |
| `status.ok` / `status.warn` / `status.error` | `#5FBF8A` / `#E0B341` / `#E06C6C` | darker variants | ingest/verify rows, never the sole signal |

Colour classes keep Photo Mechanic's eight hues (`ColorClassScheme`) but
are always shown as **number + swatch + name** (e.g. "1 Winner"): a 10 pt
swatch dot beside the filename, the number in the chip. Ratings are text
stars. Rejected is a struck-through filename plus "✕".

## Typography

| role | face | size / weight | tracking |
|---|---|---|---|
| UI body | system (SF Pro on macOS, Segoe on Windows) | 13 / 400 | 0 |
| UI label | system | 11 / 500 | 0 |
| Metadata and filenames | mono (SF Mono / Plex Mono fallback) | 11 / 400, **uppercase for field names only** | 0.08em on uppercase |
| Section eyebrow | system | 10 / 600 uppercase | 0.12em |
| Editorial moment | Libre Baskerville (bundled, OFL) | 18–22 / 400 | 0 |

The editorial serif appears in exactly three places: the assignment
headline at the top of the Caption workspace, the empty-state of a
workspace, and the About window. Nowhere near controls.

Scale: 10, 11, 13, 15, 18, 22. Line height 1.35 for UI, 1.5 for captions.
UI scale follows the system accessibility text size; cell size is separate.

## Spacing, shape, elevation

* Spacing unit 4 pt: 4 / 8 / 12 / 16 / 24.
* Radii: 4 (chips, fields), 6 (buttons, rows), 8 (popovers). Photographs
  are never rounded.
* Elevation by one of: a 1 pt `line` border, a `surface.raised` fill, or
  a 0 8 24 rgba(0,0,0,.35) shadow for popovers. Never all three.
* Sidebar 200–320 pt (default 240), inspector 280–420 (default 320),
  toolbar 38 pt, status bar 24 pt, filmstrip 88 pt. Thin 1 pt dividers.

## States

| state | treatment |
|---|---|
| hover (pointer) | `surface.raised` fill on rows and buttons; **no hover effect on photo cells** (it would alter the image's surround) |
| pressed | fill darkened 6 %, no scale |
| focus (keyboard) | 2 pt `accent` ring with 2 pt offset; on photo cells, inset ring |
| selected | `accent.wash` fill + `accent` text in lists; `select.photo` on cells |
| disabled | 40 % opacity + explanatory tooltip (Apature's "disabled with a stated reason") |

## Motion

One duration, 160 ms, easing `cubic-bezier(0.22,1,0.36,1)`, used for
panel collapse, popover appearance, preview cross-fade. Zero when Reduce
Motion is on. Nothing animates during culling keystrokes: a rating change
is instantaneous.

## Rendering notes

High DPI: thumbnails are decoded at `ceil(cell_pt × pixels_per_point)`
(the current fixed 320 px must become dynamic). Photo surfaces are drawn
in linear-to-sRGB-correct textures; UI colours are sRGB. ICC: previews
tagged Adobe RGB are converted to display space before upload (Phase 2
engine work; see STATUS).
