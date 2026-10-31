#!/usr/bin/env python3
"""Make Photo Mechanic snapshots for ONE assignment from its Apature assignment file.

    python3 -I tools/pm_from_assignment.py "103126 SPO FB Highlands-North Catholic.txt"
    python3 -I tools/pm_from_assignment.py job.txt --base "Butler Eagle Sports" --dry-run

Fill the job in once, in the assignment file (slug, venue, city, team, ...).
This writes three NEW snapshots named after the slug into Photo Mechanic's
settings folder, built from a base set you already have (default "Butler
Eagle Sports", made by tools/pm_butler_sports.py):

  IPTCDialogSettings/<slug>.snap      the metadata template, filled in
  IngestDialogSettings/<slug>.snap    card ingest with that template
  CopyOptionsSettings/<slug>.snap     Copy/Move with the same naming

Photo Mechanic's own variables stay variables ({iptcdow}, {credit}, {year4},
{headline}...): they are filled per photo. Nothing here is a placeholder: the
script refuses a file that still has a [bracket], an MMDDYY or no slug, and
never overwrites an existing snapshot. Photo Mechanic's user/client variables
are a fixed set of fields, so the per-job values cannot be PM variables; the
assignment file is the one place they live.
"""
import argparse, html, os, re, sys

PM_DEFAULT = os.path.expanduser("~/Library/Preferences/com.camerabits.PhotoMechanic")
STATES = {"pa.": "Pennsylvania", "ohio": "Ohio", "w.va.": "West Virginia", "n.y.": "New York",
          "md.": "Maryland", "va.": "Virginia", "n.j.": "New Jersey"}


def parse_assignment(path):
    a = {"codes": {}}
    for raw in open(path, encoding="utf-8"):
        line = raw.strip()
        if not line or line.startswith("#") or ":" not in line:
            continue
        k, v = line.split(":", 1)
        k, v = k.strip(), v.strip()
        if k == "code":
            if "=" in v:
                n, r = v.split("=", 1)
                a["codes"][n.strip()] = r.lstrip(" ") if r.startswith("  ") else r.strip()
                # a replacement keeps one leading space only if written with two spaces after "="
                if r.startswith("  "):
                    a["codes"][n.strip()] = " " + r.strip()
        else:
            a[k] = v
    return a


def unfilled(a):
    bad = []
    slug = a.get("slug", "")
    if not slug:
        bad.append("slug: is empty")
    for k, v in a.items():
        if k == "codes":
            continue
        if re.search(r"\[[^\]]*\]", v) or "MMDDYY" in v or "YYYY" in v:
            bad.append(f"{k}: {v}")
    return bad


def section(slug):
    m = re.match(r"^\d{6} ([A-Z]{2,4}) ", slug)
    return m.group(1) if m else None


def set_attr(xmp, name, value):
    value = html.escape(value, quote=True).replace("&#x27;", "'")
    new, n = re.subn(rf'({re.escape(name)}=")[^"]*(")', lambda m: m.group(1) + value + m.group(2), xmp)
    if n != 1:
        raise SystemExit(f"expected one {name} in the base template, found {n}")
    return new


def set_list(xmp, tag, items, kind):
    pat = re.compile(rf'(&lt;{re.escape(tag)}&gt;)(.*?)(&lt;/{re.escape(tag)}&gt;)', re.S)
    m = pat.search(xmp)
    if not m:
        raise SystemExit(f"no {tag} in the base template")
    attr = ' xml:lang="x-default"' if kind == "Alt" else ""
    lis = "".join(f"\n&lt;rdf:li{attr}&gt;{html.escape(i, quote=False)}&lt;/rdf:li&gt;" for i in items)
    return xmp[: m.start()] + m.group(1) + f"\n&lt;rdf:{kind}&gt;{lis}\n&lt;/rdf:{kind}&gt;\n" + m.group(3) + xmp[m.end():]


def fill(base_xmp, a):
    slug = a["slug"]
    x = base_xmp
    x = set_attr(x, "photoshop:Headline", slug)
    x = set_attr(x, "photoshop:TransmissionReference", slug)
    cat = a.get("category") or section(slug)
    if cat:
        x = set_attr(x, "photoshop:Category", cat)
    x = set_attr(x, "photoshop:City", a.get("city", ""))
    x = set_attr(x, "Iptc4xmpCore:Location", a.get("venue", ""))
    state = a.get("state", "")
    x = set_attr(x, "photoshop:State", STATES.get(state.lower(), state))
    credit = a.get("credit") or a.get("photographer", "")
    if credit:
        x = set_attr(x, "photoshop:Credit", credit)
        x = set_attr(x, "photoshop:Instructions", f"Mandatory credit: {credit}")
    if a.get("source"):
        x = set_attr(x, "photoshop:Source", a["source"])
    if a.get("caption_writer"):
        x = set_attr(x, "photoshop:CaptionWriter", a["caption_writer"])
    if a.get("date"):
        for k in ("photoshop:DateCreated", "xmp:CreateDate"):
            x = set_attr(x, k, a["date"] + "T00:00:00-04:00")
    x = set_list(x, "dc:title", [slug], "Alt")
    if a.get("copyright"):
        x = set_list(x, "dc:rights", [a["copyright"]], "Alt")
    team = a.get("team", "").strip()
    tail = a["codes"].get("tail")
    if tail is not None:
        caption = f"{team + ' ' if team else ''}PLAYER (##) ACTION{tail}"
        x = set_list(x, "dc:description", [caption], "Alt")
    creator = (a.get("photographer") or credit).split("/")[0].strip()
    kws = ["{headline}"] + ([creator] if creator else []) + ([a["source"]] if a.get("source") else []) \
        + [k.strip() for k in a.get("keywords", "").split(";") if k.strip()]
    seen, out = set(), []
    for k in kws:
        if k not in seen:
            seen.add(k)
            out.append(k)
    return set_list(x, "dc:subject", out, "Bag")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("assignment")
    ap.add_argument("--base", default="Butler Eagle Sports")
    ap.add_argument("--pm-dir", default=PM_DEFAULT)
    ap.add_argument("--dry-run", action="store_true")
    o = ap.parse_args()
    a = parse_assignment(o.assignment)
    bad = unfilled(a)
    if bad:
        print("Fill these in the assignment file first (nothing was written):", *bad, sep="\n  ")
        sys.exit(2)
    name = re.sub(r"[/:]", "-", a["slug"])
    rd = lambda *p: open(os.path.join(o.pm_dir, *p), encoding="utf-8").read()
    iptc = rd("IPTCDialogSettings", o.base + ".snap")
    ingest = rd("IngestDialogSettings", o.base + ".snap")
    copy = rd("CopyOptionsSettings", o.base + ".snap")
    m = re.search(r"(<XMPRecord>)(.*?)(</XMPRecord>)", iptc, re.S)
    xmp = fill(m.group(2), a)
    out_iptc = iptc[: m.start(2)] + xmp + iptc[m.end(2):]
    pad = re.search(r"(<IPTCStationeryPadData>)(.*?)(</IPTCStationeryPadData>)", ingest, re.S)
    out_ingest = ingest[: pad.start(2)] + xmp + ingest[pad.end(2):]
    for folder, text in (("IPTCDialogSettings", out_iptc), ("IngestDialogSettings", out_ingest), ("CopyOptionsSettings", copy)):
        dest = os.path.join(o.pm_dir, folder, name + ".snap")
        if os.path.exists(dest):
            print(f"skip   {folder}/{name}.snap (exists, not overwritten)")
            continue
        print(f"{'would write' if o.dry_run else 'write'} {folder}/{name}.snap")
        if not o.dry_run:
            with open(dest, "w", encoding="utf-8") as f:
                f.write(text)


if __name__ == "__main__":
    main()
