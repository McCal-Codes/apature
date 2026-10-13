# Consolidation decision (recorded 2026-10-13, not yet executed)

**The product is called Apature.** The owner decided to continue the Apature
project rather than start a separate Photo Mechanic alternative: this Rust
workspace is the foundation, the valuable work in `~/Documents/Apature`
(Tauri + React, GPL-3) is migrated into it, and the consolidated project
will live at `~/dev/apature`.

Nothing below has been done. This note exists so the next session starts
from the decision instead of rediscovering it.

## Locations

| what | where | treatment |
|---|---|---|
| original Apature | `~/Documents/Apature` | preserved as reference; never deleted or renamed; its uncommitted export work (new `export/` module, `filesystem/names.rs`, `tests/export_raw.rs`, ADR 0006) must be preserved first |
| this foundation | `~/dev/photo-workflow` | preserved until migration passes |
| consolidated project | `~/dev/apature` | to be created as a copy of this repo with history kept |

## Plan summary (owner's words, condensed)

1. Preserve both states: record commit ids, stash nothing, copy, don't move.
2. Branding: display name, Cargo names, bundle name, window titles, CLI,
   README, About, repository description all say "Apature". Not "Aperture",
   no Apple assets. Don't change internal identifiers if that would break
   persisted user data. Verify a reverse-domain identifier before using it.
3. Migrate by subsystem, in this order: RAW previews and orientation; XMP
   preservation; caching and colour management; keyboard culling and
   selection state; ingest and safe file ops; SQLite catalog; code
   replacements; undo/redo; RAW export last. One commit per subsystem with
   tests; no duplicate metadata engines, caches or selection models.
4. Keep egui for now; a framework change needs an evidence-backed proposal.
5. Licensing before any code moves: GPL-3 files do not change licence by
   being copied; verify sole authorship per file; document rawler (LGPL)
   and other obligations; recommend a coherent final licence.
6. Deliverables: `docs/migration/MIGRATION_PLAN.md`, `FEATURE_MATRIX.md`,
   `LICENSING_REVIEW.md`, `ARCHITECTURE_DECISIONS.md`,
   `VERIFICATION_REPORT.md`, and `docs/design/APATURE_DESIGN_SYSTEM.md`.

## What this session already established that the migration can use

* `docs/research/apature-review.md`: subsystem inventory, measured
  numbers, lessons, the inferred orientation bug, ownership evidence
  (37 commits, one author, DCO sign-offs).
* `docs/THIRD_PARTY.md`: this repo's graph is copyleft-free; rawler is the
  one dependency the migration would bring in and must stay optional.
* `docs/research/metadata-safety.md`: what the new XMP writer guarantees,
  so the port of Apature's quick-xml writer can be judged against it.
