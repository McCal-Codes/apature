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
