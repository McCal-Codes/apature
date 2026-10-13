# Roadmap

## 0.1 — fast browser (this repo today)
- [x] folder scan with RAW+JPEG pairing
- [x] embedded-preview thumbnails, DCT-scaled decode, disk cache
- [x] viewport-prioritised background decoding, bounded GPU memory
- [x] keyboard culling: stars, colour class, tag, reject
- [x] XMP sidecars written in place, PM/Lightroom compatible
- [x] full-screen preview with prefetch of neighbours
- [x] `apature-cli` CLI and benchmarks
- [x] benchmarks recorded (docs/benchmarks/2026-10-13-m1pro.md); a real 1 000-NEF assignment is still outstanding
- [ ] multi-select and shift-range marking
- [ ] filter the sheet by rating / class / tag
- [ ] HEIC and CR3 previews

## 0.2 — ingest and metadata
- [ ] IPTC stationery: headline, caption, byline, credit, copyright, location, keywords
- [ ] code replacements and `{variables}` compatible with the existing Butler Eagle files
- [ ] caption editor with the "type subject, append saved tail" flow
- [ ] ingest UI on top of `apature-ingest`: card detection, two destinations, rename template, report
- [ ] write IPTC into JPEG exports (sidecar stays the source for RAW)

## 0.3 — professional workflow
- [ ] export presets (long side, quality, sRGB, keep metadata, naming)
- [ ] saved assignment presets (the `generator/` text files become first-class)
- [ ] folder watching for live ingest
- [ ] Windows and Linux builds

## 0.4 — catalogs
- [ ] SQLite index across drives, metadata search, collections
