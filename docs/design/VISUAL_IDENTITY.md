# Visual identity and design tokens (Apature)

**Decision (2026-10-13):** Apature has two appearances of one design
system, not two interfaces. **Apature Dark** (Professional Darkroom: deep
charcoal, compact controls, lavender highlights, high-density sheet) and
**Apature Light** (Mac Studio: soft off-white, restrained blue accents,
open spacing). Same layout, same controls, same spacing scale; only the
tokens change, and the switch follows System Settings → Appearance.

Semantic tokens only; no literal colours in UI code. Values derive from
mccal.dev's tokens with two deliberate changes: **photo surfaces are
hue-neutral** (the site's cool near-black would tint images) and **the
light appearance is designed, not inverted**, with system blue rather
than lavender as its accent so it reads as native.

## Colour tokens

| token | Apature Dark | Apature Light | use |
|---|---|---|---|
| `surface.window` | `#121214` | `#F2F2F5` | window background behind panels |
| `surface.panel` | `#1A1A1D` | `#FAFAFC` | sidebar, inspector, toolbar |
| `surface.raised` | `#242428` | `#E9E9EE` | fields, popovers, selected rows |
| `surface.photo` | `#161618` | `#D8D8DC` | contact-sheet and preview ground; **neutral grey, no hue** |
| `surface.photo.cell` | `#202024` | `#CFCFD4` | letterbox behind a thumbnail |
| `text.primary` | `#E8E7EE` | `#1D1D22` | labels, filenames |
| `text.secondary` | `#B3B1C0` | `#4D4D58` | field values |
| `text.muted` | `#8A8899` | `#6F6F7C` | metadata, hints |
| `line` | `#2A2A30` | `#DEDEE4` | 1 pt dividers |
| `line.strong` | `#3B3A4B` | `#B9B6CB` | field borders |
| `accent` | `#CFC8F5` lavender | `#0A66D6` system blue | focus ring, active tab, active filter, selected sidebar row text |
| `accent.wash` | `rgba(207,200,245,.12)` | `rgba(10,102,214,.12)` | selected row and active filter fill |
| `focus.photo` | `#CFC8F5` | `#0A66D6` | 2 pt ring around the focused cell, **inset by 2 pt** so it never touches image pixels |
| `mark.star` | `#E3B341` | `#B8860B` | rating stars (darkened in Light for 4.5:1) |
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

* Spacing scale, 4 pt base: `s1` 4 · `s2` 8 · `s3` 12 · `s4` 16 · `s5` 24 ·
  `s6` 32. Every gap, padding and margin in the UI is one of these.
* Fixed metrics, both appearances: toolbar 48 pt with 16 pt side padding
  and 12 pt between controls; status bar 28 pt; sidebar rows 26 pt tall
  with 8 pt gutters and 12 pt between groups; inspector padding 12 with
  16 pt between sections; filmstrip 8 pt padding, 8 pt between frames.
* Contact sheet: Dark pads 12 and gaps 12 at 6 across; Light pads 16 and
  gaps 16 at 5 across. Inside a cell: 4 pt between image, filename and
  marks row; marks row 16 pt tall.
* Caption workspace: fields column 320 pt, 16 pt padding, 12 pt between
  fields; field boxes 30 pt minimum (caption 84) with 7×9 pt inner
  padding; the photo pane keeps 24 pt around the image.
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

Owner request (2026-10-14): "clean animations". The list of candidates and
the exclusions live in `docs/ROADMAP.md`; the rule stays: motion explains a
state change in one beat and never delays the next keystroke.

## Rendering notes

High DPI: thumbnails are decoded at `ceil(cell_pt × pixels_per_point)`
(the current fixed 320 px must become dynamic). Photo surfaces are drawn
in linear-to-sRGB-correct textures; UI colours are sRGB. ICC: previews
tagged Adobe RGB are converted to display space before upload (Phase 2
engine work; see STATUS).
