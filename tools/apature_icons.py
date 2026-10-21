#!/usr/bin/env python3
"""Apature icon concepts as parametric SVG.

    python3 apature_icons.py OUT_DIR [--accent-dark C9BEF8] [--accent-light 5A48C7]

Writes <concept>-<appearance>.svg for the four concepts (modern-a, photo-frame,
aperture, field) in the dark and light tile appearances. The accent pair is the
same one the app's Settings › Accent presets use, so an icon can match any
preset: pass the pale tint as --accent-dark and the deep tone as --accent-light.

Raster output (PNG, .icns) needs a real SVG renderer for the gradients and
masks; see render.py / the browser step in the marketing notes.
"""
import math
import os
import sys


def hex2rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i : i + 2], 16) for i in (0, 2, 4))


def rgb2hex(c):
    return "#%02X%02X%02X" % tuple(max(0, min(255, int(round(v)))) for v in c)


def mix(a, b, t):
    a, b = hex2rgb(a), hex2rgb(b)
    return rgb2hex(tuple(a[i] + (b[i] - a[i]) * t for i in range(3)))


def blades(n, d, w, R, hi, lo, seam, prefix, cx=512, cy=512, rot=-90, seam_w=10):
    """n aperture blades. Blade i is a thick band (width w) along a chord whose
    midpoint sits d from the centre, so the bands leave a regular n-gon opening
    of apothem d - w/2 and reach the rim at d + w/2. Each band is drawn over the
    previous one and carries its own gradient, which is what gives the pinwheel
    shading; a thin seam in the disc colour runs along each band's inner edge,
    from the opening out to the rim, the way real blades overlap."""
    out = []
    seams = []
    for i in range(n):
        a = math.radians(rot + i * 360 / n)
        nx, ny = math.cos(a), math.sin(a)       # unit normal of chord i
        tx, ty = -ny, nx                        # unit tangent
        mx, my = cx + nx * d, cy + ny * d       # chord midpoint
        half = math.sqrt(max(R * R - d * d, 1)) + 80
        x1, y1 = mx - tx * half, my - ty * half
        x2, y2 = mx + tx * half, my + ty * half
        gid = f"{prefix}{i}"
        out.append(
            f'<linearGradient id="{gid}" gradientUnits="userSpaceOnUse" x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}">'
            f'<stop offset="0" stop-color="{hi}"/><stop offset="1" stop-color="{lo}"/></linearGradient>'
            f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" stroke="url(#{gid})" stroke-width="{w}"/>'
        )
        # inner edge: parallel line at distance d - w/2, only the half that runs
        # away from the opening in the tangent direction (one seam per blade)
        e = d - w / 2
        ex, ey = cx + nx * e, cy + ny * e
        # the seam starts at the opening corner and runs outward with the tangent
        corner = e * math.tan(math.pi / n)
        sx1, sy1 = ex + tx * corner, ey + ty * corner
        sx2, sy2 = ex + tx * (half + 200), ey + ty * (half + 200)
        seams.append(
            f'<line x1="{sx1:.1f}" y1="{sy1:.1f}" x2="{sx2:.1f}" y2="{sy2:.1f}" stroke="{seam}" stroke-width="{seam_w}" stroke-linecap="round"/>'
        )
    return "\n".join(out) + "\n" + "\n".join(seams)


def tile(appearance):
    if appearance == "dark":
        return (
            '<linearGradient id="tile" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2A2A31"/><stop offset="1" stop-color="#121214"/></linearGradient>',
            '<rect x="100" y="100" width="824" height="824" rx="185" fill="url(#tile)"/>'
            '<rect x="100.5" y="100.5" width="823" height="823" rx="185" fill="none" stroke="#FFFFFF" stroke-opacity="0.08"/>',
        )
    return (
        '<linearGradient id="tile" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#FFFFFF"/><stop offset="1" stop-color="#EEEDF3"/></linearGradient>',
        '<rect x="100" y="100" width="824" height="824" rx="185" fill="url(#tile)"/>'
        '<rect x="100.5" y="100.5" width="823" height="823" rx="185" fill="none" stroke="#000000" stroke-opacity="0.10"/>',
    )


def palette(appearance, pale, deep):
    """Gradient ends for the mark on each tile. Dark tiles carry the pale tint
    down to a mid tone; light tiles carry a mid tone down to the deep one."""
    mid = mix(pale, deep, 0.55)
    if appearance == "dark":
        return pale, mid, mix(deep, "#000000", 0.15)
    return mix(pale, deep, 0.25), deep, mix(deep, "#000000", 0.25)


def svg(defs, body, appearance):
    tdef, trect = tile(appearance)
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">\n'
        f"<defs>{tdef}\n{defs}\n"
        '<filter id="drop" x="-30%" y="-30%" width="160%" height="160%"><feDropShadow dx="0" dy="14" stdDeviation="18" flood-color="#000" flood-opacity="0.35"/></filter>\n'
        '<filter id="glow" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="16"/></filter>\n'
        f"</defs>\n{trect}\n{body}\n</svg>\n"
    )


def modern_a(ap, pale, deep):
    hi, mid, lo = palette(ap, pale, deep)
    defs = (
        f'<linearGradient id="legL" x1="0" y1="1" x2="1" y2="0"><stop offset="0" stop-color="{mid}"/><stop offset="1" stop-color="{hi}"/></linearGradient>'
        f'<linearGradient id="legR" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{hi}"/><stop offset="1" stop-color="{lo}"/></linearGradient>'
    )
    w = 150
    body = (
        f'<line x1="512" y1="330" x2="748" y2="724" stroke="url(#legR)" stroke-width="{w}" stroke-linecap="round" filter="url(#drop)"/>'
        f'<line x1="276" y1="724" x2="512" y2="330" stroke="url(#legL)" stroke-width="{w}" stroke-linecap="round" filter="url(#drop)"/>'
    )
    return svg(defs, body, ap)


def photo_frame(ap, pale, deep):
    hi, mid, lo = palette(ap, pale, deep)
    defs = (
        f'<linearGradient id="fr" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{hi}"/><stop offset="1" stop-color="{mid}"/></linearGradient>'
        f'<linearGradient id="mt" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{mid}"/><stop offset="1" stop-color="{lo}"/></linearGradient>'
        '<clipPath id="inner"><rect x="300" y="300" width="424" height="424" rx="40"/></clipPath>'
    )
    s, L, w = 232, 120, 46  # bracket corner, arm length, stroke
    brackets = []
    for (x, y, sx, sy) in [(s, s, 1, 1), (1024 - s, s, -1, 1), (s, 1024 - s, 1, -1), (1024 - s, 1024 - s, -1, -1)]:
        brackets.append(
            f'<path d="M{x} {y + sy * L} L{x} {y} L{x + sx * L} {y}" fill="none" stroke="url(#fr)" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round"/>'
        )
    body = (
        "\n".join(brackets)
        + f'<circle cx="512" cy="412" r="64" fill="{hi}"/>'
        + '<g clip-path="url(#inner)">'
        + f'<polygon points="300,700 440,540 540,640 620,580 724,700" fill="url(#mt)"/>'
        + f'<polygon points="300,724 300,700 724,700 724,724" fill="{lo}"/>'
        + "</g>"
    )
    return svg(defs, body, ap)


def aperture(ap, pale, deep):
    hi, mid, lo = palette(ap, pale, deep)
    disc = "#0E0E11" if ap == "dark" else "#FFFFFF"
    defs = '<clipPath id="ring"><circle cx="512" cy="512" r="318"/></clipPath>'
    body = (
        f'<circle cx="512" cy="512" r="334" fill="{mid}" fill-opacity="0.35" filter="url(#glow)"/>'
        f'<circle cx="512" cy="512" r="318" fill="{disc}"/>'
        '<g clip-path="url(#ring)">'
        + blades(8, 218, 200, 318, hi, mix(lo, "#000000", 0.22), disc, "ap")
        + "</g>"
        + f'<circle cx="512" cy="512" r="318" fill="none" stroke="{"#FFFFFF" if ap == "dark" else "#000000"}" stroke-opacity="0.10" stroke-width="3"/>'
    )
    return svg(defs, body, ap)


def field(ap, pale, deep):
    hi, mid, lo = palette(ap, pale, deep)
    sky0, sky1 = (hi, mid) if ap == "dark" else (mix(pale, "#FFFFFF", 0.3), mid)
    defs = (
        '<clipPath id="ring"><circle cx="512" cy="512" r="330"/></clipPath>'
        '<mask id="annulus"><circle cx="512" cy="512" r="330" fill="#fff"/><circle cx="512" cy="512" r="250" fill="#000"/></mask>'
        '<clipPath id="scene"><circle cx="512" cy="512" r="250"/></clipPath>'
        f'<linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{sky0}"/><stop offset="1" stop-color="{sky1}"/></linearGradient>'
        f'<linearGradient id="far" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{mid}"/><stop offset="1" stop-color="{lo}"/></linearGradient>'
        f'<linearGradient id="water" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{lo}"/><stop offset="1" stop-color="{mix(lo, "#000000", 0.35)}"/></linearGradient>'
    )
    body = (
        f'<circle cx="512" cy="512" r="346" fill="{mid}" fill-opacity="0.35" filter="url(#glow)"/>'
        '<g clip-path="url(#scene)">'
        '<rect x="262" y="262" width="500" height="500" fill="url(#sky)"/>'
        f'<circle cx="560" cy="470" r="52" fill="{"#FFFFFF" if ap == "dark" else hi}" fill-opacity="0.95"/>'
        '<polygon points="262,600 360,470 430,540 500,430 580,560 650,480 762,600 762,640 262,640" fill="url(#far)"/>'
        f'<polygon points="262,640 340,560 420,620 520,520 610,610 700,560 762,640 762,700 262,700" fill="{lo}"/>'
        '<rect x="262" y="640" width="500" height="130" fill="url(#water)"/>'
        f'<rect x="520" y="655" width="80" height="6" rx="3" fill="{"#FFFFFF" if ap == "dark" else hi}" fill-opacity="0.6"/>'
        f'<rect x="505" y="680" width="110" height="6" rx="3" fill="{"#FFFFFF" if ap == "dark" else hi}" fill-opacity="0.35"/>'
        "</g>"
        '<g mask="url(#annulus)">'
        + f'<circle cx="512" cy="512" r="330" fill="{"#0E0E11" if ap == "dark" else "#FFFFFF"}"/>'
        + blades(8, 350, 200, 400, hi, mix(lo, "#000000", 0.2), "#0E0E11" if ap == "dark" else "#FFFFFF", "fd", seam_w=8)
        + "</g>"
        + f'<circle cx="512" cy="512" r="250" fill="none" stroke="{"#000000" if ap == "dark" else "#FFFFFF"}" stroke-opacity="0.35" stroke-width="4"/>'
    )
    return svg(defs, body, ap)


CONCEPTS = {"modern-a": modern_a, "photo-frame": photo_frame, "aperture": aperture, "field": field}


def main():
    args = sys.argv[1:]
    out = args[0] if args and not args[0].startswith("--") else "icons-out"
    pale, deep = "#C9BEF8", "#5A48C7"
    if "--accent-dark" in args:
        pale = "#" + args[args.index("--accent-dark") + 1].lstrip("#")
    if "--accent-light" in args:
        deep = "#" + args[args.index("--accent-light") + 1].lstrip("#")
    os.makedirs(out, exist_ok=True)
    for name, fn in CONCEPTS.items():
        for ap in ("dark", "light"):
            path = os.path.join(out, f"{name}-{ap}.svg")
            open(path, "w").write(fn(ap, pale, deep))
            print(path)


if __name__ == "__main__":
    main()
