# UI lineage: what Apature borrows from Caleb's other work

Apature's interface is meant to be the next iteration of the interfaces
Caleb has already shipped, not a fresh start. Before changing a screen,
look at how the same problem was solved in these, and carry the answer
over unless Apature has a reason not to. This file is the index of those
answers; add to it every time a pattern is borrowed or deliberately
dropped.

## Sources

| Project | What it is | Where to look |
| --- | --- | --- |
| TerraNova (`McCal-Codes/TerraNova`) | Desktop design studio, Tauri + React | `src/components/dialogs/settings/*`, `src/settings/registry.ts`, `src/components/ui/settingsPrimitives.tsx` |
| Folio (`McCal-Codes/folio`) | iPhone-style Android launcher | Settings, the Tweak Library, onboarding, the Roadmap screen |
| mccal.dev (`McCal-Codes/mccal.dev`) | Portfolio site | `src/content/projects.ts`, the index numbering, eyebrow labels, the meta table |
| Apple HIG | The platform Apature ships on first | System Settings, Photos, Finder |

## Patterns carried over

**Settings (from TerraNova, adopted 2026-10-23).**
- One registry of definitions: id, page, section, label, one-sentence
  description, control kind. Pages render from it; adding a setting never
  means touching a page.
- A rail of categories on the left with a count of what differs from the
  default; a search box on top that finds any setting by label or
  description and shows a breadcrumb ("Ingest › Templates").
- Rows that all look the same: label and description on the left, the
  control on the right, an accent bar and a "Modified" tag when the value
  is not the default, a Reset on the row and one for the page.
- Things that are not a single value (delivery targets, colour classes, the
  shortcut table) are panels under a category, found by section name.
- Dropped: TerraNova's "Restart required" and "Experimental" badges, because
  nothing in Apature needs a restart and nothing is shipped experimental.

**Progressive disclosure (from Folio and Apple).**
- The first screen shows the few things a photographer touches on
  deadline; everything else sits one level down (a card, a page, a
  disclosure). Ingest shows source, destinations and the plan; naming
  templates and sequence counters are in Settings.
- Short descriptions under every control, written as what happens, not
  what the control is called.
- Switches for on/off, chips for a small closed set, a slider with a unit
  for a number. Never a checkbox with a sentence as its label.

**Numbering and eyebrows (from mccal.dev).**
- Small uppercase eyebrow labels name a group (APPEARANCE, PLAN, CAPTURE);
  the group itself is a quiet card with a hairline, never a heavy box.
- Index numbers where order matters (the welcome's four steps, the
  sections of a case study).

**Onboarding (from Folio's first run).**
- Four pages, each with a way out, the loop before the settings. Tips come
  one at a time, the first time a surface opens, and can be turned off in
  one click.

**Native first (Apple).**
- Settings in its own window on ⌘, ; a real menu bar; the system trash
  sheet; a menu bar extra for status. egui draws the inside, the platform
  draws the frame.

## How to use this file

1. Before building or restyling a surface, find the nearest precedent above
   and read the source file.
2. Borrow the structure and the wording style, not the pixels; the tokens
   in `apps/desktop/src/ui/tokens.rs` are Apature's.
3. Write down what was borrowed and what was dropped, here, in one line each.
4. When Caleb ships something new elsewhere, add it to Sources.

**Workspaces as cards (2026-10-23).** Caption and Export now use the shared
`ui::card` (eyebrow, raised fill, hairline, 8 px radius) with the primary
action pinned in a bottom bar, the way Settings and TerraNova's dialogs do.
Export follows the reading order of Photo Mechanic's Save Photos As dialog:
preset, which photos, destination and names (with an example), the image
(format, size, quality), the metadata, then where to send. Photo Mechanic
puts the file-name variables and a live example beside the template, keeps
sharpening and colour-space choices in the same dialog, and uses snapshots
for presets; Apature keeps the example, the presets as chips, and leaves
sharpening out until someone asks.
