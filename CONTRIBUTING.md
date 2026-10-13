# Contributing

This started as one photographer's personal tool and is open source from day
one. Issues and pull requests are welcome, with a few ground rules.

## Principles that are not up for debate

1. **Originals are evidence.** No code path may modify, rename, move or delete
   a source image unless the user explicitly asked for that operation, and
   even then copies are verified before anything is removed.
2. **Metadata lives in standard places.** Ratings, labels, captions go into
   XMP/IPTC that other applications read. The app must be uninstallable
   without losing work.
3. **Speed is architecture, not polish.** Prefer the embedded preview, cache
   on disk, prioritise what is on screen, decode off the UI thread.
4. **Independent implementation.** Research into other applications informs
   behaviour and file-format compatibility; no proprietary code is copied.

## Workflow

```bash
cargo fmt
cargo clippy --all-targets
cargo test
```

Point `PW_FIXTURE_DIR` at a folder of real RAW files for manual testing; real
RAWs are not committed (see `.gitignore`). Benchmarks go in
`docs/benchmarks/` with the machine, the folder size and the exact command.

## Licensing

By contributing you agree your work is licensed MIT OR Apache-2.0 like the
rest of the project. Do not add GPL-licensed dependencies or code.
