# Apature review (read-only, 2026-10-13)

`~/Documents/Apature` is the owner's earlier attempt at the same product:
a Tauri 2 + Rust + React culling/ingest app. Nothing in it was modified.
This is what it contains, what it learned, and what that means for
photo-workflow. Full detail came from a read of all 37 commits, `docs/`,
`crates/apature-core` and `src-tauri`.

## Ownership and licensing

* 37 commits, all by `McCal-Codes`, each with a `Signed-off-by: McCal-Codes`
  trailer; no other authors, no `Co-authored-by`, no tags, one branch,
  remote `github.com/McCal-Codes/Apature`. First commit 2026-09-02, last
  2026-09-24. Uncommitted work in progress: an export module (develop RAW
  via rawler, embed XMP + IPTC-IIM into JPEGs), ADR 0006.
* Declared licence GPL-3.0-or-later, SPDX headers "The Apature project",
  `authors = ["Apature contributors"]`. Those are boilerplate; the
  copyright holder is the sole author.
* **Relicensing options.** As sole author, the owner can relicense any code
  he wrote under MIT OR Apache-2.0 at will; the GPL binds recipients, not
  the author. What he *cannot* relicense: (a) third-party crates, which
  keep their own licences (notably **rawler 0.8, LGPL-2.1**, and
  `moxcms`), and (b) any code that was itself derived from GPL sources.
  Nothing in the repository indicates (b), but a quick pass for copied
  snippets (histogram sampling, X-Trans notes) is prudent before any port.
  Practical path if code is reused: port the specific module into
  photo-workflow, drop the SPDX GPL header, note the origin in
  `docs/DECISIONS.md`. Because photo-workflow avoids rawler (ADR-002), the
  rawler-dependent modules (`previews/rawler_engine.rs`, `export/`,
  `metadata/raw_exif.rs`) would not transfer as-is anyway.

## Architecture (what exists)

* Workspace of `crates/apature-core` (no Tauri dependency, by design) and
  `src-tauri`. Core modules: `analysis/histogram`, `database` (SQLite,
  forward-only migrations v1–v5), `fileops` (copy/move/trash),
  `filesystem` (classify, stack basename, 64 KiB quick fingerprint),
  `ingest` (plan + run), `jobs` (bounded `WorkerPool`, `CancelToken`),
  `metadata` (kamadak-exif, rawler, a 1 200-line quick-xml XMP sidecar
  writer, an unused exiftool engine), `models`, `previews`, `validation`
  (path allowlist).
* 28 named Tauri commands; previews served by path over the `asset:`
  protocol, not base64 over IPC ("hundreds of megabytes of transient
  allocation").
* React owns transient UI state only: selection, filters, sort, grid size,
  named contact sheets, undo/redo (200 deep, one entry per multi-frame
  action), localStorage prefs, a LIFO preview request queue with 6 in
  flight.
* Dependencies worth noting: rawler 0.8 (LGPL), image 0.25, moxcms (ICC),
  kamadak-exif, quick-xml, rusqlite (bundled), trash, walkdir, sha2, md5
  (IPTCDigest), ts-rs, criterion. No file watcher, no tokio, no rayon.

## What it got right (carry over)

1. **GUI-free core crate**; `debug = 0` in dev/test profiles; only ship the
   crate types you need (removing a 171 MB staticlib took the edit-test
   loop from ~14 min to seconds). photo-workflow already follows this.
2. **Cache provenance in the filename** (`{key}.{ep|et|nd}.jpg`),
   `PIPELINE_VERSION` in the key, `.partial` + rename, one `finish()` choke
   point that always applies orientation and colour. photo-workflow has the
   key and atomic write; it lacks provenance and a pipeline version.
3. **Colour after downscale**, transforms cached by profile hash, untagged
   images passed through, tested with P3/AdobeRGB reds. photo-workflow has
   no colour management yet (see `docs/STATUS.md`).
4. **Sidecars merge, never rebuild**; refuse malformed input; read back to
   verify; empty box = unchanged; write `Iptc4xmpCore:Location` not
   Sublocation; rating/label written to XMP but the catalog wins on read.
   photo-workflow's writer follows the same rules without a catalog.
5. **Verified copy = full SHA-256**, never the prefix fingerprint; plan
   whole job, reject collisions first; sequential from cards; move = copy +
   verify + remove; delete = system trash; erase-card gated per file on full
   verification plus a second destination plus explicit acknowledgement.
6. **Focus ≠ selection**: keyboard marks act on the focused frame; menus
   act on the selection; advance-after-mark default on; shortcuts
   suppressed inside any text input (`[data-shortcut-guard]`).
7. **Accessibility done for real**: `role="grid"` + roving tabindex,
   `aria-label` with pick and rating per cell, measured focus-ring contrast
   (5.15:1 light, 6.07:1/7.65:1 dark), native `<dialog>`, `aria-live`
   status, reduced motion honoured, text stars not icons, letterboxed never
   cropped thumbnails, "mixed" spelled out for multi-select edits, Cancel
   as default button in destructive dialogs.
8. **Measured numbers**: scan 11.9k files/s flat to 10k files; sort key
   memoised 7.1 → 4.4 ms per dispatch; 2560 px loupe must say it is 43 % of
   a 6000 px frame, never "100 %".
9. **ADR-0005: no automated culling.** The product competes on being right
   about what you are looking at, losing nothing, and the speed of the
   human loop.

## What went wrong (avoid)

* Three "green by construction" incidents: CI ran against the wrong
  manifest (no tests ran), Vitest exited 0 when its worker died, docs and
  CHANGELOG left stale ("None of this code exists yet" about code that
  existed).
* Probable bug: embedded RAW previews from rawler never get EXIF
  orientation applied (rendered files and the develop path do).
  photo-workflow rotates every preview and has a unit test for it; a
  portrait-NEF fixture test is still missing.
* `/Volumes/` prefix as the removable-media heuristic is too crude.
* rawler cost 493 crates and 10–25 minute cold builds, bilinear X-Trans
  demosaic, and four reachable panic sites wrapped in `catch_unwind`.
* A `PreviewSource::Cache` variant, base64 previews over IPC, reopening
  folders at startup, `object-fit: cover` thumbnails, and a path-prefix
  notion of contact sheets were all tried and reverted.

## UI ideas worth keeping for the design phase

Named contact sheets as working sets independent of folders; a crop guide
in the loupe; pinch zoom anchored on the pointer up to 16:1; a near-black
chrome with a deliberately desaturated accent "because a saturated colour
next to a photograph biases the eye"; radii 4/6/10; "no decorative
controls: every control works, is disabled with a stated reason, or is
named as deferred".

## Status labels

* Confirmed by reading code and commits: everything above except the
  orientation bug, which is **inferred** (rawler's source was not available
  locally to check whether `preview_image` rotates).
* Not run: Apature was not built or launched in this session.
