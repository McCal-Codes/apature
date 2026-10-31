# Changelog

All notable, user-visible changes. Versions follow semantic versioning;
0.x while the app is finding its shape.

## [0.1.2] — Unreleased

### Added
- Assignment variables: `{team}`, `{teamname}`, `{opponent}` and `{event}` (and the existing `{venue}`, `{city}`, `{slug}`) come from the loaded assignment, so one caption ending or template says the right thing for each job; your own variables win if a name is in both. New assignment fills the team (the home side) and the opponent from a sheet's "FOOTBALL: A at B" line.
- One template, both apps: Import from Photo Mechanic now also reads an IPTC template (the snapshots under IPTC templates, not only ingest snapshots) into an assignment, including Category, the date, the team and the caption ending; tools/pm_from_assignment.py goes the other way and writes a Photo Mechanic IPTC template, ingest and copy snapshot named after the slug from an assignment file, with the job's values in and the per-photo variables left as variables. Photo Mechanic's own user/client variables are a fixed set of fields, so the per-job values live in the assignment file.
- An assignment that still has template text (`[venue]`, `MMDDYY`, `YYYY-MM-DD`) is refused by the app and the command line, with the lines to fix, so placeholder text cannot reach a photo.
- IPTC Category and Supplemental Categories (`photoshop:Category`, IIM 2:15 and 2:20), filled from the slug's section code (`103126 SPO FB …` → `SPO`) or a `category:` / `supp_cat:` line in the assignment, like Photo Mechanic's Status section.
- Variables in assignment fields: when a pad is applied, `{iptcdow}`, `{iptcmonthnameap}`, `{iptcday}`, `{iptcyear4}`, `{year4}`, `{credit}`, `{headline}` and the rest are filled in per photo from its own capture date, the way Photo Mechanic does for a template. Unknown names stay as typed.
- A generic sports assignment template (docs/templates/sports-assignment.txt) and a script that builds Photo Mechanic snapshots from your own (tools/pm_butler_sports.py).
- Edit Photos (Image menu, right-click): sends the selected photos to the app chosen in Settings › General › Edit photos in — Capture One when it is installed, else the system default — the way Photo Mechanic's Image › Edit Photos does. The editor reads the rating, label, caption and crop from the sidecar beside each RAW; see docs/workflows/capture-one.md for the Capture One settings and what carries over.
- Named ingest presets: a Preset row on the Ingest screen saves the folder template, rename template, destination, file filter and companion options under a name, applies, updates and deletes them, and imports every Photo Mechanic ingest and copy snapshot in one step (File presets keep their own names, marked (ingest) or (copy)). A preset whose drive is not connected keeps the current destination and says so.
- "With Prior Settings" twins in the File menu, like Photo Mechanic's: Ingest with Prior Settings (plans with the saved options and starts when the plan is clean), Rename Photos with Prior Settings (the last template and starting number), and Copy or Move Photos with Prior Settings (the last destination, mode and card fallback). Each falls back to its dialog when something is missing.
- Ingest From Selection (File menu and the right-click menu): the Ingest workspace on just the photos you picked, with your folder template, rename, pad and per-file verification, like a card ingest. An opt-in Move sends the originals to the Trash only after every file of that photo is verified at every destination, and ⌘Z puts them back. The card ingest settings you had are put back with "Use the whole source again" and are never overwritten by a selection run.
- Copy or move photos between folders and drives: `apature-cli transfer --to FOLDER [--move] [--card CARD]`. Every file (RAW, JPEG, sidecar) is checked by SHA-256, nothing is overwritten, a run can be repeated, and a move sends the originals to the Trash only after every file of that photo verified. A file a cloud drive has not downloaded is read from the card when one file there has the same size. In the app: File › Copy or Move Photos… (and Copy or Move to… in the right-click menu) asks where, shows progress, can stop after the photo in hand, and lists anything that did not transfer; ⌘Z after a move puts the originals back from the Trash.
- What's New: after an update a short page shows the newest section of the changelog once; Help › What's New in Apature… and About › What's new bring it back. The page is read from the changelog file, so it never drifts from it.
- Straighten: `apature-cli level` estimates how far a frame is tilted from the long straight edges in it (horizons, banners, poles, goal posts); `apature-cli straighten --degrees` stores the angle in the sidecar (`apature:Straighten`) and export turns the picture and then applies the crop, so exports come out level. The in-app preview does not show it yet.
- Delete key and Move to Trash act at once with no sheet; ⌘Z puts the photos back from the Trash (⇧⌘Z trashes them again). The undo history keeps pointing at the right photos after a delete or a restore.

### Fixed
- New assignment reads the Eagle's planning page as pasted: the list number before the slug, the real game day (not the slug's run date) from the date line, the town and state out of "Natrona Heights, PA 15065", the sport and teams from "FOOTBALL: A at B", and the stadium the caption ending names as the venue; the page's own labels no longer land in the notes.
- With a dialog open (Copy or Move, What's New, Welcome, About, the assignment sheet) the keys no longer reach the photos behind it: Delete, 1–5, T, X and the arrows did.
- Ingest From Selection ends when you pick another source or bring another folder to the front, instead of carrying the old file list into it.
- Ingest refuses, with the file count, when a cloud drive has not downloaded some of the files, instead of hanging on the first one.
- Copy or Move no longer trips over a photo list that changed while its dialog was open.
- Undo of a move to the Trash survives opening a folder or the ingest result in another tab, and a tab showing the folder the files left (or returned to) rescans when you switch back to it.
- Exporting photos that a cloud drive keeps online-only (Google Drive, iCloud) no longer hangs: Apature says which ones are not downloaded and stops before starting.
- On macOS the bare keys (1–5 rate, 0 clears, T tag, X reject, Z zoom, C compare, …) did nothing: AppKit offers the menu only ⌘-shortcuts while Apature's own view has the keyboard, and the app skipped bare keys on the assumption that the menu handled them. It handles them itself now.
- A `code:` line in an assignment (for example `code: tail = ...`) now beats the generated boilerplate of the same name, and keeps its leading space.

### Added
- Camera over Wi-Fi, direct: Apature finds a Nikon on the network over Bonjour, pairs with the code the camera shows (typed in Ingest; nothing is confirmed unless it matches), and receives frames as they are shot into the Wi-Fi inbox for Live Ingest. No transmitter accessory and no vendor utility. Settings › Ingest › Forget pairing.
- Assignments ahead of time: an assignments folder (Google Drive by default when it is installed), an Upcoming list in the Caption workspace, and New assignment, which turns a pasted editor's sheet into an assignment file with the outlet's credit, source and copyright from Settings. When an ingest finishes, the assignment file is copied into the day's folder on the destination drive.
- Camera over Wi-Fi reads the utility's per-camera subfolder.
- Camera over Wi-Fi: when Nikon's Wireless Transmitter Utility has a destination folder, it is listed under Drives & cards; one click opens Ingest on it with Live Ingest on. No WT-7, no phone.

### Changed
- Inspector sections drawn as cards; the tag is a switch. The filter popup uses chips and switches, with colour classes as dotted chips.

### Added
- Appearance › OLED: the Dark appearance on a true black ground, for OLED screens and dark rooms.

### Changed
- Export workspace laid out in the order of Photo Mechanic's Save Photos As dialog: Preset, Photos, Destination and naming with an example, Image (format, size, quality, RAW decode), Metadata, Then (open folder, send to); switches instead of checkboxes; the Export button pinned at the bottom.
- Caption workspace laid out as cards (Assignment, Caption, Keywords, Where, Who, More fields) with Save & next, Save and Revert pinned at the bottom; the code list sits behind a hover.

### Added
- Settings › Resources: style and ethics (AP Stylebook, NPPA, SPJ, Poynter) and internships, workshops and contests for students, each a checked link that opens in the browser.
- Photo Mechanic import now turns the stationery pad in an ingest snapshot into an assignment file and loads it.
- Windows: Eject the card (Shell's Eject verb).

## [0.1.1] — 2026-10-23

### Added
- Variables: captions and code replacements resolve Photo Mechanic's names ({iptcmonthnameap}, {iptcdow}, {city}, {credit}, {copyright}, {creator}, {filename}, …) and plain aliases; Settings › Metadata › Your variables defines your own {name} for any template or caption.
- Import from Photo Mechanic (File menu, Settings › Metadata, the welcome): an ingest snapshot sets the folder and rename templates verbatim, a code replacement file loads as is.

### Changed
- Default ingest rename is now `{yr2}{month0}{day0}{headline}_CAL{frame}` (260505_CAL500), the Photo Mechanic ingest preset, instead of the delivery-style `{headline}{seqn:3}-{month0}{day0}{yr2}`; existing settings are untouched.

### Added
- Settings rebuilt the way TerraNova's are: a search box that finds any setting and shows where it lives, a category rail with a count of what differs from the default, uniform rows (label and description left, control right), a Reset per changed row and per page, switches instead of checkboxes.
- Receive over Wi-Fi: an FTP server inside Apature (Ingest › Plan) that drops files into the source folder and turns Live Ingest on. One account, a six-digit password per session shown in the window. A phone running SnapBridge plus a transfer app stands in for a WT-7; cameras with an FTP client connect directly. `--receive` starts it headless.
- Welcome on first launch: the loop in four pages (card in, cull, caption, export and send), appearance and accent, a card or folder to start with, the keys for the day. Help › Welcome to Apature… brings it back; Settings › General › Learning the loop.
- One short tip the first time each workspace opens (Sheet, Preview, Caption, Ingest, Export); Got it or No more tips.
- Installers: a laid-out DMG (background, drag to Applications) when create-dmg is present; a Debian package beside the AppImage and tar.gz; a Windows installer (Inno Setup) beside the zip, with Start menu entry and uninstaller.
- `APATURE_CONFIG_DIR` points prefs.json at another folder, so screenshots and tests never touch real settings.
- Linux: an AppImage and a tar.gz on every release, built in CI on Ubuntu 22.04. Volumes are found under /media and /run/media, Eject uses gio or udisksctl, passwords go to the Secret Service keyring, Reveal opens the folder with xdg-open.
- Windows: a zip with apature.exe on every release, unsigned. Drives appear as volumes, Reveal selects in Explorer, passwords go to Credential Manager. No eject or menu bar extra yet.
- The update check picks the right download for the platform.
- Settings › Accent: six soft accent presets (Lavender, Sage, Sky, Peach, Rose, Mist) applied to selection, links and focus in both appearances.
- A new app icon (the modern A) with three optional alternates (photo frame, aperture, field), every size and `.icns`, generated from `tools/apature_icons.py` with the accent as a parameter.
- Press kit in `docs/marketing`: hero images, social preview, feature cards, copy kit; `tools/marketing-shots.sh` recaptures the screenshot set.
- Check for updates: once a day, or Apature › Check for Updates…, a toast offers the newest release's download. Settings › General can turn it off or include betas.

### Changed
- Screenshots now show a real storm-damage assignment instead of test frames.
- Example outlet names removed from the export preset hint and test fixtures.

## [0.1.0] — 2026-10-21

First shareable build, ad-hoc signed: on first launch right-click › Open, or allow it under System Settings › Privacy & Security. Everything below is new.

### Ingest
- Card detection on mount, Ingest in its own window, day chips, a live
  example of the names the templates will produce, MB/s and time left.
- Templates with Photo Mechanic's variable names plus camera, time, weekday,
  folder, destination and ingest-day variables; persistent `{seqn}` per
  headline.
- SHA-256 verification of every copy at every destination; a ledger so a
  card is never copied twice; `-1, -2` suffix option; eject when verified.
- Keep the card's folders; copy voice memos and sidecars; a backup
  destination; GPS for every photo; the sheet opens while the copy runs.
- Live Ingest: a watched folder, files picked up once their size settles.

### Cull
- Ratings, colour classes, tags, filters, sort, tabs, undo.
- Preview fit and 1:1 with one shared magnification, compare up to four,
  loupe, view-only exposure boost, burst stacks.
- Rotate and crop stored in the sidecar and applied on export, with ratio
  presets and thirds or golden-section guides.

### Caption
- Assignment pad, code replacements with an on/off switch, keywords as
  chips with recent values, GPS, Title, Caption writer, Instructions,
  Transmission reference, Save & next, Save to selection, copy and paste
  metadata.

### Export and delivery
- JPEG or JPEG XL (via cjxl), resize, quality, EXIF, XMP and IPTC-IIM kept,
  naming templates, presets per outlet.
- Delivery to a folder, FTP, FTPS or SFTP; passwords in the macOS Keychain;
  three tries per file; a per-photo sent ledger in the inspector; resend
  failed.

### App
- Two appearances with a lavender accent, Reduce Motion, text scaling,
  higher contrast, keyboard focus rings, every function in a menu.
- Folder strip at the top, window title names the folder, toasts, unplug
  detection, a menu bar extra with live status, About with notices, a log
  file, versioned settings with a damaged-file backup.

### Known limitations
- No screen-reader support yet.
- FTPS and SFTP are untested against a real desk; Google Drive and OneDrive
  sign-in are not built (use their desktop sync folder with a Folder target).
- JPEG XL needs Homebrew's jpeg-xl installed.
- Rotation and crop are applied on export, not shown as a crop on the sheet.
- macOS only.
