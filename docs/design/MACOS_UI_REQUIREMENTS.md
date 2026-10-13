# macOS UI requirements and current-state audit

Sources: Apple HIG "Designing for macOS", "Sidebars", "Split views" (read
2026-10-13 via the live site; quotes below are verbatim), the brief in
`docs/BRIEF.md`, and a code audit of `apps/desktop/src/app.rs` against egui
0.33's API. The "current" column is what the prototype **actually does**,
established by reading the code and running it; items marked *not
observed* could not be checked visually because the prototype is an
unbundled binary that macOS screen-capture permissions did not let this
session capture.

## Requirements (R-n are enforceable; each has a test criterion)

| id | requirement | HIG basis | test criterion |
|---|---|---|---|
| R-1 | Full menu bar: app, File, Edit, View, Window, Help, with every command reachable there and shortcuts shown | "Use the menu bar to give people easy access to all the commands they need" | every keyboard action has a menu item; VoiceOver reads the menu |
| R-2 | Standard shortcuts never repurposed: ⌘O ⌘W ⌘Q ⌘, ⌘Z ⌘A ⌘F ⌘H ⌘M ⌘1-9 (view switching per Finder), ⌃⌘F full screen | "Handle keyboard shortcuts to help people accelerate actions" | shortcut table reviewed against Finder/Photos; conflicts listed |
| R-3 | Sidebar on the leading edge, collapsible via View menu + toolbar button + ⌃⌘S, auto-collapse below a width, user-orderable contents, ≤ 2 levels, nothing critical at its bottom | Sidebars page: "Avoid hiding the sidebar by default"; "Avoid putting critical information or actions at the bottom of a sidebar"; "let people customize the contents of a sidebar" | collapse state persists; narrow-window auto-collapse; drag reorder |
| R-4 | Split view with draggable 1 pt dividers, min/max pane widths that keep dividers visible, hide/show inspector via View menu + ⌥⌘I, selection highlighted persistently in each pane | Split views page: "Set reasonable defaults for minimum and maximum pane sizes"; "Prefer the thin divider style"; "Provide multiple ways to reveal hidden panes" | resize persists across launches; divider never vanishes |
| R-5 | Window: resizable, restores size/position, native full screen, multiple displays and Retina at 1x/2x | "Let people resize, hide, show, and move your windows" | relaunch restores frame; moving between displays rescales |
| R-6 | Native open/save panels and native alerts | File management section | folder picker is NSOpenPanel (observable: sidebar, Recents, iCloud) |
| R-7 | Light and dark appearance driven by the system, with every surface and text colour a semantic token | Dark Mode guidance | toggling System Settings appearance updates the app live; no hard-coded hex in UI code |
| R-8 | Reduce Motion honoured: no scroll momentum animation, no fade/slide when on | Accessibility | setting toggled → animations off |
| R-9 | Focus, selection, picked, rating, colour class, reject and filter are separate state with separate visual treatment, none colour-only | brief §5 | state matrix test; grayscale screenshot still distinguishes |
| R-10 | Contact sheet: thumbnails dominant, letterboxed never cropped, cell size via slider + ⌘= ⌘-, filename and marks legible at every size, 60 fps scroll on 10 000 items | "comfortable information density" | frame-time stat under 16 ms while scrolling 1 000 NEFs |
| R-11 | Image presentation: neutral grey surround (no accent near photos), ICC-aware display of previews, honest zoom labels ("fit", "1:1", "43 % of 6048 px") | colour accuracy | AdobeRGB preview renders the same as Preview.app within tolerance |
| R-12 | Background work visible: a progress affordance in the toolbar/status for decoding, ingest, writes; errors as non-modal, actionable rows | "Speed must not hide uncertainty" (Apature) | every long operation has a visible count and a cancel |
| R-13 | Panels collapsible and resizable, remembered per workspace; no docking system | brief §4 | state file round-trip |
| R-14 | Personalisation: customisable toolbar, shortcut profiles, sidebar order, saved workspace layouts | "Support personalization, letting people customize toolbars…" | profiles exportable as JSON |

## Current prototype against these requirements

| id | status | evidence |
|---|---|---|
| R-1 | **missing** | eframe/winit creates only the default app menu (About/Hide/Quit). No File/Edit/View menus exist. egui 0.33 has no native menu API; needs an AppKit shim (`objc2-app-kit`) or `muda`. |
| R-2 | partial | ⌘O opens a folder; ⌘Q/⌘H/⌘M come from the default app menu; ⌘0–5 are used for rating, which collides with the Finder/Photos convention of ⌘1–4 switching views. Photo Mechanic uses ⌘0–5 for ratings on Mac, so this is a documented conflict to resolve in `INTERACTION_SPEC.md`, not an oversight. |
| R-3 | **missing** | no sidebar. |
| R-4 | **missing** | fixed top/bottom panels; egui `SidePanel::resizable(true)` exists and gives a draggable divider, so this is cheap to add. |
| R-5 | partial | resizable; initial 1400×900; frame restore requires `persistence` feature (not enabled); full screen via the green button works through winit (not observed this session); Retina handled by egui `pixels_per_point` (thumbs are decoded at 320 px for ≤ 180 pt cells, i.e. ≥ 1.8 px/pt, sufficient at 2x only for cells ≤ 160 pt — **cells above 160 pt are upscaled on Retina**). |
| R-6 | met | `rfd::FileDialog::pick_folder` uses NSOpenPanel. |
| R-7 | **failing** | `app.rs` hard-codes `Color32::from_gray(24/34/58)` and yellow focus; egui's own widgets follow the system theme, so a light-mode user gets light buttons on a dark grid. |
| R-8 | unknown | no explicit animations in our code; egui's ScrollArea momentum is on by default and is not gated on Reduce Motion. |
| R-9 | partial | focus (cursor) exists; **no selection set**, so focus = selection; rating, class, tag are distinct fields; reject is rating −1 (shares state with rating, which is also Lightroom's model); no filters. Colour class is shown as a colour bar **and nothing else** → fails "not colour-only". |
| R-10 | partial | virtual grid via `show_rows`; letterboxed; slider + +/−; frame-time stat added this session but not yet measured under load. |
| R-11 | **failing** | no ICC handling: embedded previews are drawn as sRGB. Nikon bodies set to Adobe RGB will look desaturated. Zoom labels are not shown at all. |
| R-12 | partial | "decoding N…" count in the toolbar; no cancel; sidecar write errors go to the status line. |
| R-13 | **missing** | |
| R-14 | **missing** | |

## egui capability notes (verified against egui/eframe 0.33 API and source)

* Native menus: none. `eframe` exposes winit's `NSApplication`; a menu bar
  must be built with `objc2-app-kit` or the `muda` crate and bridged to
  egui's action dispatch. Cost estimate: 1–2 days, isolated in a
  `platform/macos.rs` module.
* Accessibility: `eframe` enables AccessKit by default; widgets created via
  `ui.button`, `ui.label`, `ui.selectable_label` get nodes. Painter-drawn
  cells need `ui.interact` + `response.widget_info(|| WidgetInfo::labeled(...))`
  to be announced. Feasible; not done.
* Text input: egui's `TextEdit` lacks system spell-check, dictation, and
  the emoji picker; IME composition works. For a caption editor this is the
  single strongest argument for a native text view, and it is deferred to
  the implementation plan.
* Fonts: egui ships Ubuntu/Noto. SF Pro cannot be bundled, but it can be
  loaded at runtime from `/System/Library/Fonts/SFNS.ttf` on macOS, with a
  fallback elsewhere. Not done.
* High DPI: handled by `pixels_per_point`; text is crisp, images need
  decode size ≥ cell size × ppp.
* Dark/light: `ctx.theme()` follows the system when `follow_system_theme`
  is on (default). Our custom colours must switch on it.
