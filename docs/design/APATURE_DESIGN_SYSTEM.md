# Apature design system

One system, two appearances: **Apature Dark** (Professional Darkroom) and
**Apature Light** (Mac Studio). This file is the index; the detail lives in
the documents it points to, and the mockup is the visual reference.

| what | where |
|---|---|
| colour tokens per appearance, type scale, spacing scale (4/8/12/16/24/32), fixed metrics, states, motion | `VISUAL_IDENTITY.md` |
| the four workspaces and panel memory | `WORKSPACE_LAYOUTS.md` |
| state model, shortcut table, text-field guard, pointer, a11y behaviours, latency budgets | `INTERACTION_SPEC.md` |
| macOS requirements with HIG citations and the audit of the probe | `MACOS_UI_REQUIREMENTS.md` |
| accessibility criteria and manual test script | `ACCESSIBILITY_CHECKLIST.md` |
| how it is built in egui, phases S0–S5, the thin macOS layer | `DESIGN_IMPLEMENTATION_PLAN.md` |
| visual reference, both appearances and the Caption workspace | `mockups-design-directions.html` |

## Reusable controls (to be implemented in `apps/desktop/src/ui/`)

| control | notes |
|---|---|
| `tokens` | `Theme { surface, text, line, accent, mark }` for both appearances; chosen from the system appearance at frame start |
| `source_list` | sidebar groups and rows, 26 pt rows, selected = accent wash + accent text |
| `toolbar` | 48 pt, segmented workspace switcher, filter chips (accent outline when active), size slider, busy counter |
| `sheet_cell` | letterboxed image, 3 pt class bar, chip (number + swatch + name when room), stars, tag chip, inset focus ring, selection ring; accessible name = "filename, rating, class, tag, i of n" |
| `inspector` | preview on top with honest size label, collapsible sections with uppercase 10 pt eyebrows, `kv` grid 64 pt label column |
| `filmstrip` | 88 pt, 60 pt frames, current frame accent ring |
| `status_bar` | 28 pt mono, counts, focused file, marks, write state, frame time |
| `field` | 30 pt min, 7×9 padding, "mixed" in words, code expansion tint until blur |
