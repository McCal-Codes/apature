# Status

Last updated 2026-10-13. Vocabulary: **implemented** = code exists;
**tested** = automated test passes or a manual run is recorded here;
**verified** = tested against an external reference (exiftool, another
app, a measurement); **production-ready** is not claimed for anything.

## Engine

| feature | status | evidence |
|---|---|---|
| Folder scan, RAW+JPEG pairing, sort | tested | unit tests; `apature-cli scan` on 3 folders |
| TIFF/IFD reader | tested | unit tests incl. hostile input |
| Embedded preview discovery (IFD chain, SubIFDs, Exif IFD, Nikon MakerNote PreviewIFD, plain JPEG) | **verified** | identical previews and byte counts to exiftool on D850 NEF; synthetic TIFF tests |
| JPEG decode with DCT scaling, orientation, thumbnail | tested | round-trip tests; D850 NEF/JPG runs |
| Disk thumbnail cache (key: path, size, mtime, size) | tested | unit test; bench warm runs |
| EXIF summary (make/model/lens/date/exposure/orientation) from TIFF and JPEG APP1 | **verified** | matches exiftool on D850 NEF |
| XMP sidecar read/write, in-place, atomic, read-back checked | **verified** | exiftool; **Photo Mechanic 2026.6 round trip both directions** (NEF-only and pairs); **Capture One 16.8 round trip both directions** (reads sidecar + embedded; its sidecar rewrite handled, stale ColorClass resolved by label); see `docs/research/metadata-safety.md` |
| Embedded-XMP read **and write** in JPEG, pair policy (sidecar + embedded), conflict report | **verified** | unit tests; exiftool on a D850 JPEG; Photo Mechanic shows embedded marks for pairs |
| Verified copy (temp → SHA-256 both sides → rename; never overwrite; never delete source) | tested | unit test only; no card-reader run |
| CR3 / HEIC / RAF-with-no-TIFF previews | **not implemented** | returns "no embedded JPEG" |
| RAW decode fallback when no preview | **not implemented** | |
| ICC colour management | **not implemented** | previews shown as sRGB |
| IPTC Core fields (title, headline, caption, keywords, creator, credit, source, copyright, caption writer, city, state, country, country code, sublocation, date created, instructions, transmission ref, usage terms) read/written in place incl. rdf:Alt/Seq/Bag | **verified** | unit tests on Lightroom/PM-style sidecars and self-closing Descriptions; exiftool validates sidecar and JPEG after a write; marks survive |
| Code replacements (PM file format, `\code\`, `{iptcdow} {iptcmonthnameap} {iptcday} {iptcyear4} {credit} {headline}`) | **verified** | unit tests; `\cf\ tees off on No. 1\tail\` expands to the full AP caption |
| Assignment stationery from the owner's `key: value` generator file → IPTC pad, caption template, code table | **verified** | unit tests; `apature-cli pad` and `caption` on the scratch pair, exiftool read-back |
| Ingest planning, renaming, dual destinations, recovery | **not implemented** | Phase 4; only `copy_verified` exists |
| Catalog | not started | Phase 4+ |

## Desktop shell (`apps/desktop`, M2, 2026-10-13)

The designed shell replaced the probe. Verified by automated screenshots of
the running app in both appearances (`apature <folder> --theme dark|light
--screenshot out.png`) and by querying the native menu bar through System
Events.

| feature | status |
|---|---|
| Tokens for Apature Dark and Light, following the system appearance; egui widgets restyled through them | implemented, **verified** in screenshots |
| Native menu bar (Apature, File, Edit, View, Image, Window, Help) with accelerators; ⌘ shortcuts owned by the menu | implemented, **verified** via System Events (`Apple, File, Edit, View, Image, Window, Help`; View and Image items listed) |
| Toolbar 48 pt: sidebar toggle, workspace switcher (Caption/Ingest disabled with the milestone named), Filter and Sort disabled with reasons, size slider, decode counter, inspector toggle | implemented, verified in screenshots |
| Source-list sidebar: Favorites (add/remove), Recent (10), Drives & cards (/Volumes), Collections placeholder; 26 pt rows; resizable 200–320 with memory | implemented, verified in screenshots |
| Contact sheet: virtual grid, letterboxed cells, class bar, chip, stars, tag, pair badge, focus ring, ellipsised filenames; cells carry accessibility names | implemented, verified in screenshots; **VoiceOver not yet tested** |
| Inspector: sharp preview on top with honest size label, Capture / Marks (editable) / Caption (M4) / Files sections, resizable 280–420 with memory | implemented, verified (preview arrives at 1024 px in ~19 ms) |
| Filmstrip 88 pt (⇧F), Preview workspace with overlay and ±1 prefetch, status bar 28 pt | implemented, run manually |
| Panel memory and recent/favorites in `~/Library/Application Support/dev.mccal.Apature/prefs.json` | implemented, run manually |
| Reduce Motion read at startup → animation time 0 | implemented, not tested with the setting on |
| App icon (assets/icon.svg → icon-256.png, Apature.icns) as window/Dock icon | implemented; Dock shows it when launched; the app-menu title still reads as the launching process until an .app bundle exists (M6) |
| SF Pro / SF Mono at runtime | **not loaded**: the system variable fonts fail ab_glyph validation, so egui's bundled fonts are used (visible in screenshots) |
| Selection separate from focus: click, ⇧-range in view order, ⌘-toggle, ⌘-arrows keep, ⌘A/⌘⇧A; marks apply to the selection when the focused item is in it; "mixed" shown in the inspector | implemented, **unit-tested** (`selection.rs`), verified in a demo screenshot |
| Filters: rating ≥, hide rejected, classes, tag, pairs; chips in the toolbar; "showing x of y" | implemented, unit-tested (`filter.rs`), verified in screenshot |
| Sort: name, capture time, rating, class, reverse; EXIF read for all items at open | implemented, unit-tested |
| Undo/redo, one entry per action (⌘Z/⌘⇧Z, Edit menu) | implemented, unit-tested (`undo.rs`); writes files on undo |
| Advance after mark (Image menu toggle, shown in the status bar) | implemented |
| Caption workspace (⌘⇧3): photo left, fields right, assignment header, Load assignment / Apply pad to N / Insert template, inline `\code\` expansion with autocomplete hint, ⌘⏎ save and next, autosave on navigation, Revert | implemented, verified in a screenshot; writes verified via the CLI path |

## M4 notes (2026-10-14)

* Structured XMP (Alt/Seq/Bag) is edited with the same regex-based in-place
  approach as marks rather than the planned quick-xml port (ADR-011). It
  passed the same tests and exiftool validation; the port stays an option
  if a sidecar shape defeats it.
* IPTC edits have no undo yet (marks do). The draft's Revert covers the
  unsaved case only.
* IPTC-IIM (APP13) is not written into JPEGs; readers that ignore XMP will
  not see captions. Photo Mechanic, Lightroom, Capture One and exiftool
  read XMP first.
* The caption field is egui's TextEdit: no system spell-check or dictation.
  Watch for this being the reason to add a native text view.
* Photo Mechanic's display of our captions was not visually checked this
  session (exiftool read-back was); worth a ten-minute look.

## M3 notes (2026-10-14)

Batch writes are synchronous on the UI thread: marking 100 selected pairs
writes 200 files (~ms each). Acceptable for now; a background writer with a
progress row is the fix when it is felt. Workspace shortcuts moved to ⌘⇧1/⌘⇧2
because ⌘1–5 are Photo Mechanic's ratings.

## Previous probe (superseded)

| feature | status |
|---|---|
| Open folder (⌘O / button, native panel, CLI arg) | implemented, run manually |
| Virtual contact sheet, cell-size slider and +/− | implemented, run manually |
| Viewport-priority background decoding, texture eviction beyond 12 rows | implemented; eviction **not observed** under scrolling |
| Keyboard: arrows, Home/End, PgUp/PgDn, Enter/Space/Esc | implemented, run manually |
| Marks: 1–8 class, 0 clear, ⌘0–5 stars, X reject, T tag → sidecar | implemented; sidecar output verified via CLI path, GUI path run manually |
| Full-screen preview with ±1 prefetch, LRU of 6 | implemented, run manually |
| Status line with EXIF, marks, decode and frame stats | implemented |
| Multi-select, filters, sort options | **not implemented** |
| Menu bar, sidebar, inspector, light theme, VoiceOver names, Reduce Motion | **not implemented** (see `docs/design/MACOS_UI_REQUIREMENTS.md`) |

The probe could not be screenshotted; the M2 shell can (see above).

## Verification run (2026-10-13)

* `cargo fmt --check`: clean. `cargo clippy --all-targets`: 0 warnings.
* `cargo test`: 28 passed, 0 failed (one test skips when exiftool is absent;
  it ran here).
* `cargo build --release`: ok.
* Originals: the three NEF+JPG fixtures are size-identical to the Google
  Drive originals; the 114-file assignment folder gained no files and no
  sidecars; all access is read-only `mmap`. Byte hashes against the cloud
  originals were not computed (would re-download 165 MB).

## Known limitations and risks

* Cells larger than 160 pt are upscaled on Retina (thumbs are 320 px).
* Texture uploads happen on the UI thread; a 3 000 px preview is ~36 MB and
  may cause a frame hitch. Not measured.
* `Marks` are a single cursor's; there is no selection model, so batch
  marking is impossible.
* No undo.
* No colour management, no light theme, no accessibility names on cells.
* Font licence notices (Ubuntu, Noto) must ship with binaries; not generated.
* Toolchain: egui pinned to 0.33 (rustc 1.93 on this machine; 0.36 needs 1.95).

## Next priorities

1. ~~Photo Mechanic round trip~~ done, including pairs via embedded JPEG XMP.
2. Design decision from `docs/design/` (owner's call), then the shell:
   menu bar, sidebar, inspector, tokens, light/dark, cell accessibility.
3. Selection model + filters in the app state (no core change).
4. IPTC fields in `apature-metadata` with the same in-place discipline, on a
   streaming XML parser.
