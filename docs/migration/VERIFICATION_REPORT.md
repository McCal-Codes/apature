# Verification report

One row per milestone. "Verified" means run and observed; nothing here is
inferred from reading code.

| date | milestone | fmt/clippy | tests | release build | external checks |
|---|---|---|---|---|---|
| 2026-10-13 | foundation (pre-rename, `efe5c65`) | clean / 0 warnings | 36 pass | ok | exiftool validate; Photo Mechanic 2026.6 both directions, NEF-only and pairs; Capture One 16.8 both directions; NEF/JPEG hashes identical |
| 2026-10-13 | M1 consolidate | pending | pending | pending | app opens a folder under the new name; cache repopulates under `dev.mccal.apature` |
