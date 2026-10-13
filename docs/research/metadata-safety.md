# Metadata safety findings (XMP sidecars), 2026-10-13

What was verified, how, and what remains open. "Verified" means an
automated test exists in `crates/photo-metadata/src/xmp.rs` and passed.

## Verified

| claim | how |
|---|---|
| Unrelated metadata survives a marks write byte-for-byte | A Lightroom-style sidecar (crs develop settings, dc bags/seq/alt with a real Butler Eagle caption, xmpMM history, aux lens data, a custom `myvendor:` namespace with an escaped value) is applied to; the output with the four owned properties stripped equals the input with them stripped. |
| Photo Mechanic-style attributes update in place, no duplicates | `xmp:Rating`/`xmp:Label`/`photomechanic:ColorClass`/`photomechanic:Tagged` replaced; counted once. |
| Element form (`<xmp:Rating>3</xmp:Rating>`) and legacy `xap:` prefix are read and updated under their own prefix | test `legacy_xap_prefix_is_updated_not_duplicated`. |
| Missing properties are inserted on the first `rdf:Description`, declaring the namespace only if the document binds none | Lightroom and Camera-Raw-only fixtures. |
| Clearing a colour class removes `xmp:Label` rather than writing an empty one | `clearing_label_removes_it`. |
| Writes are atomic: temp file, fsync, rename; a failed write leaves the original untouched and no temp file behind | read-only directory test. |
| Read-back check before replacing the file | the edited text is parsed and compared with the requested marks; mismatch → `XmpError::Verify`, original kept. |
| ExifTool validates our fresh and edited sidecars with no errors and reads Rating, Label, ColorClass ("2 (Winner alt)"), Tagged and the preserved Headline | `exiftool -validate` test, runs when exiftool is installed (13.x here). |
| Embedded XMP in a sibling JPEG can be read and compared; disagreements are reported | `pw scan` prints `CONFLICT` lines. |
| Idempotence | applying the same marks twice changes nothing. |

## Photo Mechanic round trip (2026-10-13, PM 2026.6 build 9175, verified)

Done on copies of the three D850 NEFs in a scratch folder; the NEFs were
hashed before and after and are byte-identical.

| step | result |
|---|---|
| `pw mark` 7783 → rating 4, class 1, tagged; 7784 → rating 2, class 3 | sidecars written |
| open the NEF-only folder in PM | 7783 shows a **red** bar, four stars, tag checked; 7784 **green**, two stars; 7802 unmarked. Both read exactly. |
| in PM: Image › Set Color Class › Yellow, Set Rating › 3 Stars, Tag Photos on 7802 | PM wrote `850_7802.XMP` (upper-case extension, no xpacket wrapper) with `xmp:Label="Yellow"`, `xmp:Rating="3"`, `photomechanic:ColorClass="2"`, `Tagged="True"`, plus `Prefs`, `PMVersion`, crd/plus/aux fields |
| `pw scan` | 7802 → rating 3, class 2, tagged true. Our two sidecars untouched by PM. |
| same three files as RAW+JPEG pairs | **PM showed no marks**: for pairs it takes metadata from the JPEG (confirmed by observation; the exact preference was not inspected). Our sidecar is ignored until we can embed XMP in the JPEG or write marks to both. |

## Embedded XMP in JPEGs (2026-10-13, verified)

`photo_metadata::jpeg_xmp` splices one APP1 XMP segment into a JPEG:
editing the existing packet in place (the D850 writes its own packet, so
real camera JPEGs grew by ~155 bytes) or inserting a padded packet after
the Exif APP1 when none exists. Every other byte is copied; image data is
never re-encoded. Atomic temp → fsync → rename with a read-back check.
`photo_metadata::items` applies the policy: RAW → sidecar, JPEG → embedded,
RAW+JPEG pair → both; reads prefer the sidecar when a RAW exists and report
disagreements with any JPEG packet.

| check | result |
|---|---|
| unit tests on a JPEG skeleton | insert after Exif / after APP0, replace in place, byte-exact outside the segment, oversize and non-JPEG rejected, atomic + idempotent |
| exiftool on a real D850 JPEG copy after embedding | `-validate` clean; Rating 4, Label Red, ColorClass 1, Tagged Yes; ImageSize and Model unchanged; +155 bytes |
| Photo Mechanic, pair folder, after `pw mark` embedded into the JPEGs | 7783 red bar, four stars, tagged; 7784 green, two stars: **pairs now round-trip into PM** |
| Photo Mechanic → us on a pair (Blue, 1 star, tag on 7802) | PM wrote **both** the JPEG packet and `850_7802.XMP`; `pw scan` read rating 1, class 4, tagged; NEFs byte-identical. PM's own behaviour for pairs matches the sidecar+embedded policy. |

Consequences applied in code: the default colour-class names are now the
ones this PM shows (Red, Yellow, Green, Blue, Purple, Cyan, Brown, Trash)
with the classic names kept as read aliases; `sidecar_path` reuses an
existing `.XMP`/`.xmp` of either case and otherwise follows the case of the
source extension like PM; a verbatim PM 2026 sidecar is a test fixture.

## Not verified (honest list)

* **Lightroom.** Not installed. Capture One is, and reads `xmp:Rating` /
  `xmp:Label` from sidecars, so it can stand in.
* **Backup copies.** There is no `.xmp.bak` and no operation log. A bad
  edit that passes the read-back check (e.g. a value the user did not
  intend) is not recoverable except by undo in the app, which does not
  exist yet. Recommendation for 0.2: an append-only operation log per
  folder (Apature's `operation_log` pattern), not per-file backups.
* **Extended XMP** (`/xmp/extension/` APP1 chunks) is not reassembled
  when reading embedded packets; ratings never live there.

## Known limitations of the approach

1. The reader is regex-based and **does not understand XML comments or
   CDATA**: a commented-out `<rdf:Description` or `xmp:Rating` would be
   treated as live. No known sidecar writer emits comments; still a
   reason to move to a streaming XML parser (quick-xml, as Apature did)
   once IPTC fields are added and the property set grows.
2. Properties are located anywhere in the document and insertion targets
   the **first** `rdf:Description`. Lightroom, Photo Mechanic, Capture One
   and exiftool all write one Description; a multi-Description document
   from another tool could end up with the rating on a different
   Description than its siblings. Harmless to readers, untidy.
3. **JPEG writes change the JPEG's mtime**, which invalidates its thumbnail
   cache entry (the key includes mtime). One extra decode per marked JPEG;
   a content-hash-free key that ignores mtime for JPEGs would avoid it but
   would miss real edits. Left as is for now.
   Extended XMP (`/xmp/extension/` segments) is left untouched; a packet
   over 65 504 bytes is refused rather than split.
4. **Sidecar naming:** only `IMG.xmp` is read and written. darktable-style
   `IMG.NEF.xmp` is neither read nor detected.
5. The colour-class **names** must match the user's Photo Mechanic
   preferences for `xmp:Label` to round-trip textually; the numeric
   `photomechanic:ColorClass` is what we trust on read.
