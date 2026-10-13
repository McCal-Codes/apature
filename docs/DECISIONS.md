# Decision log

Format: ADR-nnn, date, status. Status is one of proposed, accepted,
superseded, or **divergence** (a place where what was built differs from the
brief in `docs/BRIEF.md`).

## ADR-000 (2026-10-13, divergence): the whole app was scaffolded before Phase 0

The brief says: "Do not scaffold the entire desktop application yet"; Phase 0
research and a Phase 1 preview benchmark come first. The first session built
the workspace, engine crates, CLI and an egui window in one pass, before the
brief existed. Rather than throw that away, the code is reclassified: the
engine crates and `pw bench` **are** the Phase 1 feasibility experiment, and
`apps/desktop` is a throwaway rendering probe, not the product UI. The design
process in `docs/design/` decides what the real UI is; nothing in
`apps/desktop/src/app.rs` is treated as settled.

## ADR-001 (2026-10-13, accepted): egui/eframe for the first prototype, not yet for the product

Alternatives: Slint (GPL/royalty-free/commercial triple licence; declarative;
better native text fields), Rust core + SwiftUI/AppKit shell (best native
feel and VoiceOver; two languages; Windows needs a second shell), Tauri
(what Apature did; web renderer; 493-crate graph).

Chosen for the prototype because it keeps everything in one language, draws a
10 000-cell virtual grid with trivial code, and has AccessKit integration.
Known costs, all confirmed in this session by reading egui 0.33's API and
the prototype: no native menu bar beyond winit's default app menu; text input
is egui's own, not NSTextView (no system spell-check, dictation, or emoji
picker); VoiceOver only sees widgets that egui registers, so painter-drawn
cells are invisible; theme follows the system but our panels hard-code
colours. **The product-UI decision is deferred to
`docs/design/DESIGN_IMPLEMENTATION_PLAN.md`**; a thin AppKit layer for menus
and dialogs is the expected compromise if egui stays.

## ADR-002 (2026-10-13, accepted): own TIFF walker for embedded previews; no LibRaw, no rawler

Alternatives: LibRaw via FFI (LGPL-2.1/CDDL; full decode + previews; C++
build), rawler 0.8 (LGPL-2.1; pure Rust; previews + decode + metadata; CR3
support; API unstable; 10–25 min cold build per Apature's ADR-0002),
kamadak-exif (only tiny thumbnails; no CR3).

Chosen: a 300-line IFD reader plus a preview locator that understands
IFD chains, SubIFDs, the Exif IFD and Nikon MakerNote PreviewIFD. Verified
against exiftool on D850 NEFs: identical three previews, identical byte
counts. Zero copyleft in the graph. Cost: CR3 (ISO-BMFF) and HEIC are not
TIFF and need a second locator; no RAW decode fallback exists when a file
has no usable preview. Revisit when CR3 matters or when a develop-for-export
feature is scoped (rawler is the obvious candidate then, kept behind a
feature flag so the default build stays permissive).

## ADR-003 (2026-10-13, accepted): jpeg-decoder for decoding, with DCT scaling

Alternatives: zune-jpeg via `image` (faster full decodes, no scaled decode),
turbojpeg FFI (fastest, C dependency).

Chosen because `Decoder::scale` gives 1/2, 1/4 and 1/8 decodes, which is
what makes thumbnails cheap. Measured: 8 ms per D850 NEF thumbnail from the
1620 px preview, 25 ms from a 2048 px camera JPEG, single-threaded per image,
8 images in flight. Default features off so it does not spawn its own rayon
threads under ours.

## ADR-004 (2026-10-13, accepted): XMP sidecars are edited in place, never regenerated

Alternatives: parse to a model and re-serialise (loses unknown structure
and formatting; what most libraries do), shell out to exiftool for every
write (correct, 100–300 ms per call, external binary).

Chosen: locate each owned property (attribute or element form, any prefix
bound to the namespace) and replace only that span; insert on the first
`rdf:Description` if absent. Verified by tests that a Lightroom-style
sidecar with crs settings, dc bags, xmpMM history and a custom namespace is
byte-identical outside the four owned properties, and by exiftool
`-validate` on both fresh and edited files. Write path: temp file →
`fsync` → rename, with a parse-back check before the rename and the original
left untouched on failure. Limitation: a sidecar whose first
`rdf:Description` is not the one carrying ratings would get a second copy
of the property; not observed in Lightroom or Photo Mechanic output.

## ADR-005 (2026-10-13, accepted): one sidecar per stem, RAW is primary

`DSC_0001.NEF` + `DSC_0001.JPG` share `DSC_0001.xmp`, which is what Photo
Mechanic and Lightroom do for the RAW. **Known gap:** both of those apps
embed XMP *inside* JPEGs rather than reading a sidecar for a JPEG-only item,
so marks we write for JPEG-only folders are invisible to them until we can
embed (Phase 3). `pw scan` reports conflicts between a sidecar and a
sibling JPEG's embedded XMP so the gap is visible rather than silent.

## ADR-006 (2026-10-13, accepted): priority queue keyed by viewport distance; no cancellation

Alternatives: rayon (no priorities, no bounded queue; Apature rejected it
for the same reason), LIFO stack with in-flight cap (Apature's choice).

Chosen: a `BTreeSet<(priority, idx, kind)>` where re-requesting an item
updates its priority instead of appending. Stale requests are not cancelled
but are always behind visible ones and still populate the disk cache.
Threads: `available_parallelism - 1`, clamped to 2..=12. Not yet measured
under fast scrolling of a 1 000-item folder; see `docs/STATUS.md`.

## ADR-007 (2026-10-13, accepted): colour-class names default to Photo Mechanic's factory names

Photo Mechanic writes the class *name* into `xmp:Label`; exiftool's tag
table confirms class 1 is "Winner" and class 2 "Winner alt". The owner's
own workflow refers to them by colour (red = print pick, yellow = gallery,
green = captioned, purple = needs ID, trash = reject). The scheme is a data
structure (`ColorClassScheme`) so it can be loaded from preferences; not yet
wired to a settings file.
