import json, sys, collections
m = json.load(sys.stdin)
ws = set(m["workspace_members"])
by = collections.defaultdict(list)
for p in m["packages"]:
    if p["id"] in ws:
        continue
    lic = p.get("license") or "UNKNOWN(file:%s)" % p.get("license_file")
    by[lic].append(p["name"])
for lic, names in sorted(by.items(), key=lambda kv: -len(kv[1])):
    print(f"{len(names):4d}  {lic}")
print("--- flagged (GPL/LGPL/AGPL/CDDL/MPL/UNKNOWN):")
for lic, names in by.items():
    if any(k in lic.upper() for k in ["GPL", "CDDL", "MPL", "UNKNOWN"]):
        print(lic, "->", ", ".join(sorted(names)))
print("--- direct deps of workspace crates:")
direct = set()
for p in m["packages"]:
    if p["id"] in ws:
        for d in p["dependencies"]:
            if d["kind"] in (None, "build"):
                direct.add(d["name"])
seen = set()
for p in m["packages"]:
    if p["name"] in direct and p["id"] not in ws and p["name"] not in seen:
        seen.add(p["name"])
        print(f"  {p['name']:18s} {p['version']:10s} {p.get('license')}")
print("total third-party packages:", sum(len(v) for v in by.values()))
