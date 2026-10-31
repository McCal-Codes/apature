#!/usr/bin/env python3
"""Make Photo Mechanic's "Butler Eagle Sports" snapshots from the owner's own.

    python3 -I tools/pm_butler_sports.py            # write the three snapshots
    python3 -I tools/pm_butler_sports.py --dry-run  # say what would be written

Reads (never changes) the owner's latest Butler sports IPTC template, Butler
ingest snapshot and Butler copy snapshot in Photo Mechanic's settings folder,
and writes three NEW snapshots named "Butler Eagle Sports" beside them:

  IPTCDialogSettings/Butler Eagle Sports.snap    the metadata template
  IngestDialogSettings/Butler Eagle Sports.snap  card ingest: RAW only, archive naming, the template above
  CopyOptionsSettings/Butler Eagle Sports.snap   Copy/Move with the same naming

An existing file of that name is never overwritten. The per-game blanks are
[bracketed]; the date, credit and rights use Photo Mechanic variables.
"""
import html, os, re, shutil, sys

PM = os.path.expanduser("~/Library/Preferences/com.camerabits.PhotoMechanic")
NAME = "Butler Eagle Sports"
SLUG_PLACEHOLDER = "MMDDYY SPO SLUG Title"
RENAME = "BUTLR-{headline}{seqn}-{iptcmonth0}{iptcday0}{iptcyear2}"
CAPTION = ("[Team]'s PLAYER (##) ACTION in a football game {iptcdow}, "
           "{iptcmonthnameap} {iptcday}, {iptcyear4}, at [venue]. {credit}")
KEYWORDS = ["{headline}", "Caleb McCartney", "Butler Eagle", "high school football",
            "football", "WPIAL", "[school]", "[city]", "PA"]


def read(*p):
    with open(os.path.join(PM, *p), encoding="utf-8") as f:
        return f.read()


def set_attr(xmp, name, value):
    new, n = re.subn(rf'({re.escape(name)}=")[^"]*(")', lambda m: m.group(1) + value + m.group(2), xmp)
    if n != 1:
        raise SystemExit(f"expected exactly one {name} in the base template, found {n}")
    return new


def set_list(xmp, tag, items, kind):
    """Replace the <rdf:li> items of &lt;tag&gt;, keeping its Bag/Seq/Alt wrapper."""
    pat = re.compile(rf'(&lt;{re.escape(tag)}&gt;)(.*?)(&lt;/{re.escape(tag)}&gt;)', re.S)
    m = pat.search(xmp)
    if not m:
        raise SystemExit(f"no {tag} in the base template")
    attr = ' xml:lang="x-default"' if kind == "Alt" else ""
    lis = "".join(f"\n&lt;rdf:li{attr}&gt;{html.escape(i, quote=False)}&lt;/rdf:li&gt;" for i in items)
    body = f"\n&lt;rdf:{kind}&gt;{lis}\n&lt;/rdf:{kind}&gt;\n"
    return xmp[: m.start()] + m.group(1) + body + m.group(3) + xmp[m.end():]


def sports_xmp(base_xmp):
    x = base_xmp
    x = set_attr(x, "photoshop:City", "[city]")
    x = set_attr(x, "Iptc4xmpCore:Location", "[venue]")
    x = set_attr(x, "photoshop:Headline", SLUG_PLACEHOLDER)
    x = set_attr(x, "photoshop:TransmissionReference", SLUG_PLACEHOLDER)
    x = set_attr(x, "photoshop:Category", "SPO")
    x = set_list(x, "dc:title", [SLUG_PLACEHOLDER], "Alt")
    x = set_list(x, "dc:description", [CAPTION], "Alt")
    x = set_list(x, "dc:subject", KEYWORDS, "Bag")
    return x


def main():
    dry = "--dry-run" in sys.argv
    iptc = read("IPTCDialogSettings", "101126 SPO FB Burr-Knoch.snap")
    ingest = read("IngestDialogSettings", "Butler Eagle - 100726 SPO BGLF WPIAL Championships Ingest.snap")
    copy = read("CopyOptionsSettings", "Butler Eagle - slug rename.snap")

    m = re.search(r"(<XMPRecord>)(.*?)(</XMPRecord>)", iptc, re.S)
    new_xmp = sports_xmp(m.group(2))
    out_iptc = iptc[: m.start(2)] + new_xmp + iptc[m.end(2):]

    out_ingest = ingest
    pad = re.search(r"(<IPTCStationeryPadData>)(.*?)(</IPTCStationeryPadData>)", ingest, re.S)
    out_ingest = out_ingest[: pad.start(2)] + new_xmp + out_ingest[pad.end(2):]
    for key, val in (("IngestRenameEdit", RENAME), ("IngestUseSeqnCheck", "1"), ("IngestFilterRaw", "1")):
        out_ingest, n = re.subn(rf"({key}\t)[^\t<]*", lambda m, v=val: m.group(1) + v, out_ingest)
        if n != 1:
            raise SystemExit(f"{key} not found once in the ingest base")

    out_copy = copy
    out_copy = re.sub(r"(<RenameString>).*?(</RenameString>)", lambda m: m.group(1) + RENAME + m.group(2), out_copy)

    plan = [("IPTCDialogSettings", out_iptc), ("IngestDialogSettings", out_ingest), ("CopyOptionsSettings", out_copy)]
    for folder, text in plan:
        dest = os.path.join(PM, folder, NAME + ".snap")
        if os.path.exists(dest):
            print(f"skip   {folder}/{NAME}.snap (already exists, not overwritten)")
            continue
        print(f"{'would write' if dry else 'write'} {folder}/{NAME}.snap  ({len(text)} bytes)")
        if not dry:
            with open(dest, "w", encoding="utf-8") as f:
                f.write(text)


if __name__ == "__main__":
    main()
