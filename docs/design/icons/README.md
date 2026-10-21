# Apature icons

Four concepts, each in a dark tile and a light tile, generated from
`tools/apature_icons.py`. The app ships **aperture-dark** (see
`apps/desktop/assets/Apature.icns`); the others are alternates for marketing
and for a future per-appearance icon.

| Concept | Idea |
| --- | --- |
| `modern-a` | A chevron A built from two aperture-like strokes. Clean at 16 px. |
| `photo-frame` | Focus brackets around a picture. Reads as "photos" anywhere. |
| `aperture` | Eight overlapping blades, the classic mark. The app icon. |
| `field` | The same blades around a landscape: on location, outdoors. |

Each `<concept>-<appearance>/` folder holds PNGs at 1024, 512, 256, 128, 64,
32 and 16 px and a macOS `.icns`. The `.svg` beside it is the source.

## Changing the accent

The mark takes the same accent pair the app's Settings › Accent presets use:
the pale tint (dark appearance) and the deep tone (light appearance).

```bash
python3 tools/apature_icons.py out --accent-dark B8DCC4 --accent-light 2F7D5A   # Sage
python3 tools/apature_icons.py out --accent-dark B9D6F7 --accent-light 2C6BB8   # Sky
python3 tools/apature_icons.py out --accent-dark F6C9B3 --accent-light B8552C   # Peach
python3 tools/apature_icons.py out --accent-dark F3BED1 --accent-light B23A6B   # Rose
```

Rasterising needs a renderer that handles SVG gradients, masks and filters.
Safari or Chrome do; ImageMagick's built-in SVG renderer does not. The
simplest path: open the SVG in a browser, draw it to a 1024 canvas, save the
PNG, then build sizes and the `.icns` with `iconutil` as the packaging step in
`docs/marketing/README.md` describes.

Palette: base `#121214`, background `#F3F2F7`, accent `#C9BEF8` (dark) and
`#5A48C7` (light).
