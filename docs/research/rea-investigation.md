# Studying Photo Mechanic with REA

[REA](https://github.com/morluto/rea) is a reverse-engineering toolkit for
coding agents: it gives an agent a structured way to investigate how an
existing application behaves (black-box tests, runtime observation, and where
permitted, binary analysis) and to write that evidence up as a specification.
It is a research tool for this project, never a dependency of it.

## Ground rules

* Prefer authorised black-box observation and public documentation. Camera
  Bits' own docs (docs.camerabits.com) cover most of what matters: variables,
  code replacement syntax, sidecar behaviour, ingest options.
* Any decompilation must respect the Photo Mechanic licence on the machine
  it runs on. The output of this research is *behaviour and file formats*,
  not code. Nothing proprietary is copied into this repository.
* The investigation must not modify the Photo Mechanic installation or its
  preferences, and must not be used to bypass licensing.

## Setup

```bash
npx rea-agents setup     # choose Claude Code; review the files it adds
```

## Investigation prompt

> Investigate Photo Mechanic's observable behaviour and produce an
> evidence-backed specification for an independent Rust implementation.
> Prioritise: contact-sheet rendering and scroll performance, which embedded
> preview is used at which zoom level, keyboard culling (colour class, rating,
> tag) and exactly what each writes to the XMP sidecar, RAW+JPEG pairing
> rules, IPTC/XMP field mapping, code replacement and variable expansion in
> captions, and ingest (destination naming, verification, sidecar creation).
> Record observations, supporting evidence, timings, and implementation
> proposals. Do not copy proprietary code, do not bypass licensing controls,
> do not modify the application, and do not write production code.

## What to measure first

| question | how | feeds |
|---|---|---|
| Which preview does PM show at thumbnail / fit / 100 %? | open a NEF, watch file reads with `fs_usage`, compare bytes to `apature-cli previews` | `apature-preview::Source::best_for` |
| What does PM write when you press `1`, `⌘3`, `T`? | mark a file, diff the sidecar | `apature-metadata::xmp` tests |
| Does PM write `xmp:Label` with the class *name* or colour? | same diff, after renaming a class in prefs | `ColorClassScheme` |
| How are pairs shown when only one side has a sidecar? | create `a.NEF`, `a.JPG`, mark one | pairing rules |
| Time to first thumbnail on a 1 000-file folder, cold vs warm | stopwatch + `fs_usage`; compare to `apature-cli bench` | benchmarks |
| What does Ingest write besides the copy? | ingest to a temp folder, list everything | `apature-ingest` |

Findings go in this folder as dated markdown files with the evidence
attached (sidecar diffs, timings, screenshots), one topic per file.
