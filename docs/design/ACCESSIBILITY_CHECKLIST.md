# Accessibility checklist

Scope: WCAG 2.2 AA as interpreted for native software by WCAG2ICT, Apple's
accessibility guidance, and VoiceOver on macOS. Each item has a method and a
status for the current prototype. "Not tested" means exactly that; nothing
below is assumed from framework capability.

| # | criterion | method | prototype status |
|---|---|---|---|
| A1 | Every interactive element has an accessible name and role | VoiceOver rotor; Accessibility Inspector tree | **fails**: grid cells are painter-drawn with no widget info; toolbar button and slider are announced |
| A2 | Grid is navigable as a grid: VO reads "row x of y, column…" and the cell's filename, rating, class, tag | VoiceOver | **fails** (A1) |
| A3 | Visible keyboard focus on every focusable control, ≥ 3:1 contrast against adjacent colours | screenshot + contrast meter | cursor ring is #FFC828 on #222 ≈ 9:1 (computed, not measured on screen); buttons use egui default focus |
| A4 | All functions available by keyboard alone | manual walk-through | **partial**: no keyboard path to the folder picker besides ⌘O, no menu bar, no Tab order through panels |
| A5 | Status is never colour-only | grayscale screenshot | **fails**: colour class is only a coloured bar; rating uses stars (ok); tag uses a "T" chip (ok) |
| A6 | Text contrast ≥ 4.5:1, large text ≥ 3:1 | contrast meter on tokens | design tokens from mccal.dev are measured at 16.6:1 / 9.7:1 / 6.1:1 on the dark ground (per `tokens.css` comment); prototype uses greys 210/140/110 on 24: 12.9:1 / 6.0:1 / 3.9:1 (the 110 grey placeholder fails for small text) |
| A7 | UI text scales with the system / app setting | egui `zoom_factor` (⌘+/−) | works in egui by default; **collides** with our +/− cell-size keys; to be resolved in the spec |
| A8 | Reduce Motion respected | System Settings toggle | not tested; egui scroll momentum not gated |
| A9 | Pointer targets ≥ 24×24 pt (WCAG 2.5.8) | measure | cells yes; slider thumb and "T" chip are below; chips are not targets (ok) |
| A10 | No time limits; background work never blocks input | run with 1 000 NEFs, type during decode | decode is off-thread; texture upload capped at 48/frame; **not measured** |
| A11 | Errors identified in text, next to where they occur | manual | sidecar write errors appear in the status line only |
| A12 | Light and dark appearances both meet contrast | both modes | **fails**: no light theme |
| A13 | VoiceOver can read the full preview's metadata (filename, exposure, caption) | VoiceOver | not implemented |
| A14 | Drag-and-drop has a keyboard alternative | n/a yet | no drag-and-drop exists |
| A15 | Shortcuts do not fire while typing | type in a text field | no text fields yet; rule required in `INTERACTION_SPEC.md` |

## Manual test script (to run before any release)

1. Enable VoiceOver (⌘F5). Open a folder. Navigate the grid with arrows and
   confirm each cell is announced with filename, rating, class, tag.
2. Press ⌘1 then ⌘3 on a cell; confirm VO announces the change.
3. Switch System Settings → Appearance to Light; confirm every surface
   changes and text stays ≥ 4.5:1.
4. Enable Reduce Motion; scroll; confirm no momentum/easing.
5. Set System Settings → Accessibility → Display → Larger text (or app
   zoom); confirm layout reflows without clipping.
6. Take a grayscale screenshot of a sheet with mixed classes; confirm
   classes are distinguishable.
7. Tab through the whole window; confirm the order is sidebar → toolbar →
   grid → inspector and nothing is skipped or trapped.
