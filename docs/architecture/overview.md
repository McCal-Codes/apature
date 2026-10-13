# Architecture

## Crate layout

| crate | owns | must not |
|---|---|---|
| `apature-core` | `PhotoItem`/`Variant` model, RAW+JPEG pairing, folder scan, `Marks`, TIFF/IFD reader | touch pixels, know about the UI |
| `apature-preview` | embedded-JPEG discovery, JPEG decode with DCT scaling, orientation, thumbnailing, disk cache | parse metadata beyond what it needs for preview offsets |
| `apature-metadata` | EXIF summary, XMP sidecar read/write | decode images |
| `apature-ingest` | verified copy | delete anything |
| `apps/desktop` | egui window, background pool, keyboard model | file-format knowledge |
| `apps/cli` | same engine, no window; the benchmarking surface | — |

Crates only depend downwards. The UI is replaceable: nothing in `crates/`
imports egui.

## The preview pipeline

```
open folder
  └─ scan (walkdir, non-recursive, pair by stem) ─────────── ~0.5 ms / 100 files
       └─ read sidecars in parallel (rayon) ───────────────── small text files
            └─ UI shows the grid immediately, cells empty
                 └─ each frame: request thumbs for visible rows ± 3,
                    priority = row distance from viewport
                      └─ worker (N-1 threads):
                           disk cache hit?  → decode cached 320px JPEG
                           else mmap file → walk IFDs → pick smallest
                                embedded JPEG ≥ 2×thumb size → DCT-scaled
                                decode → rotate → box-filter → cache → UI
                 └─ textures further than 12 rows away are dropped
```

Why each step:

* **mmap, not read.** A 55 MB NEF costs a few page faults to index; the
  preview bytes are the only part ever paged in.
* **Smallest sufficient candidate.** A Nikon D850 NEF carries previews at
  8256×5504, 1620×1080 and 640×424. Thumbnails use the 1620 one, so the
  decoder never touches the 3 MB full-size JPEG until you open the preview.
* **DCT scaling.** `jpeg-decoder` can decode at 1/2, 1/4, 1/8 size directly,
  so a 320 px thumb from a 1620 px preview is a 1/4-scale decode.
* **Priority queue keyed by viewport distance.** Scrolling re-prioritises
  rather than appends, so the rows under the cursor always win.
* **Bounded GPU memory.** Textures are evicted outside a window of rows and
  come back from the disk cache in a few milliseconds.

## Marks and sidecars

In-memory `Marks { rating, color_class, tagged }` are the source of truth
while the folder is open; every change is written straight to `name.xmp`
with an atomic rename. Reading accepts attribute or element form and both
the `xmp:` and legacy `xap:` prefixes. Writing edits the first
`rdf:Description` in place, so a Lightroom sidecar keeps its `crs:` settings
and a Photo Mechanic sidecar keeps its captions.

Colour-class names must match the Photo Mechanic preferences on the machine
for `xmp:Label` to round-trip exactly; the numeric
`photomechanic:ColorClass` is always written too and is what we trust when
reading.

## Not yet

* IPTC captions, code replacements, stationery (Phase 2)
* Ingest UI, renaming templates (Phase 2; `apature-ingest::copy_verified` exists)
* HEIC/CR3 previews (not TIFF containers; need a different locator)
* Multi-select, filtering by marks, sort orders
* Catalog database (Phase 4)
