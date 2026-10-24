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
| After ingest | unmount source disks; open contact sheet(s), optionally in background | open folder after (pref); eject pref + button (M10) | open in background |
| Verification | verifies copies; "verify all photos ingested before ejecting" is advice | SHA-256 both sides, per-file report, never deletes | ours is stricter |
| Erase / reformat | `IngestReformatDisks` exists, off by default | absent by design | keep absent until the triple gate exists |
| Companion files | WAV voice memos copied with photos (copy options `CopyWavFilesCheckbox`) | not copied | **WAV/XMP companions** |
| Collisions | `IngestAutoIncSuffix`: auto-increment a suffix instead of failing | refuse (default) or `-N` suffix option (M10) | parity |
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
| Preview | loupe, zoom 100 %/fit, pan, compare two or more frames, rotate, crop guide, overlays | fit, 1:1 with pan, compare 2–4 with shared zoom (M8) | loupe, rotate, crop guide later |
| Rotation | lossless rotate (orientation tag) | none | rotate via `tiff:Orientation`/`Orientation` write (4) |
| Crop | crop stored in XMP, applied on export | none | later |
| Metadata | Metadata (IPTC) template / stationery pad, variables, code replacements, hot codes, structured keywords panel, GPS, batch apply, "Set Sequence Variable", copy/paste metadata | have pad, codes, variables, caption workspace; no keywords panel, no GPS, no copy/paste metadata | **copy/paste metadata between frames**, persistent sequence (3) |
| Save As / Export | resize, JPEG quality, sRGB, keep/strip metadata, watermark, rename on export, "Send photos to" | **have (M7)**: resize, quality, EXIF/XMP/IIM/ICC, rename; no sRGB conversion, no watermark | done except colour conversion and watermark |
| Delivery | FTP/SFTP upload, email, Send to, Dropbox folder | nothing | FTP/SFTP after export (5) |
| File ops | copy/move/rename/delete to trash, "Rename Photos" tool with variables | rename in place with the template engine, move to Trash (M9); no move/copy between folders | copy/move to folder later |
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

## 5. What Photo Mechanic users ask Camera Bits for (read 2026-10-20)

Source: the public Feature Requests board at forums.camerabits.com (three
pages of thread titles with view counts; thread bodies need a login) plus
G2 reviews and Adobe-forum threads. Views are the only demand signal
available; treated as *observed*.

| ask (views) | what it means | Apature |
|---|---|---|
| Destination path variable (66k) | folder/rename templates that can use the destination path and more variables | have: full variable engine incl. `{folder}` `{today}`; add `{dest}` cheaply |
| Unified zoom (22k) | one zoom behaviour across preview/loupe/compare | partial: 1:1 + compare share zoom; loupe missing |
| AI keywording (13k) · AI culling · AI on ingest | suggested keywords, blur/duplicate detection | none; burst stacks + sharpness hint planned |
| On/off switch for code replacements (12k) | toggle `\code\` expansion without unloading the file | cheap: a checkbox in Caption |
| Info tooltips options (7k) | choose which fields the thumbnail tooltip/info board shows | partial: fixed inspector; make rows configurable |
| FTP wish list 2026 (8k) · resend queue · upload status persistency · FTP status in IPTC window | queue, retry, persistent status, status next to the caption | partial: delivery done; add a queue with retry and a per-file sent ledger |
| Set GPS on ingest (5k) · GPS taken over on ingest | GPS from a track/phone or a fixed location applied during ingest | none; keywords/GPS slice |
| Larger, more visible tag marker (5k) | the tag check is too small on thumbnails | cheap: bigger tag badge |
| Turn autocomplete lists on/off (5k) · keyboard shortcuts for IPTC templates/snapshots | control over autocomplete; snapshots on keys | partial: recent-values menus; add per-field toggle and snapshot keys |
| Live Ingest from FTP as source (3.6k) | the camera-FTP receiver | planned (Live Ingest) |
| AVIF / WebP (3k) | more output formats | JPEG XL done; AVIF via libavif later |
| Prioritize star-rated images on ingest (1.3k) · in-camera rating/crop info | honour camera ratings and crops at ingest | cheap: read XMP Rating from NEF; order plan by it |
| Golden-section overlay (6k) · extra crop anchors · two constrained ratios · auto roll alignment · AF point in preview | preview/crop tools | crop tool slice: ratios, overlays, AF point from EXIF |
| Exposure correction for viewing only (1k) | brighten the preview without editing | cheap on the preview (gamma slider) |
| Ingest from phones · remote editing · SmugMug folders · social | out of scope for now | — |

Takeaways for Apature's order: (1) keep the template engine ahead (it is the
single most-viewed ask), (2) a delivery queue with retry and persistent
status is the next delivery step before cloud drives, (3) small wins with
real demand: code-replacement switch, bigger tag badge, camera ratings on
ingest, view-only exposure boost, (4) crop tool with ratios/overlays and AF
point, (5) AI keywording is the most-wanted thing we do not plan; a local
model later, never cloud by default.

## 6. Menu audit against the installed Photo Mechanic (2026-10-23)

Source: *observed*. The menu bar of the Photo Mechanic installed on the
owner's Mac, read through accessibility (File, Edit, Image, View, Tools; no
settings dialogs opened, nothing changed). Ranked by what the Norwin–Seneca
Valley game (deadline: 2–4 print photos in 75 minutes) actually needed.

**Have, at parity or close:** ingest and Live Ingest, rename photos with
variables, Save Photos as (export) with prior settings, FTP/SFTP delivery,
metadata template and copy/paste snapshot, rotate, tag, colour class, rating,
filters, favorites, remove-to-Trash, JPEG preview extraction (CLI), crop
stored in the sidecar.

### Needed next, in this order

| # | Photo Mechanic | Why tonight | Apature |
|---|---|---|---|
| 1 | **Copy/Move Photos…** (and "with prior settings") | Moving a day's RAWs from Google Drive to the archive drive was a hand-rolled script: verify each file, never delete without asking, survive Drive's online-only placeholders | missing: verified copy/move between folders with a ledger, a "source is a placeholder, use the card" fallback, and an explicit "now trash the source?" step |
| 2 | **Transferred** state: Clear Transferred, Select Transferred | Which frames went into the print set and which into the gallery was kept in my head | missing: mark a frame when it is exported or delivered, show a badge, select/clear by it |
| 3 | **Select** menu: Rating, Color Class, Tagged, Rotated, Cropped, Transferred, RAW, Date Range, Others (invert), Load/Save Selection | "the 34 five-stars" meant grepping sidecars | missing: select by rating/class/state, invert, save and load a selection file; then Export uses the selection |
| 4 | **Save Photos as… (prior settings)** and the Export button on the selection | He asked for "make the export easier" | partial: one-click Export picks with the last preset and the destination subfolder named from the slug |
| 5 | **Remove Crops…**, Select Cropped, crop ratios and overlays, auto roll alignment (a top feature request) | Most wide frames were ~2° tilted and loosely framed; crops had to be redone by hand | partial: crop and straighten are in the sidecar and export (CLI); needs the in-app tool: draw, ratio presets (3:2, 4:5, 16:9), straighten handle, level suggestion from `apature-cli level`, preview that shows them |
| 6 | **Find… / Find and Replace…** across captions | Names and play descriptions were corrected after the fact (118 became a kick) | missing |
| 7 | **Edit Photos with…** (open in Capture One, Photoshop) | Capture One is installed; some frames need real edits | missing |
| 8 | **Adjust Capture Dates and Times…** | Camera clock offsets after travel; sequences from two bodies | missing |
| 9 | **Keywords Panel / Structured Keywords** | Keywords were pasted from the assignment sheet | missing; assignment keywords work |
| 10 | **Set GPS / Import GPS Coordinates / Reverse Geocode** | Not used tonight | missing |
| 11 | **Update IPTC/XMP…**, Delete Metadata, Mask/Unmask, Apply Rotation to JPEGs, Embed ICC Profile, Change Resolution | Housekeeping tools | missing; low |
| 12 | **View**: Info Tooltips choices, Make Arrangement, Cursor mode, Customize Toolbar, Unknown Files as Proxies | Comfort | partial |
| 13 | **Send via email…**, Upload… (cloud), Export… | Delivery variety | FTP/SFTP only |
| — | Burn to disc, Print, Page Setup, Slide Show, Live Slide Show, Convert RAW to DNG, Convert IPTC encodings, Remap, Trim data, Revert TIFF-based RAW | Not part of the daily loop | not planned |

### Lessons from using it on a real deadline

* A feature that cannot be undone or explained in the status bar costs more
  than it saves; Delete is now immediate and ⌘Z restores from the Trash.
* Bare-key shortcuts must be handled in the app's own view on macOS; a menu
  accelerator without ⌘ is only a label.
* Files on a cloud drive can be online-only placeholders; any batch needs to
  detect that and fall back or fail fast instead of hanging.
* Always look at the output (a contact sheet of the export) before saying it
  is done.
