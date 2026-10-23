# Changelog

All notable, user-visible changes. Versions follow semantic versioning;
0.x while the app is finding its shape.

## [Unreleased]

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
