# Apature mascot: brief for an illustrator

Draft 1, 2026-10-14. Everything in `docs/design/brand`, `mascot/` and
`motion/` is a placeholder made in-house to test placement, sizing and
motion. The commissioned character replaces it.

## 1. The product, in one paragraph

Apature is an open-source, macOS-first tool for newspaper photojournalists
that replaces Photo Mechanic: ingest a card, cull two thousand frames to
the three that matter, caption them, deliver to the desk, on a deadline.
The interface is dark, dense and quiet (charcoal with a lavender accent;
a light appearance exists). Users are professionals working fast; the
character is the one warm thing in the app, and it never gets in the way.

## 2. What we want: a companion in the spirit of Firefox's Kit

We like Kit (Mozilla, 2026, illustrated by Marco Palmieri with JKR)
specifically for: a soft, rendered body rather than flat shapes; **no
mouth**, with expression carried by the eyes, posture and one signature
detail (Kit's tail); a published front / three-quarter / side turnaround;
and a character that "shows up as a small signal that the app is working
for you, then steps back". We are not asking for a fox or for Kit's
palette. We are asking for that level of craft and that restraint.

## 3. The animal

We want Kit's silhouette (pointed face, expressive ears, a bushy tail
that does the acting) without being a fox: the Firefox logo is a fox, Kit
is a fox, MetaMask's mascot is a fox, and Mozilla's trademark policy
forbids designs "confusingly similar" to its marks. So the direction is
**fox-adjacent and unclaimed**:

* **Primary: a pine marten.** A mustelid (so "to ferret out", the oldest
  word for what culling is, still applies), agile, lives in dark woods,
  pointed face, round ears, bushy tail, cream bib. No software brand owns
  it.
* **Alternate: a ringtail.** Strictly nocturnal, enormous eyes, a long
  ringed tail nobody else has. Arizona's state animal; no software brand.
* **Also acceptable if you make the case:** the sable ferret we drafted
  (mask, long body) or an owl (night vision; crowded by Duolingo,
  Hootsuite, Tripadvisor).
* **Not a fox, not a fennec, not a tanuki, not a red panda** (Firefox,
  Firefox, GitLab, Firefox).

Name: to be decided. Candidates that checked clear: Dook, Cutline, Mitts,
Fitch, Dek. Not Kit (it is a baby ferret, and it is Firefox's).

## 4. Personality

Curious, quick, unbothered. Looks at the pictures, not at the user. Never
nags, never jokes in text. Reacts to what the photographer just did:
a card arriving, a frame getting five stars, an export landing, a copy
failing, an empty folder. Reads at 16 pixels and at full size.

## 5. Deliverables

**Round 1, exploration (1 week):** 6–12 pencil or rough silhouettes of
the ferret (and 3–6 of the owl if you want to make the case), one page
each. We will pick one silhouette. Duolingo's team drew thirty-six before
a face; we expect the same spirit.

**Round 2, the character (2 weeks):**
* Turnaround: front, three-quarter, side, back, on one sheet.
* Expression sheet: at least 8 (rest, curious, happy, star-struck,
  surprised, worried, asleep, sleepy), eyes and posture only.
* Proportion notes: eye line below the head's midline, head at least
  half the body height, short limbs, no neck (we follow the baby-schema
  studies, and Kit).
* Palette locked to ours: body from the brand set (charcoal #26262B,
  body highlight #4A4560, lavender #BDB4EE / #E2DDFB / #8F84D6, blush
  #F2A6C9, gold #F2C14E, off-white #ECEBF3) on the darkroom ground #1B1B1F;
  one version checked on the light ground #F6F5FA.

**Round 3, the library (2 weeks):**
* 10 poses, one per app moment (see §6), as **SVG** with the gradients as
  SVG gradients (not raster), plus 1024 px PNG.
* The **mark**: head-only, legible at 16, 32, 64 px; a dark-ground and a
  light-ground version; a one-colour (mono) cut-out.
* The **app icon**: the mark on our charcoal squircle tile.
* Two **set-piece animations**, delivered the way Firefox ships Kit: an
  SVG with CSS keyframes baked in and a `prefers-reduced-motion` guard
  inside the file, plus a 25 fps PNG frame sequence (and a GIF for docs):
  1. *Peek* on first launch: rise from the bottom edge, two eye darts,
     tail sway, settle. About 3.4 s, plays once.
  2. *War dance* when a job finishes clean: three sideways hops, arched
     back, puffed tail, landing squash. About 1.6 s, plays once.
  Both under 5 s, no loops. We will drive the small, interruptible
  moments (eye direction, ear/tail follow-through) procedurally from your
  parts, so please keep ears, tail, eyes and paws on separate layers with
  clean pivots.

## 6. Where it appears (and does not)

| moment | pose | size |
|---|---|---|
| first launch, About window, README | peek / hello / rest | full |
| no folder open (empty sheet) | asleep | full |
| card mounted | holding an SD card | 20 pt toast |
| five stars or the winner class | star eyes | 24 pt, 600 ms |
| ingest or export finished clean | war dance, then content | 48 pt on the progress card |
| a copy failed or a plan refused | worried, ears back | beside the error text |
| long decode | curious, eyes aside | static |
| contact sheet, preview, caption fields, toolbar | **never** | |

Motion rules the app enforces: one-shot, under five seconds, only after a
user action, nothing loops beside the pictures, static under Reduce
Motion, and a setting to turn the character off.

## 7. Files and licence

* Source files (Illustrator, Affinity, Figma or Procreate as you prefer)
  plus exported SVG/PNG. Gradients as gradients, text as outlines.
* The project is open source (MIT or Apache-2.0). We would like the
  character art released under **CC BY 4.0** with credit to you by name,
  with the project holding the name and the app icon as its marks and a
  clause that nobody may register the character as a trademark. This is
  the Go gopher (Renee French) and Krita Kiki (Tyson Tan) arrangement.
  If you prefer a different arrangement, say so in your quote.
* You keep portfolio rights and are credited in the app's About window,
  the README and the brand page.

## 8. References we will send with this brief

* Firefox Kit announcement and turnaround (blog.mozilla.org, "Meet Kit").
* Our research: `docs/design/MASCOT_RESEARCH.md` (cuteness science,
  mascot case studies, the owl problem, the ferret), `docs/design/MOTION_DESIGN.md`
  (fluid one-shot rules, spring table), `docs/research/kit-animation.md`
  (how Firefox ships Kit's animation, read from the source).
* The placeholder character sheets, so you can see the placements we
  tested, not as a style to follow.
* The interactive motion lab (a web page) showing the events and the
  feel we are after.

## 9. Practicalities

* Scope and timeline above are a starting point; quote per round.
* Contact: Caleb McCartney (McCal-Codes on GitHub).
* We will pay for exploration even if the direction changes after round 1.
