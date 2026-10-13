# photo-workflow

A fast, keyboard-first contact sheet and culling tool for photojournalists,
written in Rust. Think of the ingest → cull → caption → deliver loop of Photo
Mechanic, rebuilt as an independent open-source application.

**Status: 0.1 engine proof of concept with a throwaway UI probe.** The
engine opens a folder of RAW/JPEG files, finds the camera-embedded previews
(no RAW decoding), decodes them with DCT scaling, caches thumbnails, and
records star ratings, colour classes and tags into XMP sidecars edited in
place so Photo Mechanic and Lightroom data survives. The egui window
exercises all of that but is not the product interface; see
[docs/STATUS.md](docs/STATUS.md) for exactly what is implemented, tested
and verified, and [docs/design/](docs/design/) for where the real UI is
being designed.

## Run it

```bash
cargo run --release -p photo-workflow -- /path/to/assignment
```

Or use the engine from the command line:

```bash
cargo run --release -p photo-workflow-cli -- previews DSC_0001.NEF   # what previews are embedded
cargo run --release -p photo-workflow-cli -- bench ./assignment --cache  # thumbnail throughput
cargo run --release -p photo-workflow-cli -- mark DSC_0001.NEF --rating 4 --class 1
```

Keys in the app (Photo Mechanic conventions): arrows move, Enter/Space opens
the full preview, Esc returns, `1`–`8` set a colour class, `0` clears it,
`⌘0`–`⌘5` set stars, `X` rejects, `T` toggles the tag, `+`/`-` resize cells,
`⌘O` opens a folder.

## How it is built

```
crates/
  photo-core/      file model, RAW+JPEG pairing, folder scan, marks, tiny TIFF reader
  photo-preview/   find embedded JPEGs in RAW files, decode with DCT downscaling, disk cache
  photo-metadata/  EXIF summary; XMP sidecar read/write that edits in place
  photo-ingest/    verified copy (temp name → hash both sides → rename), never deletes
apps/
  desktop/         egui/eframe contact sheet and preview
  cli/             `pw`: scan, previews, extract, exif, bench, mark, copy
docs/
  architecture/    design and the preview pipeline
  research/        how Photo Mechanic behaves, and how REA is used to study it
  benchmarks/      measured numbers and how to reproduce them
```

The one rule everything follows: **never decode a RAW to show a thumbnail
when the camera already embedded a JPEG.** See
[docs/architecture/overview.md](docs/architecture/overview.md),
[docs/DECISIONS.md](docs/DECISIONS.md) for why each piece is the way it is,
[docs/BRIEF.md](docs/BRIEF.md) for the standing brief, and
[docs/benchmarks/](docs/benchmarks/) for measured numbers.

## Compatibility promises

* Originals are never modified. Marks go to `name.xmp` next to the file.
* Sidecars written by other software are edited property-by-property, never
  regenerated, so captions, develop settings and anything else survive.
* `xmp:Rating`, `xmp:Label`, `photomechanic:ColorClass` and
  `photomechanic:Tagged` are written together, which is what Photo Mechanic
  and Lightroom each need to agree on a label.

## Requirements

Rust 1.80 or newer (`rustup update`). macOS is the development platform;
Windows and Linux should build but are untested.

## License

MIT OR Apache-2.0, at your option. See [LICENSE-MIT](LICENSE-MIT) and
[LICENSE-APACHE](LICENSE-APACHE).
