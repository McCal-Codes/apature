# Design implementation plan (Apature)

Preconditions: the owner picks a direction (recommended: Darkroom surfaces
on the Native Editorial skeleton). Nothing here replaces the current UI
until then. All work lands on a `design/shell` branch in new files under
`apps/desktop/src/ui/`; `app.rs` is retired when the new shell reaches
parity, not edited in place.

## egui assessment (what forces a native layer)

| need | egui 0.33 | plan |
|---|---|---|
| menu bar | none | `platform/macos.rs` with `muda` (MIT) or `objc2-app-kit`; actions dispatched into the same command enum the keyboard uses; Windows gets `muda` too |
| native panels, alerts | `rfd` ok | keep |
| appearance, Reduce Motion, accent colour | theme yes; others no | read `NSWorkspace` accessibility flags and `NSColor.controlAccentColor` in the platform layer |
| SF Pro / SF Mono | not bundled | load from `/System/Library/Fonts` at runtime; fall back to bundled Inter/Plex Mono |
| VoiceOver | AccessKit nodes for registered widgets | give every cell `WidgetInfo` with the full name; test with the checklist |
| caption text editing | `TextEdit` without spell-check/dictation | v1 egui; measure complaints; v2 option: an `NSTextView` overlay positioned over the egui field (prototype cost ~3 days) |
| window state restore | `persistence` feature | enable |
| full screen, multi-display, Retina | winit ok | verify on an external 1x display |

Conclusion: **egui stays** for the grid, preview and inspector, with a
thin, isolated macOS layer. A SwiftUI rewrite is not justified by the
evidence; the one open risk is the caption editor, and it has a bounded
fallback.

## Phases

| phase | scope | files | core API change | tests |
|---|---|---|---|---|
| S0 tokens | `ui/tokens.rs`: semantic colours, type scale, spacing; light/dark from system; Reduce Motion flag | new | none | snapshot of token values; contrast computed in a unit test |
| S1 shell | menu bar, unified toolbar, side panels with dividers and memory, status bar, workspace switcher | `ui/shell.rs`, `platform/macos.rs`, `platform/mod.rs` | none | manual HIG checklist R-1…R-5, R-13 |
| S2 sheet | new grid with cell accessibility names, selection model, filters, sort | `ui/sheet.rs`, `state/selection.rs` | `photo-core`: a `Marks` predicate helper for filters | unit tests for selection rules; VoiceOver script A1–A5 |
| S3 preview | overlay, filmstrip, honest zoom, 1:1 | `ui/preview.rs` | `photo-preview`: expose chosen candidate size for the label | latency measured |
| S4 caption | fields, code replacements, stationery diff | `ui/caption.rs` | `photo-metadata`: IPTC property table on a streaming parser (Phase 3 engine work) | round-trip tests as for marks |
| S5 ingest | plan/progress UI | `ui/ingest.rs` | `photo-ingest`: plan + run API (Phase 4) | fault-injection tests |

Each phase ends with fmt, clippy, tests, a release build, and a dated
entry in `docs/STATUS.md` saying what was verified.

## Windows

Same core, same `ui/` code; `platform/windows.rs` provides menus via
`muda`, Segoe fonts, and the Fluent shortcut table (Ctrl for ⌘). No
Windows work before S3 is done on macOS.

## Risks

* egui text rendering at 11 px mono may look soft on 1x displays; verify
  on an external monitor before committing to the density of direction 2.
* AccessKit + VoiceOver on a custom grid has not been tried by this
  project; S2 includes a spike before the full grid.
* The caption editor is the one place a native control may be required.
