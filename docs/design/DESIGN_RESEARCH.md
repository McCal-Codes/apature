# Design research

Product name: **Apature** (decision of 2026-10-13; code and binaries still
say photo-workflow until the consolidation).

## What was inspected, inferred, and not verified

| source | how | what it gave |
|---|---|---|
| `~/dev/mccal-codes.github.io/src/styles/tokens.css` (mccal.dev) | read in full | the token system: cool near-black ground `#0B0B10`, panel `#15151C`, raised `#1D1C27`, text `#ECEBF3` / `#B7B4CB` / `#8F8CA6`, borders `#25242F` / `#3B3A4B`, one accent family (lavender `#CFC8F5`, dim `#A79FD6`, wash 10 %), glow `#2C2444`; type: Jakarta sans + Plex Mono; three-tier hierarchy (display sans, compact body, uppercase mono metadata); measured contrasts; "status is never color alone"; motion 180 ms `cubic-bezier(0.22,1,0.36,1)`; radii 16/28 for tiles |
| `src/pages/HomePage.module.css` (mccal.dev) | read first 120 lines | tile grid with one tall tile, hero with radial lavender glow, pill chips with 6 % white fill and 16 % borders |
| `~/dev/McCals-Website/sites/mcc-cal-vite/src/styles/homepage.module.css` + font usage | read 100 lines, grepped fonts | warm near-black `#060608`/`#08080B` with a faint warm radial (`rgba(195,171,147,.08)`), warm off-white text `#F8F4EE`, eyebrows in 0.72rem 700 uppercase 0.14em tracking, **Libre Baskerville** (also Fraunces) for editorial titles, Inter for body, rules at 8–18 % white |
| Folio (`~/dev/duo-fold-launcher`): README, `docs/architecture.md`, ADR list | read | iOS-style settings built from menus/segmented controls/switches; themes as shareable JSON files; a Tweak Library where only installed tweaks show in Settings; a Roadmap inside Settings; layouts exported as portable descriptions; "declarative first" ADR |
| Apature (`~/Documents/Apature`) | agent read of code, docs, commits (see `docs/research/apature-review.md`) | near-black chrome with a deliberately desaturated accent; radii 4/6/10; letterboxed never cropped; text stars; `role="grid"` with roving tabindex; "mixed" spelled out; named contact sheets as working sets; crop guide; pinch zoom to 16:1 |
| Current Rust GUI (`apps/desktop/src/app.rs`) | code audit + running it; **no screenshot possible** | see `MACOS_UI_REQUIREMENTS.md`: fixed panels, hard-coded greys, painter-drawn cells, no menus |
| Apple HIG: Designing for macOS, Sidebars, Split views | read via browser 2026-10-13 | quoted in `MACOS_UI_REQUIREMENTS.md`; notable: sidebar customisable and ≤ 2 levels, nothing critical at its bottom, auto-collapse on narrow windows, thin 1 pt dividers, multiple ways to reveal hidden panes, "support personalization… toolbars… views… colors and fonts" |
| mccal.dev and mcc-cal.com live sites | **not fetched** (local sources used instead) | production typography is inferred from source |
| Apple Photos, Finder, Capture One, Photo Mechanic, Narrative | **from working knowledge, not re-inspected this session** (Photo Mechanic and Capture One are installed; a screenshot pass is a cheap next step) | patterns below are marked inferred |

## Reference patterns (inferred unless noted)

| reference | take | leave | adapt as |
|---|---|---|---|
| Apple Photos | one quiet toolbar; thumbnails edge to edge with no chrome between; zoom slider; split-view sidebar with user-ordered albums; keyboard ⌘1–4 view switching | the hidden metadata (one Info panel for everything); no colour labels; cropped square thumbs | Contact Sheet default: full-bleed grid, inspector optional |
| Finder | source-list sidebar with Favorites you reorder; ⌘1–4 view modes; ⌘I; label colours as dots next to names (**not colour-only: name + dot**); column view for hierarchy | icon view's slow gallery | Sidebar = Favorites + Recents + Volumes; colour class = dot + name/number |
| Capture One | user-configurable tool tabs; collapsible tool panels with a persistent order; viewer with filmstrip; metadata presets applied to many | heavy chrome, tiny 11 px controls everywhere, dark-only | Workspace layouts saved per task; inspector built from collapsible sections |
| Photo Mechanic | the keyboard: single-key colour class, ⌘-number rating, T tag; contact sheet with large, legible filenames; stationery pad; code replacements; ingest dialog with two destinations and verification | 2000s widget density; modal dialogs for everything; no theming | keep every shortcut; replace dialogs with workspaces |
| Narrative Select | image first: huge preview, thin filmstrip, overlays that fade; focus/closed-eye assessments (not planned) | AI culling (ADR-0005 in Apature: none) | Preview workspace: chrome fades, metadata on hover or ⌘I |
| mccal.dev | tokens, mono uppercase metadata tier, one accent for focus only | 28 px tile radii, hero glow | chrome tokens; accent never near a photograph |
| mcc-cal.com | Libre Baskerville for editorial moments; warm off-white; eyebrow labels | warm tint on photo surfaces | serif only for assignment headline and empty states |
| Folio | themes as JSON; only the tools you installed appear; settings as a roadmap | mobile layout | shortcut profiles, workspace layouts and tool visibility as exportable JSON |

## Three directions

Full mockups: `docs/design/mockups-design-directions.html` (open in a
browser) and the published copy at
https://claude.ai/artifact/ECuspfScf1TiU2Ezp8yDxY. Each shows the Contact Sheet workspace with the Butler Eagle golf
assignment as data; the recommended one also shows Caption.

### 1. Native Editorial
Closest to macOS. System appearance (light shown), SF Pro, source-list
sidebar, unified toolbar, inspector with grouped fields. Identity appears
only in the lavender selection/focus and the serif assignment headline.
*Trade-offs:* fastest path to HIG compliance and VoiceOver; the light
chrome competes with photographs; least distinctive.

### 2. Professional Darkroom
Photo-centred. Neutral dark grey everywhere near images (no blue cast),
compact controls, mono metadata, numbered colour-class chips, a persistent
tool strip. Identity in the type hierarchy and the desaturated accent.
*Trade-offs:* best photographic evaluation and density for deadline work;
needs a deliberate light theme for daylight laptops and accessibility
review of 11 px text.

### 3. Minimal Workspace
Progressive disclosure. The grid is the window; sidebar and inspector are
overlays summoned by key or hover-edge; a single floating toolbar; tools
appear only when enabled (Folio's library idea). mccal.dev tokens on the
chrome. *Trade-offs:* most distinctive and calm; discoverability and
VoiceOver navigation of transient overlays are the hard problems; risks
hiding file-operation status, which the brief forbids.

## Recommendation

**Professional Darkroom as the base, with Native Editorial's structure.**
Keep the HIG skeleton of direction 1 (real menu bar, source-list sidebar,
thin-divider split view, inspector sections, system shortcuts) and
direction 2's surfaces and density around photographs. Borrow direction
3's tool visibility and saved layouts as a setting, not the default.
Reasoning: the brief's non-negotiables are keyboard speed, photographic
evaluation and accessibility; 1 wins accessibility and 2 wins evaluation,
and their combination has no conflict, while 3 trades discoverability for
calm. Implementation cost is also lowest for 1+2 in egui (panels, tokens,
AccessKit names) and highest for 3 (overlay focus management).
