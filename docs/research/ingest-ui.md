# Ingest and copy UI: what the references do, what Apature does

Status words as in the brief: *documented* = from the vendor's own docs or
help, *observed* = seen on this machine, *inferred* = from working
knowledge, *ours* = decided for Apature.

## Photo Mechanic (Ingest dialog) — documented / observed

* **Sources**: cards are detected when mounted and can open the Ingest
  dialog automatically; folders can be ingested too ("Ingest Disks" vs
  "Ingest Folders"). Multiple cards can be ingested in one go.
* **Copy to**: a primary destination and an optional secondary; folder
  naming options (into a named folder, into dated folders, keep the card's
  structure); `{variables}` in folder and file names, including `{seqn}`
  with a per-ingest sequence counter.
* **Rename ingested photos as**: template with variables; RAW+JPEG pairs
  keep the same base name.
* **Apply IPTC stationery pad**: the pad is written into copies during
  ingest, so captioning starts from a filled sheet.
* **Incremental ingest**: only photos not previously ingested from this
  card; "Filter files" (RAW only, JPEG only, all).
* **After ingest**: open a contact sheet of the copies, unmount the card.
  "Erase source after ingest" exists and is off by default.
* **Verification**: a copy is read back and compared; failures are listed,
  the card is never modified by a failure.
* Observed on this Mac (2026.6): the owner's existing ingest `.snap`
  presets set `IngestRenameEdit {headline}-{seqn}`, a primary destination
  on an external volume, and the stationery pad from the assignment
  generator (see `~/dev/shoot-log/photo-mechanic/generator`).

## Original Apature (Tauri) — observed in its code

* Plan then run: the whole plan (every destination path) is built first,
  shown, and refused outright if two sources map to one destination or a
  destination exists.
* Copy = `.partial` temp + full SHA-256 of source and copy + rename;
  secondary copy verified the same way; sequential by design because the
  card reader is the bottleneck.
* Templates use Photo Mechanic variable names so habits transfer.
* Incremental ingest keyed on a 64 KiB fingerprint, recorded only after a
  verified copy.
* Erasing the card is per file, gated on every destination verifying, on a
  second destination existing, and on an explicit acknowledgement.
* The UI preview calls the same planner, so what is shown cannot differ
  from what is written.

## Lightroom Classic import — inferred

Grid of thumbnails with checkboxes (import all / new photos / selected),
destination folder tree, rename presets, "Apply during import" (develop
preset, metadata preset, keywords), "Make a second copy to", "Don't import
suspected duplicates". The checkbox grid is the clearest model for *choosing*
a subset before copying; the rest is the same shape as Photo Mechanic.

## Apple Image Capture / Photos import — inferred

Minimal: pick device, "Import to" folder, Import All vs selected, "Delete
after import" toggle (off by default). Progress is a bar plus a count; errors
are a single alert. Good at not asking questions; bad at telling you what
happened to each file.

## Our Ingest workspace (M5) — ours

Built on `apature-ingest` (plan / run / ledger / template), all of which is
unit-tested:

| part | behaviour |
|---|---|
| Source | mounted volumes listed; any folder choosable; include subfolders (cards keep `DCIM/100NIKON`); file filter all / RAW only / JPEG only; "only photos not ingested before" (fingerprint ledger) |
| Destinations | primary and optional secondary; folder template (`{date}-{headline}`, `{year}/{month}/{day}`, empty); rename template (`{headline}-{seqn}`), sequence start, headline/slug (prefilled from the loaded assignment) |
| Plan | rebuilt on a worker thread on every option change: files, photos, bytes, destinations, skipped count, first copy path, collisions listed in red; Start is disabled while any collision exists |
| Run | sequential; primary from the card, secondary from the fresh primary copy; each copy verified by full SHA-256; per-file list with ✓/✕ and the error text; cancel after the current file; ledger updated only for files verified everywhere |
| After | report says "every file verified at every destination" or names the failures; optional assignment pad applied to the copies; "Open the ingested folder" |
| Never | deletes or modifies a source; overwrites a destination; hides a failure behind a bar |

Deferred, with reasons: card auto-detect and unmount (needs disk
arbitration APIs; soon), erase-source (needs the Apature-style triple gate
and a design pass; not before an .app bundle), checkbox grid for partial
ingest (filter + "only new" covers the deadline case), parallel copies (no:
card readers are the bottleneck).

## Open questions for the owner

1. Default folder template: `{date}-{headline}` (what this doc assumes) or
   the Butler Eagle slug alone?
2. Should a successful ingest switch the app to the new folder automatically?
3. Is a secondary destination the norm (second SSD) or the exception?
