# Plan for the next sessions (written 2026-10-15, budget-aware)

## Handoff 2026-10-15 evening (for the next model; Fable budget nearly spent)
- Cloud PRs #1 (ingest companions, preserve folders, CLI timing, CI) and #2
  (rotate + crop in the sidecar, applied on export) are MERGED into main and
  build clean on the Mac: 84 tests, clippy 0.
- Fixed on the Mac: `sidecar_path` now resolves the real on-disk case (the
  cloud test failed on APFS because `.xmp` and `.XMP` both probe true).
- Verified on the Mac: companions + preserve-folders CLI ingest on a scratch
  card (WAV landed as Party001.WAV under DCIM/100NIKON); `timing:` line prints;
  `apature-cli rotate` writes tiff:Orientation=6 and the sheet thumbnail turns;
  persistent {seqn} continued at 109 after the owner's Oct 08 "Dragons Den"
  ingest; Ingest at 1000x650 had the Seq. start row clipped (fixed: wraps).
- NOT verified: real-card ingest with both new checkboxes (card was ejected);
  the 46 s vs 125 s gap on the real card (timing line is in, just run it);
  Lightroom/PM reading our rotate; the on-screen crop tool does not exist yet
  (CLI `crop --rect` only); Ingest window headless capture.
- 2026-10-15 night: template engine gained Photo Mechanic's camera/time/weekday/
  folder/today variables (see template.rs header; 85 tests). One chip style
  (`icons::chip`) across Ingest, Export; export hint no longer clips.
  Number keys pref (1–5 rate by default, PM colour-class mode), Settings chips
  and bigger window, Caption: More fields fold, Save to N selected, ▾ recent
  values (PM IPTC Info parity; still missing: variables expanding inside
  fields, snapshots/presets, Prev/Next without saving).
  Sheet-while-ingesting done 2026-10-19 (tab opens at first verified file,
  refresh every 25 + at end, ⌘R). Not yet seen on a real card.
  Toolbar workspace glyphs done. Polish pass still open: inspector density,
  empty states (Fen). Caption at 1000x650 checked: no overlap. Then: sheet-while-ingesting, burst
  stacks, day chips on the sheet, FTP delivery.
- 2026-10-19: apature.log (Application Support) added; audit of gaps in chat → top three: delivery, real-card backup/cancel/timing run, log (done).
- 2026-10-20: inspector density pass; export presets per outlet (prefs.export_presets). Remaining fine-tune: thumbnail priority on big folders, focus rings. Features next: keywords/GPS, crop tool, loupe.
- 2026-10-20: unplug detection (status + sticky toast + tab banner + ingest warning), toasts module (ui/toasts.rs). Next fine-tuning: inspector density, thumbnail priority on big folders, focus rings, export presets per outlet, then keywords/GPS, crop tool, loupe.
- 2026-10-19 night: JPEG XL export (owner request): Format::JpegXl in apature-export transcodes the finished JPEG losslessly with cjxl (Homebrew libjxl), EXIF+XMP carried, ~21 % smaller; Export sheet FORMAT chips, CLI --jxl. Not bundled: needs cjxl on the Mac (sips and ImageIO on macOS 27.2 cannot write JXL; jpegxl-rs vendored needs cmake). Later: link libjxl into the app and bundle the dylib.
- 2026-10-19 evening: FTP/FTPS/SFTP delivery shipped (see item 9). Untested: FTPS, SFTP, Settings › Delivery page rendering (headless settings capture hangs), real desk.
- Owner's operating rules for Claude sessions are in memory
  (`claude-agent-efficiency-rules`): no subagents unless independent, cheaper
  models for routine work, no repeated research, concise reports.


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

- Design system artifact (2026-10-19): https://claude.ai/artifact/KKoM7Em7u3hrLtBHt3ouCu, extracted from tokens.rs/icons.rs. Font decided: IBM Plex Sans + IBM Plex Mono. TODO: embed the Plex files in the desktop crate (egui font definitions), add licence to THIRD_PARTY.md, list them under fonts/ in the design system.

## Status 2026-10-20 (night)
Live Ingest shipped (polling + 2-poll size stability; Live chip on the Ingest
sheet; verified with dropped NEFs). CI: Linux fixed, macOS desktop job added.
Focus rings. Left for Caleb: client IDs, FTP details, Developer ID, notary,
real-card run. Left for Claude: AF point (maker note), Plex embed, VoiceOver,
thumbnail priority, Fen, caption slice two, release engineering.

## Status 2026-10-20 (evening)
Done today: unplug detection, toasts, inspector pass, export presets, PM-board
quick wins (codes switch, big tag badge, B exposure boost, {dest}), delivery
queue (retry+reconnect, delivered.json ledger + Sent row, resend), Folder
delivery kind (shares and Google Drive/OneDrive/Dropbox sync folders, no
sign-in), keywords chips + GPS in Caption, set-GPS-on-ingest. Cloud APIs
(Drive/Graph with OAuth) still wait on client IDs. Focus rings on painter-drawn controls 2026-10-20. AF point overlay: NEF keeps the AF point in the Nikon maker note (AFInfo2), not in standard EXIF; needs a maker-note parser (half a day) — deferred. Crop tool shipped 2026-10-20 (K in Preview; AF point still to do: needs
SubjectArea/AFPoint in ExifSummary). Next: loupe with unified zoom, burst stacks.

## Re-ordered after reading Camera Bits' feature-request board (2026-10-20)
See docs/research/pm-feature-gap.md §5. Quick wins with real demand first:
a. Code-replacement on/off switch in Caption; bigger tag badge on thumbnails;
   honour in-camera ratings on ingest (sort/filter by them); view-only
   exposure boost in Preview; `{dest}` variable.
b. Delivery queue: retry, resend, persistent per-file sent status shown next
   to the caption; then Google Drive / OneDrive.
c. Keywords + GPS (incl. set GPS on ingest), then crop tool (ratios,
   golden-section overlay, AF point), loupe with unified zoom.
d. Burst stacks + sharpness hint as the honest answer to "AI culling".

## Order of work, sized for budget

Small (each fits in a slice of a session)
1. ~~Preserve the card's folder structure (option) + copy WAV/XMP companions.~~ Done 2026-10-15 (cloud session); still to do on the Mac: one run against the real card.
2. ~~Open the contact sheet while the ingest runs~~ done 2026-10-19.
3. ~~CLI timing instrumentation.~~ Done 2026-10-15: `apature ingest` prints a `timing:` line (ledger load, plan, copy+verify+save, total) to stderr. Re-run the 134-NEF card on the Mac to find the missing ~80 s.
4. Export hint, stale roadmap, performance numbers on the 839-photo day.
5. ~~CI workflow file.~~ Done 2026-10-15: `.github/workflows/ci.yml`, Linux only (fmt, clippy, test) to keep runner minutes cheap.

Medium (half a session each)
6. Lossless rotate (orientation tag in XMP/EXIF) and crop stored in XMP, applied on export.
7. Keywords panel and GPS fields in Caption; sRGB conversion on export; export presets per outlet (Eagle / TribLive).
8. Fen wiring: empty state, About, toasts, "Show Fen" setting, Fen app icon; name check.

Large (a full session each; start them fresh)
9. Delivery after export (owner, 2026-10-19): ~~FTP/SFTP~~ done 2026-10-19 (apature-deliver crate; Settings › Delivery; Export 'Then send to'; CLI deliver; FTP verified end to end against a local server, FTPS and SFTP compiled but untested against a real host) **and** cloud drives with a
   proper sign-in: Google Drive (OAuth 2.0 + PKCE, loopback redirect, Drive v3
   resumable upload) and OneDrive / SharePoint (Microsoft identity platform,
   MSAL-style PKCE, Graph `createUploadSession`). Refresh tokens in the macOS
   Keychain (`security-framework`), never in prefs. One `Destination` enum
   {Ftp, Sftp, GoogleDrive, OneDrive} with per-outlet presets, a "Send to desk"
   command, retry and a per-file sent ledger. Needs app registrations (Google
   Cloud OAuth client, Entra app) that Caleb creates; the client IDs ship in
   the app, secrets are never needed with PKCE. Full milestone; start fresh.
10. Live Ingest: watched folder with file-stability check; the camera-FTP receiver.
11. VoiceOver pass with AccessKit, full keyboard audit, ACCESSIBILITY_CHECKLIST.md walked line by line.
12. Release engineering: Developer ID signing, notarisation, DMG, update check.
13. Installers for macOS and Windows: DMG + Windows MSI/.exe built by a tag-triggered CI job, attached to a GitHub release. Depends on 12 for the Mac side and a Windows build pass for the other.

Rules that apply to all of it: easy to read, nothing overlapping (check with
`--size 1000x650` screenshots), Reduce Motion honoured, every claim measured.
