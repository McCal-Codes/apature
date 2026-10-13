# Status

Last updated 2026-10-13. Vocabulary: **implemented** = code exists;
**tested** = automated test passes or a manual run is recorded here;
**verified** = tested against an external reference (exiftool, another
app, a measurement); **production-ready** is not claimed for anything.

## Engine

| feature | status | evidence |
|---|---|---|
| Folder scan, RAW+JPEG pairing, sort | tested | unit tests; `pw scan` on 3 folders |
| TIFF/IFD reader | tested | unit tests incl. hostile input |
| Embedded preview discovery (IFD chain, SubIFDs, Exif IFD, Nikon MakerNote PreviewIFD, plain JPEG) | **verified** | identical previews and byte counts to exiftool on D850 NEF; synthetic TIFF tests |
| JPEG decode with DCT scaling, orientation, thumbnail | tested | round-trip tests; D850 NEF/JPG runs |
| Disk thumbnail cache (key: path, size, mtime, size) | tested | unit test; bench warm runs |
| EXIF summary (make/model/lens/date/exposure/orientation) from TIFF and JPEG APP1 | **verified** | matches exiftool on D850 NEF |
| XMP sidecar read/write, in-place, atomic, read-back checked | **verified** | exiftool; **Photo Mechanic 2026.6 round trip both directions** (NEF-only folders); see `docs/research/metadata-safety.md` |
| Embedded-XMP read **and write** in JPEG, pair policy (sidecar + embedded), conflict report | **verified** | unit tests; exiftool on a D850 JPEG; Photo Mechanic shows embedded marks for pairs |
| Verified copy (temp → SHA-256 both sides → rename; never overwrite; never delete source) | tested | unit test only; no card-reader run |
| CR3 / HEIC / RAF-with-no-TIFF previews | **not implemented** | returns "no embedded JPEG" |
| RAW decode fallback when no preview | **not implemented** | |
| ICC colour management | **not implemented** | previews shown as sRGB |
| IPTC fields, captions, code replacements, templates | **not implemented** | Phase 3 |
| Ingest planning, renaming, dual destinations, recovery | **not implemented** | Phase 4; only `copy_verified` exists |
| Catalog | not started | Phase 4+ |

## Desktop prototype (`apps/desktop`, see ADR-000: a probe, not the product UI)

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

Visual review of the running window was **not possible** in this session
(unbundled binary; screen-capture permission unavailable to the shell).
Correct rendering is inferred from the thumbnail cache being populated by
the GUI and from no panics in 10+ minutes of running; it is not verified.

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
4. IPTC fields in `photo-metadata` with the same in-place discipline, on a
   streaming XML parser.
