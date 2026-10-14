# Verification report

One row per milestone. "Verified" means run and observed; nothing here is
inferred from reading code.

| date | milestone | fmt/clippy | tests | release build | external checks |
|---|---|---|---|---|---|
| 2026-10-13 | foundation (pre-rename, `efe5c65`) | clean / 0 warnings | 36 pass | ok | exiftool validate; Photo Mechanic 2026.6 both directions, NEF-only and pairs; Capture One 16.8 both directions; NEF/JPEG hashes identical |
| 2026-10-13 | M1 consolidate (`~/dev/apature`) | clean / 0 warnings | 36 pass | ok (`apature`, `apature-cli`) | `apature <folder>` opened the 114-file assignment and wrote 114 thumbnails under `~/Library/Caches/dev.mccal.Apature` |
| 2026-10-13 | M2 shell (branch m2-shell → main) | clean / 0 warnings | 36 pass | ok | screenshots of Dark and Light with the 114-file assignment; native menu listed via System Events; inspector preview 1024 px in 19 ms; frame 8 ms idle |
| 2026-10-14 | M3 cull (branch m3-cull → main) | clean / 0 warnings | 43 pass | ok | demo screenshot: 3 selected, selection rings, filter popup, inspector shows mixed class and 3 selected; light screenshot of the 114-file sheet with Filter/Sort live |
| 2026-10-14 | M4 captions (branch m4-captions → main) | clean / 0 warnings | 52 pass | ok | Caption workspace screenshot with the golf assignment loaded; `apature-cli pad` + `caption` on a scratch pair: exiftool validates sidecar and JPEG, reads Headline/Description/Credit/City/Location/DateCreated, Rating and ColorClass intact, NEF hash identical |
| 2026-10-14 | M5 ingest (branch m5-ingest → main) | clean / 0 warnings | 60+ pass | ok | CLI ingest of 4 files from a fake card tree to two destinations with rename: all verified, re-run skipped by ledger, card hashes unchanged; Ingest workspace screenshot (plan state) |
| 2026-10-14 | M6a tabs + settings (branch m6-tabs-settings → main) | clean / 0 warnings | 59 pass | ok | screenshot with three tabs open and the inspector reading real captions; the Settings window ran 240 frames without a panic but could not be screenshotted (egui child viewports do not answer the screenshot command), so its layout is unreviewed; tabs restored across a relaunch not yet observed |
| 2026-10-14 | M6b bundle | n/a | n/a | ok | `scripts/bundle.sh` produced a signed `Apature.app`; launched via `open`, menu bar listed with the Apature title, bundle identifier confirmed, quit via AppleScript |
| 2026-10-14 | M6c animations | clean / 0 warnings | 60 pass | ok | runs 200 frames with two tabs without a panic; motion itself not reviewed visually |
