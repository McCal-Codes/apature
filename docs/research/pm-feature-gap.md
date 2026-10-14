# Photo Mechanic: how ingest works, and what else to implement

Sources, labelled as the brief asks: *documented* = Camera Bits' own
articles read on 2026-10-14 ("Ingesting Photos", "Live Ingest",
"Automatically Open Ingest Window…", "The Sequence Variable");
*observed* = the owner's preset files in `~/dev/shoot-log/photo-mechanic`
(ingest, copy and stationery `.snap` files, whose keys are PM's actual
option set); *inferred* = working knowledge; *ours* = Apature today.

## 1. Ingest, the way Photo Mechanic does it (documented + observed)

| aspect | Photo Mechanic | Apature today | gap |
|---|---|---|---|
| Open | File › Ingest… (⌘G), on card mount (General › On Mount of Camera Disk = Show ingest dialog), at launch, or Auto Ingest with last settings | ⌘⇧4 workspace, ⌘⇧I menu | **card-mount trigger and auto-ingest** |
| Sources | Disks (all mounted; Rescan; Unmount; hide a volume via `.pmingestignore`), Folders (user list), From Selection; ⌘-select several at once | volumes list + one folder | multi-source, from-selection, hide-volume |
| Destinations | Primary, optional Secondary; Live Ingest adds Tertiary | primary + secondary | tertiary only for Live Ingest; fine |
| Copy Photos | directly / into dated folder `YYYYMMDD` (ingest date) / into folder with name (template, `/` makes subfolders) / dated then named | one folder template (empty = direct) | add the two dated presets as template defaults: `{today}` and `{today}/{name}` |
| Source directory structure | Ignore / Keep separate source folder names / Preserve all | flattened | **preserve-structure option** |
| Filter files | all / RAW only / JPEG only, lockable | same | parity |
| Metadata | Apply Metadata Template (global pad) or a local template per ingest; variables resolve per photo | optional assignment pad after copy | pad is applied *after* copying in ours; PM applies during. Same result on disk |
| Rename | template with variables; `{seqn}` is a **persistent counter** with prefixes, settable/resettable, leading zeros set the width | template; sequence restarts at seq_start each run | **persistent sequence counter** |
| Incremental | remembers what was ingested on this computer, works retroactively | fingerprint ledger, same idea | parity (our ledger records even when "all" is chosen) |
| Merge folders into same destination | collapse several sources into one destination | n/a (one source) | with multi-source |
| Background | ingest runs in the background; a contact sheet opens as soon as some files land | runs on a worker thread; workspace stays on Ingest; folder opens at the end (pref) | **open the sheet early and keep ingesting** |
| After ingest | unmount source disks; open contact sheet(s), optionally in background | open folder after (pref) | **unmount** |
| Verification | verifies copies; "verify all photos ingested before ejecting" is advice | SHA-256 both sides, per-file report, never deletes | ours is stricter |
| Erase / reformat | `IngestReformatDisks` exists, off by default | absent by design | keep absent until the triple gate exists |
| Companion files | WAV voice memos copied with photos (copy options `CopyWavFilesCheckbox`) | not copied | **WAV/XMP companions** |
| Collisions | `IngestAutoIncSuffix`: auto-increment a suffix instead of failing | refuse the plan | **offer auto-suffix as an option**, default refuse |
| Catalog | Auto Catalog (Plus) | later | later |

The owner's own conventions (observed in the presets), now supported verbatim
by the template engine: folder `{year4}/{month0} {monthname}/{mnn3} {day0} {head}`,
ingest rename `{iyr2}{imn0}{idy0}{headline}…`, copy rename
`BUTLR-{headline}{seqn}-{iptcmonth0}{iptcday0}{iptcyear4}`, destination root
on an external volume, pad applied on ingest, open destination after copy.

### Live Ingest (documented)

Watches one or more folders and ingests new files as they appear, with the
same destination/folder/rename options plus tertiary destinations, and
"monitoring files for stability while copying" (a file is only ingested once
its size stops changing). This is exactly the receiving end of the owner's
camera-FTP idea: the camera pushes to a folder, Live Ingest files it. Ours
has nothing yet; the engine is reusable, the watcher (`notify`) and
stability check are the new parts.

## 2. Everything else Photo Mechanic does, against Apature

Status: **have** = parity or better; **partial**; **missing**. Priority is
for the owner's deadline workflow (Butler Eagle / TribLive).

| area | Photo Mechanic (inferred unless noted) | Apature | priority |
|---|---|---|---|
| Contact sheet | thumbnails from embedded previews, sort by name/capture/etc., filter by class/rating/tag, select, tag, class, rate, zoom, multiple tabs | have (tabs, filters, sort, marks) | — |
| RAW+JPEG | combined or separate display; which is primary | pairs combined, RAW primary | partial: a "show JPEG as primary" option |
| RAW → JPEG conversion | Save As renders RAW through its own engine | opt-in system RAW decode on export (macOS ImageIO), else embedded preview | **have (opt-in)**; built-in demosaic later |
| Preview | loupe, zoom 100 %/fit, pan, compare two or more frames, rotate, crop guide, overlays | fit only; no 1:1, no pan, no compare | **1:1 zoom + pan, compare view** (2) |
| Rotation | lossless rotate (orientation tag) | none | rotate via `tiff:Orientation`/`Orientation` write (4) |
| Crop | crop stored in XMP, applied on export | none | later |
| Metadata | Metadata (IPTC) template / stationery pad, variables, code replacements, hot codes, structured keywords panel, GPS, batch apply, "Set Sequence Variable", copy/paste metadata | have pad, codes, variables, caption workspace; no keywords panel, no GPS, no copy/paste metadata | **copy/paste metadata between frames**, persistent sequence (3) |
| Save As / Export | resize, JPEG quality, sRGB, keep/strip metadata, watermark, rename on export, "Send photos to" | **have (M7)**: resize, quality, EXIF/XMP/IIM/ICC, rename; no sRGB conversion, no watermark | done except colour conversion and watermark |
| Delivery | FTP/SFTP upload, email, Send to, Dropbox folder | nothing | FTP/SFTP after export (5) |
| File ops | copy/move/rename/delete to trash, "Rename Photos" tool with variables | ingest copy only | **rename in place (batch) and move-to-trash** (3) |
| Live Ingest | watched folders | nothing | after export; pairs with the camera idea (6) |
| Colour | ICC display, embedded preview colour space | none | ICC-aware preview (4) |
| Catalog (Plus) | catalogs, collections, search | nothing | later |
| Slideshow, printing, dust removal | yes | no | not planned |

## 3. Implementation order (proposed)

1. **Export / Save As** (M7): resize to long side, JPEG quality, sRGB (when a
   profile is known), keep IPTC/XMP and write IPTC-IIM for legacy desks,
   rename template, destination folder, batch over the selection, progress
   list. The one feature standing between Apature and "deliver".
2. **Preview 1:1 and compare**: Space toggles 1:1 at the pointer, drag to
   pan, `C` compares the selection side by side.
3. **Rename in place, move to trash, copy/paste metadata**, persistent
   `{seqn}` counters with prefixes (Settings › Metadata).
4. **Ingest polish from §1**: card-mount trigger, unmount after, preserve
   source structure, companion WAV/XMP files, auto-suffix option, open the
   sheet while ingesting, multi-source merge.
5. **Rotate** (orientation write), ICC-aware previews.
6. **Live Ingest** with folder watching and file-stability checks; this is
   where the camera FTP idea lands.
7. FTP/SFTP delivery.

Items 1–3 complete the daily loop; 4–7 are what make it comfortable.

## 4. Open questions for the owner

* Export defaults: 2048 px long side, quality 90, sRGB, filename
  `{filename}` (from the Butler Eagle workflow note), or per-outlet presets
  (Eagle vs TribLive)?
* Does the desk need IPTC-IIM (older systems) or is XMP enough?
* Should the `{seqn}` counter be per assignment (prefix = slug) by default?
