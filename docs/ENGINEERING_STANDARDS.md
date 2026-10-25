# Engineering standards

Adapted from Folio's practice of giving every rule an identifier and a test
you can run, cut down to what Apature needs. Each standard says what must be
true and how that is checked. The design rules (tokens, motion, spacing) stay
in `docs/design/`; release steps stay in `docs/RELEASES.md`.

| id | standard | the check |
|---|---|---|
| **DAT** | **Data integrity.** The image data of an original is never changed. Metadata is: marks, rotation, crop and straighten go to the XMP sidecar next to a RAW; for a JPEG the metadata is written into the file's own XMP segment and the image data is untouched. Copies are verified by SHA-256 before they count. Nothing is deleted without an explicit step that names it (Trash, not removal). | copy/verify tests; `crates/apature-metadata` write tests re-read what they wrote; ExifTool comparison on fixtures for any new write path |
| **STA** | **State.** One owner per piece of persistent state: prefs (`prefs.json`, versioned, backed up on migration), the ingest ledger, the delivery ledger, sidecars. Interrupted work (ingest, delivery, trash, restore) leaves a state that can be read and resumed or rolled back; undo entries follow photos by path. | migration tests; kill-and-resume run for ingest; undo/remap tests |
| **INT** | **Interaction.** Everything on the contact sheet works from the keyboard. Bare keys are handled by the app's own view (a menu accelerator without ⌘ is only a label on macOS). Destructive actions are immediate only when undo exists; otherwise they ask. Long work can be cancelled. | a real-keystroke check of 1–5, T, X, Delete, ⌘Z on each release (`docs/screenshots` or a note in the PR) |
| **MOT** | **Motion.** Animation explains a change and never holds up the next action; it is interruptible and honours Reduce Motion (`App::anim`). | review against `docs/design/`; Reduce Motion run |
| **PRF** | **Performance.** Numbers, not feelings: first thumbnails, preview switch, ingest throughput, memory on a large folder are measured with `apature-cli bench` and recorded in `docs/benchmarks/`; a regression of more than 20 % needs a reason in the PR. | benchmark table updated when touching decode, thumbnails or ingest |
| **SEC** | **Untrusted input.** Files from cards, drives, other people's sidecars, the network and the assignment sheets are hostile until parsed: no panic, no unbounded allocation, no path escape. Credentials live in the system keyring, never in prefs or logs. See `docs/THREAT_MODEL.md`. | `crates/apature-metadata/tests/malformed_input.rs`; a threat-model row for every new input |
| **TST** | **Verification.** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets`, `cargo test --workspace` before a merge; targeted tests while working. Anything that writes to a photo also gets a fixture test, and a real-card or real-camera test before a release (`docs/RELEASES.md`). Look at the output (contact sheet, screenshot) before calling visual work done. | CI; the PR checklist |

## Known gaps against these standards

* **STA:** an interrupted Copy/Move (planned) must be resumable; ingest and
  delivery already keep ledgers.
* **PRF:** budgets are not set yet; the first numbers come from a 300-frame NEF
  folder on a cloud drive (see `docs/benchmarks/`).
* **SEC:** no fuzz target yet; the deterministic malformed-input tests are the
  stand-in. A `cargo-fuzz` target for the TIFF/EXIF and XMP parsers is next.
* **INT/MOT:** screen-reader labels and keyboard focus order are not fully
  verified; track in `docs/STATUS.md`.
