# Feature matrix: original Apature vs the Rust foundation

Status words: done = implemented and tested; partial = implemented, gaps
noted; none = absent. "Decision" says which implementation the
consolidated Apature keeps.

| feature | Apature (Tauri) | foundation (this repo) | decision |
|---|---|---|---|
| folder scan, pairing | done (12k files/s, stack by basename) | done (walkdir, pair by stem) | foundation; add incremental scan events later |
| embedded RAW previews | done via rawler (LGPL), orientation bug suspected | done, own TIFF walker, orientation applied, verified vs exiftool | foundation |
| thumbnail cache | done, provenance in filename, 2 GiB trim, pipeline version | done, key = path+size+mtime+size, no trim | foundation + port trim and provenance |
| colour management | done (moxcms, after downscale) | none | port the approach; pick crate (moxcms or lcms2) |
| EXIF summary | kamadak-exif + rawler | own walker (TIFF + JPEG APP1) | foundation |
| XMP ratings/labels | done, catalog wins over sidecar | done, sidecar+embedded, label beats stale class; verified in PM and C1 | foundation |
| IPTC fields | done (dc, photoshop, Iptc4xmpCore, xmpRights, crs crop) | none | port |
| JPEG embedding | export only (APP1 + APP13 IIM) | done for marks (APP1) | foundation + port the IIM writer for exports |
| exiftool engine | present, unreachable | none | drop |
| keyboard culling | done (React), focus ≠ selection, shortcut guard | partial (cursor only) | adapt rules into Rust |
| undo/redo | done (200 deep, data entries) | none | adapt |
| ingest plan/run | done (templates, collisions, incremental, 2 destinations, erase gated) | copy_verified only | port |
| copy/move/trash | done (copy-verify-remove, system trash, removable gate) | none | port |
| catalog (SQLite) | done (v1–v5 migrations, operation log) | none | defer, then port |
| RAW develop/export | uncommitted (rawler, bilinear X-Trans) | none | defer |
| histogram | done (sensor data) | none | defer |
| UI | React, accessible grid, named contact sheets, crop guide | egui probe | new egui shell per design docs |
| tests | ~135 unit + integration + fuzz + benches | 36 unit + exiftool/PM/C1 manual | port tests with each subsystem |
