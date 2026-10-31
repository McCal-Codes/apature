# Rating in the camera, and a phone bridge (research, 2026-10-30)

Two ideas from the owner (Z 6 II): star-rate frames **in the camera while
shooting**, and an **Apature app bridge** like the Nikon app that gets photos
onto a phone. Status: researched, not built. Labels: *documented* (a manual or
article says it), *unverified* (plausible, not checked on the owner's body).

## 1. Five stars in the camera

* *Documented (Z-series manuals, Z6III and Z9 pages):* in playback press the
  **i button**, highlight **Rating**, pick 0 to 5 stars (or a delete candidate).
  If **Rating** is assigned to a control with **Custom Setting f3 "Custom
  controls (playback)"**, a frame can be rated by holding that button and
  turning the main command dial, one hand, no menu. Sources:
  [Z6III manual](https://onlinemanual.nikonimglib.com/z6III/en/rating_pictures_57.html),
  [Z9 manual](https://onlinemanual.nikonimglib.com/z9/en/rating_pictures_guid-65748287-47e5-ef65-f099-ae119662b889_39.html).
* *Unverified:* that the **Z 6 II** has the same custom-control assignment (the
  Z 6 II manual was not in the search results; check **Custom Settings › f ›
  Custom controls (playback)** on the body).
* *Unverified, the important part:* **where the rating is stored.** Nikon's
  manuals say NX Studio shows it. Whether the NEF carries it in a tag that
  Photo Mechanic, Capture One and Apature read at ingest (the request on
  Camera Bits' board is "prioritize star-rated images on ingest") has to be
  measured, not assumed.
* **Test (10 minutes, owner's body):** shoot four frames; rate two 5 stars in
  playback; ingest the card with Apature and Photo Mechanic; read the sidecar
  and the NEF (`apature-cli exif`, and `exiftool -a -G1 -Rating* -XMP*` if
  installed). Record which tag carries it.
* **Build (if the tag exists):** read it at ingest and carry it into the
  sidecar `xmp:Rating` so a frame arrives already starred; a "ratings from the
  camera" filter; order the ingest plan so starred frames copy first (also what
  the print-deadline workflow needs: the 2 to 4 picks reach the desk first).
* **Shooting workflow to teach:** assign Rating to a playback control; chimp
  only the keepers; one star rule per job (5 = print pick, 4 = gallery).

## 2. An Apature bridge, like Nikon's phone app

* *Documented (press coverage, 2016-era):* SnapBridge keeps an always-on
  **Bluetooth LE** link for pairing and small transfers, and uses **Wi-Fi** for
  full-size images, video and remote control; the default is a 2 MP JPEG,
  full-size optional, up to five paired devices. Sources:
  [Digital Trends](https://digitaltrends.com/photography/new-nikon-cameras-to-use-bluetooth-for-always-on-pairing-photo-transfer),
  [Nikon USA](https://www.nikonusa.com/press-room/nikons-new-snapbridge).
* *Not found:* an open reimplementation or a packet-level description of the
  Bluetooth pairing. (See `wifi-receive.md`: two open projects have partial
  Bluetooth pairing; the Wi-Fi side, PTP/IP, is what Apature already speaks
  from the Mac.)
* **What Apature has already:** direct camera-to-Mac over Wi-Fi (Bonjour,
  pairing with the camera's code, frames received into the inbox) and an FTP
  receiver for phones and FTP-capable cameras.
* **Options for a phone bridge, in order of effort:**
  1. *No new protocol:* a small companion that watches SnapBridge's photos on
     the phone and uploads to Apature's FTP receiver (PhotoSync does this
     today; document the setup, then consider our own).
  2. *Apature phone app, Wi-Fi only:* the phone joins the camera's network or
     the hotspot, speaks PTP/IP like the Mac code (the Rust crate is reusable
     on iOS/Android), shows frames as they land, rates and captions them, and
     forwards to the Mac or the desk.
  3. *Bluetooth pairing like SnapBridge:* research track only; undocumented.
* **Questions for the owner:** phone first (iPhone or Android)? Who needs the
  frames on the phone: the photographer (cull while still at the game) or the
  editor? Is 2 MP enough for the print-deadline use?
