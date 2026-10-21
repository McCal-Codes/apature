# Apature press kit

Everything here is generated. Edit the PSD, or change the source and
regenerate; do not hand-edit the PNGs.

| File | Size | Use |
| --- | --- | --- |
| `hero-dark.png` / `.psd` | 2400 × 1500 | README front image, site hero, launch post |
| `hero-light.png` / `.psd` | 2400 × 1500 | Same, light appearance |
| `social-preview.png` / `.psd` | 1280 × 640 | GitHub social preview (Settings › Social preview), link cards |
| `feature-*.png` / `.psd` | 1600 × 1000 | One card per feature: ingest, caption, cull, crop, export |
| `COPY.md` | | Approved wording: one-liners, taglines, short and medium descriptions, posts |

Icons are in [`../design/icons`](../design/icons): four concepts, dark and light
tiles, every size, `.icns`, with the accent colour as a parameter.

## Photoshop files

Every composite has a `.psd` beside it with the same name. Layers, bottom to
top: Background (gradient), Glow, Window shadow, Window (the screenshot,
rounded, with a hairline), Subhead, Headline (and Icon, Tagline, Wordmark on
the social preview). The PSDs are not committed to git (35 MB a set); they
are attached to each GitHub release as `apature-press-kit-<version>.zip`, and
`tools/marketing_composites.py` rebuilds them.

Text layers are raster. To re-set type, hide the layer and add a text layer:

| Layer | Face | Size | Colour (dark) | Colour (light) |
| --- | --- | --- | --- | --- |
| Headline (hero) | SF Pro Bold | 96 px | `#F2F0FA` | `#1D1D22` |
| Subhead (hero) | SF Pro Regular | 40 px | `#B3B1C0` | `#4D4D58` |
| Headline (card) | SF Pro Bold | 64 px | `#F2F0FA` | |
| Subhead (card) | SF Pro Regular | 30 px, 40 px leading | `#B3B1C0` | |
| Wordmark (social) | SF Pro Bold | 104 px | `#F2F0FA` | |
| Tagline (social) | SF Pro Regular | 36 px / 30 px | `#C9BEF8` / `#B3B1C0` | |

The brand face is IBM Plex Sans (decided, not yet embedded in the app). Swap
SF Pro for Plex when the font is in the repo.

Palette: base `#121214`, ink `#1C192E`, paper `#F3F2F7`, accent `#C9BEF8`
(dark) and `#5A48C7` (light).

## Regenerating

1. Put a shoot in a folder with no outlet or client names in the folder, the
   file names or the captions. Add ratings and a caption through sidecars so
   the sheet looks used (`exiftool -o X.xmp -XMP-xmp:Rating=5 -XMP-dc:Description=...`).
2. Capture the screenshot set from the real app, headless:

   ```bash
   tools/marketing-shots.sh "/path/to/shoot" docs/screenshots
   ```

3. Build the composites and PSDs:

   ```bash
   python3 -I tools/marketing_composites.py docs/screenshots docs/design/icons/modern-a-dark/modern-a-dark-1024.png docs/marketing
   ```

   Needs Pillow and ImageMagick (`brew install imagemagick`).

4. Zip `docs/marketing` and `docs/design/icons` and attach the zip to the release.

## Rules

- Caleb's own photographs only, from his previous photojournalism work.
- No outlet, client or newspaper names anywhere in the material.
- Screenshots come from the app, not from mock-ups.
- Every composite ships with its PSD.
