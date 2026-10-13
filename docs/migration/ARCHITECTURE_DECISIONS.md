# Architecture decisions for the consolidated Apature

The running decision log is `docs/DECISIONS.md` (ADR-000 … ADR-010).
Migration-specific decisions:

* **One metadata engine**: `apature-metadata`, in-place XMP edits
  (verified) plus the ported IPTC property table on quick-xml. No exiftool
  engine, no second XMP writer.
* **One cache**: `apature-preview::ThumbCache`, extended with provenance
  and a pipeline version. Apature's `previews/cache.rs` trimming is ported;
  its rawler engine is not.
* **One selection model**: focus, selection, marks, filters as distinct
  state in the egui app (`docs/design/INTERACTION_SPEC.md`); Apature's
  TypeScript rules are the spec, not the code.
* **GUI-free core stays law**: no egui types below `apps/`.
* **Catalog later, as an index**: metadata stays in files; SQLite only
  caches and holds working state, exactly Apature's rule.
* **rawler behind a feature flag** if and when export or CR3 lands.
