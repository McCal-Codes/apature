# Licensing review for the consolidation

## Facts

* `~/Documents/Apature`: GPL-3.0-or-later; 37 commits, all authored and
  signed-off by `McCal-Codes`; no other contributors, no `Co-authored-by`;
  SPDX headers say "The Apature project", `authors = ["Apature
  contributors"]` (boilerplate; no other person is identified anywhere).
* This repo: MIT OR Apache-2.0, copyright "Caleb McCartney and
  photo-workflow contributors"; 452-package graph with no GPL/LGPL/CDDL
  (see `docs/THIRD_PARTY.md`).
* Apature's non-permissive dependencies: `rawler` 0.8 (LGPL-2.1), used by
  previews, histogram, RAW EXIF and export. Everything else permissive.

## Conclusions

1. **Copying files does not relicense them.** A file under GPL-3 stays
   GPL-3 until its copyright holder says otherwise.
2. **The owner is the copyright holder of every line he wrote**, and the
   author history is consistent with sole authorship. It is evidence, not
   proof: the remaining risk is code copied *into* Apature from GPL
   sources. A pass over the ported modules for copied snippets (histogram
   sampling, X-Trans notes, exiftool tag tables) is part of each port.
3. **Relicensing act:** when a module is ported, the owner changes its
   SPDX header to `MIT OR Apache-2.0` in the Apature repo (or records the
   grant in `docs/migration/RELICENSE.md` with the file list). That record
   is what makes the port legitimate; do it per port, not in bulk.
4. **rawler stays out of the default build.** Any module that needs it
   (export, histogram, CR3) goes behind a Cargo feature, so the shipped
   binary's licence story stays permissive. LGPL via static linking would
   impose relinking obligations on a binary release.
5. **Final licence for Apature: MIT OR Apache-2.0**, as this repo already
   is. The original Apature repo is archived under its GPL-3; nothing
   needs to change there.
6. Font notices (Ubuntu, Noto) and the ICU notice ship with binaries
   (`THIRD_PARTY_NOTICES`, to be generated with `cargo-about`).
