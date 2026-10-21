"""Marketing composites for Apature from the headless screenshots and the icon.

    python3 -I composites.py SHOTS_DIR ICON_PNG OUT_DIR

Every piece is built as named full-canvas layers. Each is written twice: a
flattened PNG, and a layered PSD (assembled with ImageMagick) so the layers
can be moved, recoloured or retyped in Photoshop. Text layers are raster; the
type used is noted in docs/marketing/README.md.
"""
import os
import subprocess
import sys
import tempfile

from PIL import Image, ImageDraw, ImageFilter, ImageFont

SHOTS, ICON, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
os.makedirs(OUT, exist_ok=True)

LAV = (0xC9, 0xBE, 0xF8)
DEEP = (0x5A, 0x48, 0xC7)
BASE = (0x12, 0x12, 0x14)
INK = (0x1C, 0x19, 0x2E)
TEXT_D = (0xF2, 0xF0, 0xFA)
TEXT2_D = (0xB3, 0xB1, 0xC0)
TEXT_L = (0x1D, 0x1D, 0x22)
TEXT2_L = (0x4D, 0x4D, 0x58)


def font(size, weight="Bold"):
    for path in ("/System/Library/Fonts/SFNS.ttf", "/System/Library/Fonts/Helvetica.ttc"):
        try:
            f = ImageFont.truetype(path, size)
            try:
                f.set_variation_by_name(weight)
            except Exception:
                pass
            return f
        except Exception:
            continue
    return ImageFont.load_default()


def gradient(w, h, c0, c1):
    base = Image.new("RGB", (1, h))
    px = base.load()
    for i in range(h):
        t = i / max(h - 1, 1)
        px[0, i] = tuple(int(c0[k] + (c1[k] - c0[k]) * t) for k in range(3))
    return base.resize((w, h)).convert("RGBA")


def glow_layer(size, cx, cy, r, color, alpha):
    layer = Image.new("RGBA", size, (0, 0, 0, 0))
    ImageDraw.Draw(layer).ellipse((cx - r, cy - r, cx + r, cy + r), fill=color + (alpha,))
    return layer.filter(ImageFilter.GaussianBlur(r * 0.6))


def text_layer(size, items):
    """items: [(x, y, text, font, fill)] on a transparent canvas."""
    layer = Image.new("RGBA", size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    for x, y, t, f, fill in items:
        d.text((x, y), t, font=f, fill=fill)
    return layer


def wrap(text, f, max_w, d):
    words, lines, cur = text.split(), [], ""
    for w in words:
        trial = (cur + " " + w).strip()
        if d.textlength(trial, font=f) > max_w and cur:
            lines.append(cur)
            cur = w
        else:
            cur = trial
    lines.append(cur)
    return lines


def window_layers(size, shot, width, x, y, radius=28, dark=True):
    """Two layers: the window's drop shadow and the window itself (rounded,
    hairline), placed at (x, y) on a canvas of `size`."""
    im = Image.open(shot).convert("RGB")
    h = int(im.height * width / im.width)
    im = im.resize((width, h), Image.LANCZOS)
    shadow = Image.new("RGBA", size, (0, 0, 0, 0))
    ImageDraw.Draw(shadow).rounded_rectangle((x, y + 30, x + width, y + h + 30), radius, fill=(0, 0, 0, 170 if dark else 90))
    shadow = shadow.filter(ImageFilter.GaussianBlur(34))
    mask = Image.new("L", (width, h), 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, width - 1, h - 1), radius, fill=255)
    frame = Image.new("RGBA", (width, h), (0, 0, 0, 0))
    frame.paste(im, (0, 0), mask)
    ImageDraw.Draw(frame).rounded_rectangle((0, 0, width - 1, h - 1), radius, outline=(255, 255, 255, 40) if dark else (0, 0, 0, 40), width=2)
    win = Image.new("RGBA", size, (0, 0, 0, 0))
    win.alpha_composite(frame, (x, y))
    return shadow, win


def write(name, size, layers):
    """layers: [(label, RGBA image of canvas size)], bottom first. Writes
    OUT/name.png (flattened) and OUT/name.psd (layered)."""
    flat = Image.new("RGBA", size, (0, 0, 0, 255))
    for _, layer in layers:
        flat.alpha_composite(layer)
    flat.convert("RGB").save(f"{OUT}/{name}.png", optimize=True)
    with tempfile.TemporaryDirectory() as tmp:
        flat_p = f"{tmp}/flat.png"
        flat.save(flat_p)
        cmd = ["magick", flat_p]
        for i, (label, layer) in enumerate(layers):
            p = f"{tmp}/{i:02d}.png"
            layer.save(p)
            cmd += ["-label", label, p]
        cmd += ["-compress", "rle", f"psd:{OUT}/{name}.psd"]
        subprocess.run(cmd, check=True)
    print(name)


def hero(name, shot, dark):
    size = (2400, 1500)
    bg = gradient(*size, INK, BASE) if dark else gradient(*size, (0xFF, 0xFF, 0xFF), (0xE7, 0xE3, 0xF8))
    glow = glow_layer(size, 300, 200, 700, DEEP if dark else LAV, 120 if dark else 160)
    fg, fg2 = (TEXT_D, TEXT2_D) if dark else (TEXT_L, TEXT2_L)
    headline = text_layer(size, [(120, 120, "Card in. Cull. Caption. Send.", font(96), fg)])
    sub = text_layer(size, [(124, 250, "The Photo Mechanic loop, rebuilt in Rust for the Mac. Free and open source.", font(40, "Regular"), fg2)])
    shadow, win = window_layers(size, shot, 2000, 350, 420, dark=dark)
    write(name, size, [("Background", bg), ("Glow", glow), ("Window shadow", shadow), ("Window", win), ("Subhead", sub), ("Headline", headline)])


def social():
    size = (1280, 640)
    bg = gradient(*size, INK, BASE)
    glow = glow_layer(size, 180, 120, 420, DEEP, 130)
    icon = Image.new("RGBA", size, (0, 0, 0, 0))
    icon.alpha_composite(Image.open(ICON).convert("RGBA").resize((200, 200), Image.LANCZOS), (80, 96))
    wordmark = text_layer(size, [(310, 104, "Apature", font(104), TEXT_D)])
    tag = text_layer(size, [(314, 226, "Ingest, cull, caption, export, deliver.", font(36, "Regular"), LAV), (314, 276, "Free, open source, macOS. Originals never modified.", font(30, "Regular"), TEXT2_D)])
    shadow, win = window_layers(size, f"{SHOTS}/sheet.png", 1100, 250, 450, radius=22)
    write("social-preview", size, [("Background", bg), ("Glow", glow), ("Window shadow", shadow), ("Window", win), ("Icon", icon), ("Tagline", tag), ("Wordmark", wordmark)])


def card(name, shot, title, line):
    size = (1600, 1000)
    bg = gradient(*size, INK, BASE)
    glow = glow_layer(size, 200, 120, 500, DEEP, 100)
    headline = text_layer(size, [(80, 70, title, font(64), TEXT_D)])
    f = font(30, "Regular")
    d = ImageDraw.Draw(Image.new("RGBA", (1, 1)))
    lines = wrap(line, f, 1440, d)[:2]
    sub = text_layer(size, [(82, 156 + i * 40, l, f, TEXT2_D) for i, l in enumerate(lines)])
    shadow, win = window_layers(size, shot, 1440, 80, 320, radius=22)
    write(name, size, [("Background", bg), ("Glow", glow), ("Window shadow", shadow), ("Window", win), ("Subhead", sub), ("Headline", headline)])


hero("hero-dark", f"{SHOTS}/sheet.png", True)
hero("hero-light", f"{SHOTS}/sheet-light.png", False)
social()
card("feature-ingest", f"{SHOTS}/ingest.png", "Ingest you can trust", "Day chips before you copy, a live example of every name, SHA-256 at every destination, a ledger so a card is never copied twice.")
card("feature-caption", f"{SHOTS}/caption.png", "Captions at deadline speed", "Assignment pad, code replacements, the variables Photo Mechanic users already know, save to the whole selection.")
card("feature-cull", f"{SHOTS}/compare.png", "Cull without leaving the keyboard", "Ratings, colour classes, burst stacks, compare up to four frames, a loupe, 1:1 with one shared magnification.")
card("feature-crop", f"{SHOTS}/crop.png", "Crop and rotate in the sidecar", "Edits live in the XMP next to the file. The original is never modified, and Photo Mechanic reads the result.")
card("feature-export", f"{SHOTS}/export.png", "Export and deliver", "JPEG or JPEG XL, presets per outlet, EXIF, XMP and IPTC-IIM kept, then folder, FTP, FTPS or SFTP with a sent ledger.")
print("composites ok")
