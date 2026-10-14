# Motion and polish: what else would make Apature feel finished

Research note, 2026-10-14. Status words: **have** (on main, seen in a
screenshot or exercised by a flag), **proposed** (not built), **assumption**
(not measured). egui 0.33 facts come from its API: `animate_bool_with_time`,
`animate_value_with_time`, `show_animated`, per-frame repaint; there is no
spring or keyframe system, so anything beyond one eased value is ours
(`ui/motion.rs`).

## What is there (have)

**Slice 1 landed 2026-10-14** (branch m7-motion-1): thumbnail arrival fade (seen
mid-fade in a screenshot), selection ring fade and keyboard focus slide
(implemented, not yet watched by eye: the screenshot flag cannot press keys),
ingest rows staggered on start, ✓ colour fade, byte-driven bar, done flash on
the Progress card (seen), and the card-mount watcher: `/Volumes` polled every
2 s; a new volume with a DCIM folder sets the status and, with the Ingest
preference on (default), switches to Ingest with that card as the source
(seen: image attached while the app ran, workspace switched, plan built).

Panels slide (show_animated), tab highlight glides and the sheet cross-fades
on tab switch, the preview grows out of its cell, chips/overlays/status fade,
the ingest bar eases, and every duration goes to 0 under Reduce Motion.

## Principles (from Apple HIG "Motion" + our design docs)

* Motion explains a change of state; it never decorates. One curve
  (ease-out), two durations (120 ms micro, 160–200 ms layout), no bounce.
* Nothing the photographer waits on may animate longer than the work takes:
  a 3-frame ingest is over in 2 s, the bar must not lag behind it.
* Reduce Motion removes movement, not feedback: a fade may stay, a slide
  becomes a cut.
* 60 fps or nothing: an animation that drops frames on a 5-across sheet of
  NEF previews is worse than a cut. Measure with the `frame` number in the
  status bar before keeping any of these.

## Proposed, ranked by how often it would be seen

| # | where | motion | why / cost |
|---|---|---|---|
| 1 ✓ | sheet | **Thumb arrival fade** (120 ms) when a thumbnail replaces its placeholder, instead of popping | seen on every folder open; 1 `animate_bool` per visible cell |
| 2 ✓ | sheet | **Selection ring** fades in/out and the selected cell's label colour eases; marquee rectangle fades out on release | culling is the main loop; cheap |
| 3 ✓ | sheet | **Focus move** after a mark with "advance after mark": the focus ring slides to the next cell (160 ms) rather than jumping, so the eye follows | the one place a slide carries meaning; uses `lerp_rect` already in motion.rs |
| 4 | sheet | **Reflow on resize / zoom slider**: cells move to their new positions over 160 ms | PM and Finder both cut; Bridge animates. Assumption: nicer at 5-across, distracting at 100 cells. Cap: only animate when ≤ 40 visible cells |
| 5 | inspector | **Section collapse** (height ease) and value change flash (muted → text over 300 ms) when a mark changes the shown rating/class | confirms the keystroke did something |
| 6 ½ | preview | **Zoom to 1:1** scales from the pointer (have, M8) (needed anyway for the 1:1 feature); **pan** has inertia under drag release (assumption: 200 ms decay, off under Reduce Motion) | part of the preview work in the roadmap |
| 7 | filmstrip | current-frame indicator slides; strip auto-scrolls with easing when focus leaves the viewport | |
| 8 ✓ | ingest | **Rows appear as the plan builds** (stagger 15 ms, cap 20 rows), the ✓ glyph fades in per file, the bar is driven by bytes not files so big NEFs don't stall it; a short **done flash** on the progress card border | directly visible in the owner's daily use |
| 9 | drag-and-drop | drop target overlay fades in (have) **plus** the sheet dims and a folder glyph scales 0.9→1 on hover-over | |
| 10 | toasts / status | status text cross-fades on change; a transient toast for "6 files verified" slides up 8 px while fading in | |
| 11 | tabs | opening a tab grows its width from 0 over 160 ms, closing shrinks; the + button nudges | Safari-style; needs manual width lerp |
| 12 | settings | page switch cross-fade (have for the sheet, not for Settings) | |
| 13 | menus / popups | egui popups already fade; add 4 px rise for the filter popup | micro |
| 14 | app launch | first sheet fades in from the empty state | once per session, low value |

Avoid: parallax, bouncy springs, animated blur, anything on the thumbnails
themselves (hover zoom) because it fights the eye during culling.

## Beyond animation: the polish list

* **Card mount awareness** (have, polling): watch `/Volumes` (FSEvents via
  `notify`, or a 2 s poll, both fine) so a card appears in the sidebar the
  moment it mounts; optional pref "on card mount: open Ingest". This is the
  `IngestAutoIngestCheck` behaviour from the gap analysis.
* **Eject from the sidebar** after a verified ingest (`diskutil unmount`
  via `std::process::Command`; never after a failed one).
* **Card contents before ingest**: show a thumbnail strip of what is on
  the card inside the Source panel so the photographer sees it is the right
  card. Previews already come from embedded JPEGs so this is cheap.
* **Hover state and pointer feedback** on every clickable (the HIG
  checklist item that is still only partly done).
* **Keyboard focus ring** visible on all controls (VoiceOver pass pending).
* **SF Pro** loading from the system (planned since M2, not done).
* **Empty states** with one sentence and one action (sheet has one; the
  inspector, caption and ingest panels do not).
* **Frame budget HUD** behind a debug flag so animations can be measured
  rather than assumed.

## Suggested first slice

1, 2, 3, 8 and the card-mount watcher: all touch files the owner uses every
shift, none need new dependencies except `notify` (MIT/Apache, already
vetted in THIRD_PARTY for Live Ingest later).
