# Benchmarks

Reproduce with:

```bash
cargo run --release -p photo-workflow-cli -- bench <folder> --size 320          # cold
cargo run --release -p photo-workflow-cli -- bench <folder> --size 320 --cache  # warm after a first run
```

Record machine, OS, folder contents (camera, count, RAW/JPEG mix), whether
the files were on internal SSD or a card, and the full command. Numbers go
in dated files in this folder.

## Targets (goals, not results)

| metric | target |
|---|---|
| files in a stress-test folder | 10 000 |
| cached-thumbnail navigation latency | < 50 ms |
| UI frame time while scrolling | 16 ms (60 fps) |
| source files modified during a cull | 0 |
