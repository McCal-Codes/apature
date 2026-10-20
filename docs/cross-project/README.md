# Folio ↔ Apature: cross-project engineering audit

Audited 2026-10-20. Folio at `main` 0b5d80b (shallow clone, read-only).
Apature at `main` 9efd087. Everything below was read from files, not
descriptions; where a claim is an inference it says so. Nothing in either
repository was changed by this audit; this folder lives on the
`cross-project-learning` branch and is a proposal.

## A. Folio engineering assessment

**Strongest practices (evidence)**

- Standards with rule ids and a "where Folio is today / gaps" table in every
  one (`docs/standards/*.md`, 14 documents). The gaps tables are honest and
  sized (S/M). This is the single best habit in either repo.
- Threat model with a test per threat (`docs/sdk/threat-model.md`, T1–T24,
  each row names `RepoClientTest`/`PackageInstallerTest`/…). Packages are
  parsed in memory, never extracted (T7); scripts are refused outright (T10).
- Parser fuzzing: Jazzer over 12 parsers, weekly in CI and on demand
  (`.github/workflows/ci.yml` `fuzz` job, `ParserFuzzTest`), failing inputs
  kept as artifacts.
- Saved-format discipline: `STATE_SCHEMA = 11`, `stateSchemaFor` writes the
  lowest schema the data needs so older builds keep reading, a one-time backup
  per migration step, a `state_damaged_backup` on an unreadable save with a
  visible message (STA-5/6/7, `LauncherModel.kt:1567–1596`, `LayoutLoadTest`).
- Capability model: `FolioCapability` with a privilege tier and risk,
  `CapabilityProvider`, and `UnavailableReason` ("always a state, never an
  error") (`CapabilityBroker.kt:11–45`). Safe Mode after two crashes within
  60 s of a package change (T9, `ActionRegistry.kt:63`).
- Release engineering enforced by a machine: `release-rules` CI job checks
  the branch, changelog line and version bump on every PR; `VersionCodeTest`
  checks the packaged manifest; SHA-pinned actions; secrets scan first.
- `docs/in-flight.md`: who holds which shared file, written after the 0.6.7
  three-way edits. Cheap and effective.
- Test volume: 89 JVM test files (~476 tests), 41 androidTest files (~120),
  :market 144 tests; `ScreenCoverageTest` sweeps thousands of window sizes.

**Actual deficiencies (from Folio's own gaps tables plus code)**

- No timing benchmarks: `FrameTimingMetric`/`StartupTimingMetric` appear
  nowhere (grep: 0 hits); one journey in the baseline profile. Performance
  targets are still "proposed".
- Instrumented tests compile in CI but do not run; no screenshot tests; no
  accessibility checks in UI tests (`enableAccessibilityChecks` absent).
- No lint baseline; 148 warnings.
- Layout state lives in SharedPreferences as one JSON string
  (`prefs.edit().putString("state", …)`). Correct for its size; it means no
  independent tool can validate the saved format, and the damaged-backup path
  is the only recovery.
- Layout Backup is a portable description (STA-11), but nothing verifies a
  backup against an independent reader, and there is no round-trip test
  against a *previous release's* backup fixture beyond `LayoutLoadTest`'s
  schema steps.

**What Folio could take from Apature**

- Verified exports: Apature hashes every copy at every destination
  (`crates/apature-ingest/src/copy.rs`, SHA-256 both sides, temp name then
  rename). For Folio the analogue is cheap: after writing a Layout Backup,
  re-read and re-parse it before saying "saved", and keep the previous backup
  until the new one parses. No hashing needed.
- Round-trip fixtures against an external reference: Apature keeps real
  Photo Mechanic sidecars and validates its XMP/IIM output with exiftool
  (`docs/STATUS.md` "Photo Mechanic round trip"). Folio's equivalent is a
  fixture folder of backups from each shipped version, loaded in a test.
- Measurements in the status doc: Apature records numbers (scan ms, thumb
  ms, ingest MB/s, 125 s vs 46 s discrepancies) next to the feature. Folio's
  "where today" sections are qualitative; adding the Fold8 numbers that
  already exist in `DuoMotionTrace` logs costs nothing.
- Bounded, prioritised background work: Apature's thumbnail pool clears
  pending work on folder change (`self.pool.clear_pending()`) and limits the
  texture budget (768 MB). Folio's package-change refresh loads every app's
  name (0.45 s on the Fold8, its own gap 7); the same "cancel what is no
  longer visible" rule applies.

## B. Apature engineering assessment

**Strongest practices (evidence)**

- Data integrity on the file path: temp-write-hash-rename for ingest copies
  and exports (`copy.rs`, `apature-export/src/lib.rs` atomic write), rename
  with rollback (`fileops.rs:110–150`), `.part` then rename for delivery, the
  card never written, sidecars the only edit target.
- External validation: exiftool and Photo Mechanic round trips on real
  files; the same-card ingest verified 134 NEF byte-identical.
- Headless verification harness: `--screenshot`, `--ingest-run`,
  `--export-run`, `--size`, `--crop`, `--live` flags; most features in this
  repo were accepted from a screenshot.
- Design tokens as code (`ui/tokens.rs`) with a design system extracted
  from them; Reduce Motion honoured; one chip style; focus rings.
- 90 tests across crates, clippy clean, CI on Linux plus a macOS desktop job.

**Technical gaps**

- `docs/STATUS.md` is stale by about a week of work: it still lists "no
  selection model, no undo, no light theme, no accessibility names" as
  limitations and its "next priorities" are M3-era. The living status
  document stopped living; PLAN.md took over informally.
- `docs/DECISIONS.md` stops at ADR-018. Undocumented decisions since:
  Ingest as a separate window, delivery with Keychain secrets, toasts,
  Live Ingest by polling, lavender in both themes, IBM Plex, JPEG XL via an
  external cjxl, number-key mode, burst stacks, the `Folder` delivery kind.
- No fuzzing of parsers (EXIF, XMP, IIM, TIFF, sidecar, JPEG segments) and
  no threat model, although Apature parses untrusted camera and desk files
  and now talks to FTP/SFTP servers.
- Prefs have no schema version and no damaged-prefs backup; a rename of a
  field silently drops the user's settings (serde default).
- No criterion benchmarks; `docs/benchmarks` is a hand-recorded page.
- Known-and-accepted: one commit pushed without compiling (fixed next
  commit, CI now catches it); a dependency slipped under the macOS-only
  table and broke Linux CI for a day.
- Capability handling is ad hoc: `cjxl` present or not, `sips` present or
  not, Keychain present or not are each an `Option` checked at the call
  site, with no single "what can this Mac do" view for the user.

**What Apature could take from Folio**

- Rule ids plus "where we are / gaps" tables per standard. Apature has the
  prose (`docs/design/*.md`, `ACCESSIBILITY_CHECKLIST.md`) but no ids, so
  nothing can cite a rule, and no gaps tables, so STATUS rots.
- Threat model with a test per row, adapted: untrusted inputs are camera
  files, sidecars, desk servers, prefs and the ledger; threats are parser
  crashes, path traversal in templates, a hostile FTP server, Keychain
  misuse, a card yanked mid-copy.
- Parser fuzzing: `cargo fuzz` targets for the six parsers, seeded with the
  fixtures, weekly in CI like Folio's.
- Versioned prefs with migration and a damaged backup (STA-5/6/7).
- A capability table: one `Capabilities` struct computed at start
  (RAW engine, cjxl, Keychain, Reduce Motion, delivery kinds), shown in
  Settings and logged, with `UnavailableReason`-style states.
- Release rules in CI: a changelog line per user-visible change, version
  bump on its own, tag → bundle → notarise, like `release-rules`.
- `in-flight.md` is not needed yet (one session at a time), but the habit
  of a per-branch handoff already exists in PLAN.md.

### Apature Pocket (proposed embedded receiver)

Not in the repository yet; nothing to audit. The transferable Folio
patterns for it: a capability broker over USB/PTP (what this camera can do,
measured, with unavailable-as-state), a mock PTP backend so the state machine
is tested without a camera, Safe Mode after repeated crashes, versioned
on-device configuration with a damaged backup, and resource limits on every
parser that reads from the camera. From Apature: the verified-copy ledger and
the "never write to the source" rule carry over unchanged.

## C. Cross-project learning matrix

| # | From → To | Existing implementation | Problem | Proposed adaptation | Size | Risk | Evidence | Priority | Verdict |
|---|---|---|---|---|---|---|---|---|---|
| 1 | Folio → Apature | `docs/standards/*.md` rule ids + gaps tables | STATUS.md rotted; design docs can't be cited | Add ids (DAT-, UI-, A11Y-, REL-) and a gaps table to the four Apature standards; retire STATUS "limitations" into them | S | none | this audit | now | worth adopting |
| 2 | Folio → Apature | `LauncherModel.kt` STATE_SCHEMA, `state_damaged_backup`, `LayoutLoadTest` | prefs.json unversioned; a bad write loses settings silently | `prefs.version`, backup before first write after an upgrade, keep `prefs.damaged.json` on parse failure, one load test per version | S | low | Folio STA-5/6/7 | now | worth adopting (see F) |
| 3 | Folio → Apature | `ParserFuzzTest`, weekly `fuzz` job | six untrusted-input parsers never fuzzed | `cargo fuzz` targets: exif, tiff, xmp, iim, jpeg_xmp, sidecar edits; seeded from fixtures; weekly CI | M | low | T8 in Folio found real crashes (per its tests) | next | worth adopting |
| 4 | Folio → Apature | `docs/sdk/threat-model.md` | no threat model while adding network delivery | `docs/THREAT_MODEL.md`: assets, boundaries, T-rows each naming a test | S | none | — | next | worth adopting |
| 5 | Folio → Apature | `CapabilityBroker.kt`, `UnavailableReason` | cjxl/sips/Keychain checked ad hoc | one `Capabilities` struct at startup, Settings page, log line | S | low | — | next | worth adopting |
| 6 | Folio → Apature | `release-rules` job, `VersionCodeTest` | no release gate; version in Cargo.toml and bundle differ by hand | CI check: CHANGELOG entry per PR, version bump alone, tag builds bundle | M | low | Folio 0.6.7 lesson | later (after Developer ID) | worth adopting |
| 7 | Folio → Apature | Compose render tests at window sizes (`ScreenCoverageTest`) | layout checked by eye at two sizes | turn the `--size` screenshot sweep into a CI job that diffs against stored PNGs | M | flaky on font rendering | this repo's sweeps | later | needs investigation |
| 8 | Apature → Folio | `copy.rs` verify-then-rename; export re-read | Layout Backup says "saved" without re-reading | parse the written backup before confirming; keep the previous one until then | S | none | STA-11 | now | worth adopting |
| 9 | Apature → Folio | exiftool/PM round-trip fixtures | no fixtures from past releases | `app/src/test/resources/backups/<version>.json` loaded by `LayoutLoadTest` | S | none | TST-7, STA-7 | now | worth adopting (see F) |
| 10 | Apature → Folio | STATUS numbers (ms, MB/s) | perf "where today" is qualitative | paste `DuoMotionTrace` and performance-log p95s into performance.md | S | none | Folio gap 1 | now | worth adopting |
| 11 | Apature → Folio | `pool.clear_pending()`, texture budget | package refresh loads every app name (0.45 s) | cancel-on-invalidate for the refresh; names from the launcher cache | M | medium | Folio perf gap 7 | next | needs investigation |
| 12 | Apature → Folio | SHA-256 per copy | — | not for launcher state: SharedPreferences is atomic enough | — | — | — | — | should be avoided |
| 13 | Folio → Apature | `in-flight.md` | single session, PLAN.md handoff exists | — | — | — | — | — | not applicable yet |
| 14 | Folio → Apature | SHA-pinned actions, secrets scan | actions by tag | pin SHAs; add a secrets scan | S | none | — | next | worth adopting |
| 15 | Apature → Folio | headless `--screenshot` harness | screenshot tests missing | Roborazzi (Folio gap 1) is the native equivalent | M | — | — | next | already planned in Folio |

## D. Shared standards

See `SHARED_STANDARDS.md` in this folder.

## E. Roadmaps

**Folio** (in its own order, from its gaps tables plus this audit)
1. Backup re-read before "saved" (#8) and version fixtures (#9).
2. Numbers into performance.md (#10); then the `FrameTimingMetric` swipe
   benchmark it already plans.
3. Emulator job for androidTest; Roborazzi with accessibility checks.
4. Lint baseline at a quiet moment.
5. Cancel-on-invalidate for the package refresh (#11).

**Apature**
1. Prefs schema + damaged backup (#2); refresh STATUS.md and ADR-019…027.
2. Standards with ids and gaps tables (#1); threat model (#4).
3. Capabilities struct and Settings page (#5).
4. Parser fuzzing (#3); SHA-pinned actions (#14).
5. Release rules in CI once Developer ID signing exists (#6).

**Apature Pocket** (before any code)
1. Write the capability table for the cameras to support, measured, not
   assumed (PTP operation codes per body).
2. Mock PTP backend first; the state machine and reconnect logic are tested
   against it before hardware.
3. Versioned on-device config with a damaged backup from day one.
4. Safe Mode and resource limits on every camera-fed parser.
5. The verified-copy ledger and "never write to the source" carry over.

## F. Practical improvements (proposed, not implemented)

**Apature: versioned prefs with a damaged backup.** Files:
`apps/desktop/src/prefs.rs` (add `version: u32` = 1, `load()` keeps
`prefs.damaged.json` on a parse error and starts from defaults with a status
line; `save()` writes `prefs.json.bak-<old-version>` once before the first
write after an upgrade), one test per version in `prefs.rs`. Validation:
unit tests load v0 (today's file, no version key) and a corrupted file; the
app shows a toast on a damaged file. Benefit: a settings loss becomes
recoverable and visible; the ingest templates and delivery targets survive
renames. Size S.

**Folio: version fixtures for Layout Backup.** Files:
`app/src/test/resources/com/mccal/folio/backups/0.6.7.json`, `0.6.8.json`
(exported from a phone, apps anonymised), and a case in `LayoutBackupTest`
that imports each and asserts the row and widget counts. Validation: the
test runs in the existing CI unit job. Benefit: a future change to the
import parser cannot silently break restores from a shipped version; this is
Apature's round-trip-fixture habit in Folio's own test shape. Size S.

Both wait for approval before any code changes.

## G. Lessons from failures

| Where | What happened | Generalisable lesson | Transferable? |
|---|---|---|---|
| Folio 0.6.7 | beta branch ran 12 ahead / 5 behind main, two changelog sections | one branch owns the changelog; machines check it (`release-rules`) | yes; Apature before its first release |
| Folio 0.6.8→#381 | instrumented tests did not compile for ~20 PRs and nothing noticed | anything not run must at least be compiled in CI | yes; Apature now builds the desktop in CI for the same reason |
| Folio T9 | a package could make Home unusable | crash-count safe mode that disables the last change | yes; Pocket, and Apature's plugin-free design makes it moot for now |
| Apature 2026-10-19 | `.xmp` probe matched `.XMP` on APFS; Linux test passed, Mac failed | case-insensitive filesystems lie to `is_file`; resolve the real directory entry; run tests on the target OS | yes; Folio's exports on exFAT SD cards |
| Apature 2026-10-20 | commit pushed without compiling after a formatter moved an anchor | gate commits on a build in the same step; CI must build the shipping crate | yes |
| Apature 2026-10-19 | deps slipped under a platform-specific table | manifest edits by script need a build on the other platform before push | yes |
| Apature STATUS.md | the status doc stopped at M3 while PLAN.md grew | one living document, with a gaps table that a PR must touch | yes; Folio solved it with "where today" sections |
| Apature headless | screenshot counter stalls when the app idles | a harness that counts frames needs a repaint request, or a wall clock | partly; Compose tests use idling resources for the same reason |
| Apature ExFAT | `._` AppleDouble files beside every copy | OS behaviour on foreign filesystems must be in the spec, not a surprise | yes |
| Both | Photo Mechanic renders nothing on this Mac | a reference tool can fail; keep an independent validator (exiftool) | yes |

## H. Workflow for future sessions

1. Read the project's own context first (`docs/PLAN.md` or Folio's
   `in-flight.md` and standards).
2. Name the task and the subsystem it touches.
3. Before designing, grep this folder's matrix for the subsystem. If the
   other project solved it, read that implementation, not this summary.
4. Reuse the idea in the host language and conventions; no shared runtime.
5. Change on a branch or worktree; one concern per change.
6. Verify the way that project verifies (Folio: tests + lint + CI; Apature:
   tests + clippy + headless screenshot).
7. A genuinely new lesson goes in that project's standard or ADR; a
   transferable one also gets a row in the matrix above.
8. Never run this audit on every task; use it when the task matches a row.
