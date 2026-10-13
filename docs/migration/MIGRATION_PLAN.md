# Apature: consolidation and "up and running" plan

Goal, in the owner's words: get it working and up and running. So the
order below optimises for the first day a real assignment is culled and
captioned in Apature, not for completeness. Dates are targets, not
promises; each step ends with fmt, clippy, tests, release build and a
line in `VERIFICATION_REPORT.md`.

## State at planning time (2026-10-13)

| repo | HEAD | notes |
|---|---|---|
| `~/Documents/Apature` (Tauri, GPL-3) | `caae611` 2026-09-24 | **uncommitted**: `export/` module, `filesystem/names.rs`, `tests/export_raw.rs`, ADR 0006, edits to five files. Preserved as is; never deleted or renamed. 8 120 lines of core Rust. |
| `~/dev/photo-workflow` (this repo, MIT/Apache) | `efe5c65`, 7 commits, pushed to private `McCal-Codes/photo-workflow` | 4 836 lines of Rust, 36 tests, verified against exiftool, Photo Mechanic 2026.6 and Capture One 16.8. |

## M1: consolidate and rebrand (this session)

1. `git clone ~/dev/photo-workflow ~/dev/apature` (history kept; original untouched).
2. Rename crates `photo-*` → `apature-*`, desktop binary `apature`, CLI
   binary `apature-cli`, window title and About "Apature", cache dir under
   the identifier `dev.mccal.apature` (reverse of mccal.dev, which the owner
   controls). README and docs say Apature.
3. New private repo `McCal-Codes/apature`; `photo-workflow` stays as the
   frozen pre-rename history until the owner archives it.
4. Done when: `cargo test` passes, `apature <folder>` opens a sheet, the
   thumbnail cache repopulates under the new identifier.

## M2: shell (next, the big one, ~3 sessions)

Replaces the probe UI with the designed one, both appearances. Scope is
`docs/design/DESIGN_IMPLEMENTATION_PLAN.md` S0 + S1: tokens, menu bar via a
thin macOS layer, source-list sidebar, inspector with preview on top,
filmstrip, status bar, light/dark following the system, cell accessibility
names. Done when the HIG checklist rows R-1, R-3, R-4, R-5, R-7, R-13 flip
to met and VoiceOver reads a cell.

## M3: cull for real (~1 session)

Selection model, shift-range, filters by rating/class/tag, marks applied to
the selection, undo (one entry per batch; port the rule set from Apature's
`undo` store, not its React code). Done when a 1 000-file assignment can
be culled start to finish with the keyboard and Photo Mechanic agrees with
every mark.

## M4: captions (~2 sessions)

IPTC fields in `apature-metadata` on a streaming XML parser (quick-xml,
porting the property table and the merge rules from Apature's
`metadata/xmp.rs`, relicensed by the owner), stationery presets from the
existing Butler Eagle generator files, code replacements with the existing
`code-replacements-100726.txt` format, the Caption workspace. Done when
the golf assignment's captions written in Apature show correctly in
Photo Mechanic and exiftool, and `\cf\ tees off\tail\` expands.

## M5: ingest (~1–2 sessions)

Port Apature's planner (`ingest/plan.rs`: templates with Photo Mechanic
variable names, collision refusal, incremental by fingerprint) and run
loop onto `apature-ingest::copy_verified`; the Ingest workspace. Done when
a card ingests to two destinations with per-file verification and a
report.

## Later, deliberately

SQLite catalog (Apature's migrations v1–v5 are the starting point), RAW
export (Apature's uncommitted work; rawler behind a feature flag),
histogram, HEIC/CR3, Windows.

## Subsystem decisions (summary; detail in FEATURE_MATRIX.md)

| subsystem | source of truth | why |
|---|---|---|
| previews, cache | **new** | 4 ms/NEF, no copyleft, verified vs exiftool; add Apature's provenance-in-filename and pipeline version |
| XMP marks | **new** | verified in three apps; Apature's merge rules already matched |
| IPTC fields, sidecar property table | **port from Apature** | 1 200 lines of tested quick-xml code; owner-authored |
| EXIF | **new** (own TIFF walker) | Apature's needed rawler for CR3/RAF; revisit for CR3 |
| ingest planner/run | **port from Apature** | 28 tests, PM variable names, incremental ingest |
| file ops (copy/move/trash) | **port from Apature** | safety rules already encoded |
| selection, keymap, undo rules | **adapt from Apature** (TypeScript → Rust) | the rules are good; the code is React |
| catalog | **defer**, then port migrations | not needed to cull and caption |
| RAW export, histogram | **defer** | rawler dependency; unfinished |
| UI | **new** (egui, designed) | decided; Tauri/React not carried over |
