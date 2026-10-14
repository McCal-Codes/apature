# Interaction specification (Apature)

## State model (all distinct)

| state | cardinality | visual | persisted |
|---|---|---|---|
| focus | exactly one cell (or none) | 2 pt accent ring, inset | no |
| selection | zero or more | white/black ring + wash; count in status | per session |
| tagged | per photo | "T" chip + filled flag glyph | XMP `photomechanic:Tagged` |
| rating | per photo, 0–5 | text stars | XMP `xmp:Rating` |
| rejected | per photo | strike + ✕; stored as rating −1 (Lightroom convention) | XMP `xmp:Rating="-1"` |
| colour class | per photo, 0–8 | number + swatch + name, 3 pt bar | XMP `photomechanic:ColorClass` + `xmp:Label` |
| filters | per workspace | chips in filter bar; status says "showing 37 of 114" | layout file |

Rules: keyboard marks apply to the **selection if the focused cell is in
it, otherwise to the focused cell only** (so a lone cursor behaves like
Photo Mechanic, and a multi-select behaves like Lightroom). Arrow keys
move focus and collapse the selection to it unless ⇧ (range) or ⌘ (keep)
is held. "Advance after mark" is on by default and shown in the status
bar. Marks are undoable (⌘Z), one entry per batch.

## Shortcut table (default profile "Photo Mechanic")

| action | key | note |
|---|---|---|
| move focus | ← → ↑ ↓, Home, End, PgUp, PgDn | |
| extend selection | ⇧ + arrows, ⇧-click, ⌘-click | |
| select all / none | ⌘A / ⌘⇧A | |
| colour class 1–8 / clear | 1–8 / 0 | single-key, as the owner's PM prefs |
| rating 0–5 | ⌘0–⌘5 | Photo Mechanic's Mac mapping (its Image › Set Rating menu confirmed 2026-10-13) |
| reject / unreject | X / U | Lightroom convention; PM has no reject |
| tag toggle | T | |
| preview / back | ⏎ or Space / Esc | Space = 1:1 inside Preview |
| workspaces | ⌘⇧1 ⌘⇧2 (⌘⇧3 ⌘⇧4 later) | ⌘1–5 are Photo Mechanic's ratings, so workspaces take ⇧ (decided in M3) |
| cell size | ⌘= ⌘- | conflicts with egui's UI zoom, which moves to Settings |
| sidebar / inspector / filmstrip | ⌃⌘S / ⌥⌘I / ⇧F | HIG: also in View menu |
| clear filters | ⌥⌘F | filter popup from the toolbar button |
| open folder / ingest | ⌘O / ⌘⇧I | |
| save caption and advance | ⌘⏎ | Caption workspace |
| undo / redo | ⌘Z / ⌘⇧Z | |
| full screen | ⌃⌘F | system |

Profiles are JSON files (`shortcuts/photo-mechanic.json`,
`shortcuts/lightroom.json`), editable in Settings, with conflict detection
against the reserved macOS set (⌘Q ⌘W ⌘H ⌘M ⌘, ⌘Tab ⌃⌘F). A profile cannot
rebind those.

## Text fields own the keyboard

When any text field has focus, no single-key or ⌘-number culling shortcut
fires. Esc blurs. This is enforced centrally (one `shortcut_guard` check
in the dispatcher), not per widget.

## Pointer

Click focuses; double-click opens Preview; drag selects a marquee;
right-click opens a context menu that acts on the selection; drag-out to
Finder copies the files (later). Every pointer action has a keyboard
equivalent listed above.

## Accessibility behaviours

Grid cells expose name "filename, rating, class, tag, index of count";
marks changes are announced; the status bar is a live region; focus is
never lost when a panel collapses (it moves to the workspace); overlays
in Preview are reachable with ⌘I when hidden; Reduce Motion disables the
overlay fade and panel animation.

## Performance budget per interaction

| interaction | budget | how measured |
|---|---|---|
| keystroke to mark visible | 1 frame | frame-time stat |
| sidecar write | off UI thread, < 50 ms typical | log |
| focus move in Sheet | 1 frame; thumbnail from cache ≤ 5 ms | bench warm numbers |
| Preview next/prev | ≤ 50 ms with ±1 prefetched; ≤ 250 ms cold | to measure |
| open 1 000-file folder | grid visible < 100 ms; all visible thumbs < 1 s warm | to measure |
