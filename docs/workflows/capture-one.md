# From Apature to Capture One

For the days a frame needs a real edit: cull and caption in Apature, edit in
Capture One, export from there. Apature never edits the RAW; everything it
knows about a photo sits in the `.XMP` sidecar beside it, and Capture One
reads that sidecar.

## In Apature

1. Rate, label and caption as usual. Ratings, colour label and caption go in
   the sidecar (`xmp:Rating`, `xmp:Label`, `dc:description`); the headline,
   credit and keywords go in with them.
2. Select the photos (for example the 5 stars) and use **Image › Edit Photos**
   or right-click › **Edit Photos**. Settings › General › **Edit photos in**
   picks the app; empty means Capture One when it is installed, else the
   system default. Tick "Send a RAW's JPEG partner too" for RAW+JPEG.
3. Files that a cloud drive has not downloaded are refused with a count, not
   hung on.

## In Capture One (once)

Preferences › Image › Metadata:

* **Auto Sync Sidecar XMP: Load** (reads the sidecar when the photos come in;
  "None" needs Image › Load Metadata after each import).
* Tick **Prefer Sidecar XMP over Embedded Metadata**.

Image tab: **Automatically Apply Crop Metadata from File or Sidecar XMP**
(off by default) is what lets a crop made in Apature show up. Apature writes
the crop as `crs:CropTop/Left/Bottom/Right` with `crs:CropAngle` 0.

## What carries over, and what does not

| from Apature | in Capture One |
|---|---|
| star rating | yes, the most reliable field |
| caption, headline, credit, keywords | yes, with Load Metadata |
| colour label | not confirmed: `xmp:Label` is written as a word ("Purple") and the number in `photomechanic:ColorClass`; reports say Capture One handles Photo Mechanic's colour classes only through a setting in Photo Mechanic, and has ignored an `xmp:Label` element. Test one photo. |
| crop | only with the crop option above; not tested |
| straighten angle | **no.** It is stored as `apature:Straighten`, which Capture One does not know. Straighten again in Capture One. |
| rotation | `tiff:Orientation` is in the sidecar; not tested |

Sources: Capture One's support threads on Photo Mechanic sidecars, and two
workflow write-ups ([imagealchemist](https://imagealchemist.net/sync-metadata-between-photo-mechanic-and-capture-one),
[jmpeltier](https://www.jmpeltier.com/photo-mechanic-capture-one-workflow/)).
They describe Photo Mechanic; Apature's sidecars have the same fields but
nothing here has been run against Capture One yet. **Test with one photo
before a batch**, in a throwaway session.

## After editing

Capture One never writes to the RAW. Export JPEGs from it (keep the original
names), or send the edited TIFF/JPEG back through Apature's Export. Finish
metadata first: Photo Mechanic and Apature do not look for sidecars beside
processed JPEG and TIFF files.
