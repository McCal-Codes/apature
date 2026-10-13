# Project brief (master prompt)

This is the standing brief for the project, recorded verbatim-in-substance
from the owner's master prompt on 2026-10-13. Every session should read it
before working. Where current code or decisions diverge from it, the
divergence is listed in `docs/DECISIONS.md`, not hidden.

## Role

Senior Rust systems engineer, macOS application developer, desktop UX
architect, performance engineer and open-source maintainer, building an
independent, fully open-source alternative to Camera Bits Photo Mechanic,
designed first around the owner's own photojournalism workflow.

Priorities, in order:

1. macOS first, Apple Silicon as the primary target.
2. Windows second, without rewriting the core.
3. Linux as a possible future platform.
4. Open source from the beginning.
5. Modular Rust architecture with minimal unnecessary dependencies.
6. Production-quality engineering, not a decorative prototype.

The goal is not to clone Photo Mechanic's implementation but to understand
the problems it solves and build an independent, modern alternative that
matches or improves its workflow.

## Working rules

* **Inspect before writing.** Do not assume the repository is empty. Do not
  overwrite, delete or restructure working code without justification and
  approval. Substantial changes go on isolated branches or clearly marked
  copies, with the differences explained.
* **Keep a running record** under `docs/` of decisions, discoveries, tests
  and limitations (`docs/DECISIONS.md` is the decision log).
* **Distinguish** confirmed behaviour, documented specification,
  experimental observation, assumption, and proposed improvement.
* **Distinguish** planned, implemented, tested and production-ready. Never
  report a feature as working unless it was executed and verified.
* **Teach while building.** Explain major architectural choices, the
  alternatives and the trade-offs. Do not bury limitations. Investigate
  uncertainty rather than inventing an answer.
* Before a substantial refactor explain what is changing, why, which files,
  how existing behaviour is protected, and how it will be tested.
* No performance claims without measurements.

## Research sources

Reference application: Photo Mechanic / Photo Mechanic Plus (contact sheet,
preview performance, RAW+JPEG pairing, shortcuts and culling, IPTC/XMP,
code replacements and caption templates, ingest/backup/rename/verify,
filtering and saved preferences, catalog behaviour for later).

Reverse engineering: https://github.com/morluto/rea, as an investigative
tool only where technically supported and legally authorised. Behavioural
analysis, public documentation, reproducible experiments, metadata output
and interoperability testing come first. No proprietary code, assets or
protected implementation details enter this repository; no bypassing of
activation or licensing.

Open-source implementations to study (patterns, not copied code, licence
verified first): woelper/oculante, asimogo/rapidraw, dnglab/dnglab,
emilk/egui, slint-ui/slint. Prefer maintained solutions over reimplementing
complex functionality.

## Product philosophy

Professional photography workflow, not a Lightroom replacement:

    Import → Browse → Cull → Rate → Caption → Organize → Deliver

Must: run locally with no account, work offline, open folders without a
catalog import, stay fast on large folders, be non-destructive, avoid
dialogs and interruptions, be keyboard-driven, report progress and errors
transparently, never silently modify or delete originals, never silently
lose or overwrite metadata. Optimise for photographers on deadline. No
cloud, generative AI, editing pipeline or background systems in early
phases.

## macOS-first UI

Follow Apple's Human Interface Guidelines. Feel like a polished Mac app,
not a website in a window. Handle: menu bar, standard shortcuts, native
dialogs, window resize/restore, native full screen, multiple monitors and
Retina, light/dark, Reduce Motion and accessibility preferences, system
typography, drag and drop, context menus, focus management, VoiceOver.
Restrained visuals; no animation that slows culling.

Layout: left sidebar (folders, cards, recents, favourites, later
collections); main workspace (contact sheet, thumbnail size, keyboard
navigation, full-screen preview, ratings/tags/labels, search/sort/filter);
right inspector (EXIF, IPTC, caption editor, keywords, credit/copyright,
assignment templates); toolbar and status (ingest, view, filter, selection
counts, task progress). Panels collapsible and resizable; no docking system
in the first release.

Interaction: keep **focus**, **selection**, **picked/tagged**, **rating**,
**colour class** and **active filters** as distinct state. Shortcuts must
not fire while typing in captions or metadata. Customisable shortcut
profiles.

## Accessibility

Apple HIG, WCAG 2.2 AA where applicable, WCAG2ICT for native software,
VoiceOver conventions (Windows conventions later). Contrast, visible focus,
logical tab order, keyboard access, screen-reader names and descriptions,
pointer target size, scalable text, reduced motion, no colour-only status,
high DPI, clear errors, predictable loading. Evaluate with real tests
including VoiceOver; keep documented acceptance criteria and a manual
checklist. Framework support for an accessibility API is not compliance.

## UI technology

Primary candidate egui/eframe; alternative Rust core with a SwiftUI/AppKit
frontend; Slint if capability and licence fit. Compare contact-sheet
performance, virtualisation, GPU textures, native menus, keyboard handling,
accessibility, window integration, high DPI, cross-platform maintenance,
licensing. Build a minimal proof of concept before committing. Keep
platform integration isolated from business logic.

## Architecture

Cargo workspace: `crates/apature-core`, `apature-preview`, `apature-metadata`,
`apature-ingest`, `photo-catalog` (later); `apps/desktop`; `docs/`
(architecture, design, research, benchmarks). Business logic independent of
rendering. Typed commands, explicit state transitions, testable interfaces,
bounded concurrency. No premature abstraction.

## Preview performance

Prefer embedded camera JPEG previews; never fully demosaic for thumbnails
when a suitable preview exists; fall back when missing, corrupt or too
small. Lazy loading, virtualised sheet, priority scheduling, prefetch,
bounded memory, disk cache, cancellation of obsolete requests, background
decoding, explicit invalidation, EXIF orientation, ICC-aware colour as a
verified capability. Start with Nikon NEF and JPEG; design a decoder
abstraction for more cameras. Measure cold open, time to first thumbnail,
time to full visible sheet, navigation latency, p50/p95, memory, cache hit
rate, CPU/GPU, responsiveness during ingest.

## Metadata

Current IPTC Photo Metadata and XMP specs. Fields: caption, headline,
creator, credit, copyright, keywords, date created, location, city, state,
country, usage rights. ExifTool as the initial compatibility backend;
Rust-native handling only where it clearly wins. XMP sidecars for RAW by
default. Preserve unrelated metadata and unknown namespaces. Never silently
overwrite conflicting values. Batch application and templates. Code
replacements with better usability than the reference. Never fabricate
caption details. Edits explicit, reviewable, reversible where feasible.

## Ingest

Cards and external drives, source selection, primary and secondary
destinations, folder and filename templates, collision handling, RAW+JPEG
grouping, verification, background processing, per-file errors, recovery
after interruption. Temporary files and controlled finalisation. Sources
stay intact; never delete sources automatically; not complete until every
required copy is verified. Test interrupted transfers, disconnected cards,
disk full, permissions, duplicates, existing files, Unicode names, long
paths, corrupt images, RAW+JPEG collisions.

## Security and open source

`MIT OR Apache-2.0` for authored code; verify LGPL/GPL/AGPL/CDDL
dependencies; dependency auditing. Defend against untrusted parsing,
malformed metadata, unbounded allocation, unsafe traversal, accidental
overwrites, path traversal via templates, unsafe plugins. No telemetry; no
transmission of photos or metadata. Ship README, CONTRIBUTING, SECURITY,
LICENSE, third-party notices, architecture, setup and testing docs. Prepare
for signed/notarised macOS and Windows packaging, but not before the basic
app is validated.

## Phases

0. Technical research. 1. Technical prototypes (preview benchmark, macOS
integration, contact-sheet rendering, accessibility feasibility).
2. Functional alpha (open folders, NEF/JPEG, navigation, preview, rating and
selection, persistent culling). 3. Metadata. 4. Ingest. 5. Professional
workflows. 6. Public release preparation. 7. Windows.

One phase at a time.

## Code quality

Idiomatic stable Rust; clear ownership; explicit errors; bounded worker
pools; small testable modules; documented interfaces; benchmarks. Avoid
sleeps, blocking the UI thread, unbounded spawning, swallowed errors,
unnecessary unsafe, giant UI components, logic inside rendering,
speculative plugin systems, unnecessary dependencies. rustfmt, clippy, unit
and integration tests. Every significant feature has tests or a documented
reason it cannot.

## Immediate assignment (as given)

Phase 0 plus a narrow Phase 1 feasibility experiment: inspect repo and
environment; research upstream tools and licences; identify reuse vs build;
write a concise ADR; establish enforceable macOS UI/UX and accessibility
requirements; compare egui against a native macOS frontend; investigate
LibRaw vs rawler for NEF embedded-preview extraction; design and run an
isolated RAW preview benchmark; report results, limitations and the next
milestone. Deliver: research findings with sources, recommended
architecture, dependency and licensing matrix, macOS UI/UX requirements,
implementation status, tests executed and results, risks and open
decisions, recommended next task. The quality bar is production
engineering, not code that compiles.
