# Plan for the next sessions (written 2026-10-15, budget-aware)

State: M1–M10 done, first real shoot (Ryan's birthday, 134 NEF) ingested in the
app and verified. Ingest is its own window. Persistent `{seqn}`, Accessibility
settings, drive icons, day chips, live example name, MB/s all shipped today.
Owner's parity order is in ROADMAP.md ("Requested 2026-10-15").

## What is still missing (found by sweep, not yet on the list)

Workflow gaps
- Secondary (backup) destination never tested on a real drive; cancel mid-ingest never tested on real hardware.
- Copy safety on power loss: confirmed 2026-10-15, `copy.rs` writes to a temporary name, hashes it, then renames; a dead copy leaves only a temp file.
- Opening a big day (Aug 30, 839 photos) from the card: measure scan, thumbnail and memory; the 2,400-photo card root too.
- CLI ingest timing: run reported 45.9 s, wall clock 125 s; instrument plan / copy / ledger-save / exit.
- Export naming hint clips at narrow widths; shorten the hint.
- HEIC and CR3 previews (not for the Z 6 II; low).
- Thumbnail cache: size cap and eviction for multi-thousand-photo folders (verify what exists).

Product and release
- VoiceOver: egui's AccessKit integration is not enabled (check `accesskit` feature in eframe); labels for every control, focus order in the Ingest window.
- CI: no GitHub Actions yet (cargo test + clippy + fmt on push).
- Release: code signing with a Developer ID, notarisation, a DMG, and an update check. Today's bundle is ad-hoc signed.
- Installers (owner, 2026-10-15): macOS DMG and a Windows installer, built by CI from a tag. Windows needs the macOS-only bits (volumes, eject, `sips` RAW decode, menu bar) behind platform switches first; a Windows code-signing certificate is a cost to plan for.
- README for the public repo, licence file check (THIRD_PARTY.md exists), screenshots.
- Fen: name availability unverified; placeholder not wired into the app (empty state, About, "Show Fen" switch); app icon still the tile, not Fen; artist brief not sent.
- Test harness: `--settings --screenshot` hangs (viewport screenshot never arrives); the Ingest window has no headless capture either. Fix once, both use the same mechanism.

Observed once, unexplained
- The first Ingest window opened via File › Ingest disappeared and the main window was minimised. Watch for it; if it repeats, log viewport events.

## Order of work, sized for budget

Small (each fits in a slice of a session)
1. ~~Preserve the card's folder structure (option) + copy WAV/XMP companions.~~ Done 2026-10-15 (cloud session); still to do on the Mac: one run against the real card.
2. Open the contact sheet while the ingest runs: open the destination tab after the first verified file, keep ingesting; the window stays for progress.
3. ~~CLI timing instrumentation.~~ Done 2026-10-15: `apature ingest` prints a `timing:` line (ledger load, plan, copy+verify+save, total) to stderr. Re-run the 134-NEF card on the Mac to find the missing ~80 s.
4. Export hint, stale roadmap, performance numbers on the 839-photo day.
5. ~~CI workflow file.~~ Done 2026-10-15: `.github/workflows/ci.yml`, Linux only (fmt, clippy, test) to keep runner minutes cheap.

Medium (half a session each)
6. Lossless rotate (orientation tag in XMP/EXIF) and crop stored in XMP, applied on export.
7. Keywords panel and GPS fields in Caption; sRGB conversion on export; export presets per outlet (Eagle / TribLive).
8. Fen wiring: empty state, About, toasts, "Show Fen" setting, Fen app icon; name check.

Large (a full session each; start them fresh)
9. FTP / SFTP delivery after export (keychain-stored credentials, retry, per-outlet target).
10. Live Ingest: watched folder with file-stability check; the camera-FTP receiver.
11. VoiceOver pass with AccessKit, full keyboard audit, ACCESSIBILITY_CHECKLIST.md walked line by line.
12. Release engineering: Developer ID signing, notarisation, DMG, update check.
13. Installers for macOS and Windows: DMG + Windows MSI/.exe built by a tag-triggered CI job, attached to a GitHub release. Depends on 12 for the Mac side and a Windows build pass for the other.

Rules that apply to all of it: easy to read, nothing overlapping (check with
`--size 1000x650` screenshots), Reduce Motion honoured, every claim measured.
