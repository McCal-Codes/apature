# How Firefox animates Kit (read from the source, 2026-10-14)

Inventory from the `mozilla-firefox/firefox` main branch via searchfox and
raw files. Everything here is **primary** (the shipped assets and code).

## Inventory: 27 Kit assets, three ways of moving

| asset | where it ships | how it moves |
|---|---|---|
| `newtab/…/kit-circle-animated.svg` | New Tab privacy widget, swapped in for the static `kit-in-circle.svg` "only during an earned moment" (a protection milestone) | CSS keyframes **inside the SVG**: `head_t` rotates the head 2° → −4° → −4° → 2° over a 5 s loop with cubic-bezier easing; two eye groups morph their `d` path for one step (34–38 %) = a blink; `@media (prefers-reduced-motion: reduce)` inside the file sets `animation: none` on head and body. Loaded via `<img>`, so the asset guards itself |
| `aiwindow/…/kit.svg` (139 KB) | AI window "kit-mention" easter egg, shown **once per conversation** for 4 200 ms via `<img>`, hidden entirely under reduced motion | A **4.04 s one-shot** baked into the SVG: 32 `animation:` declarations, all `linear both`; 14 motion paths (`offset-distance` with per-segment cubic-beziers), rotations that overshoot and settle (0° → 13.2° → −1.4° → −2° → 3°), 15 path morphs, and 27 gradients swapped stepwise with `steps(1)` to fake lighting changes. Tool-exported (SVGator-style `a0_t/_mo/_d/_f` names) |
| `privatebrowsing/kit-pbw.svg` + `private-browsing-mask-intro.css` | Private window intro: "Kit puts on the mask" | CSS: Kit holds for 60 % of 1.6 s, rises 6 % with ease-in (anticipation), then drops out of a `clip-path: circle()` to 150 %; the mask logo fades in over 0.6 s after. Played only when the host sets `playing` |
| `shell/assets/kit.gif` | Set-as-default / shell prompt | 84 frames at 40 ms = **3.36 s at 25 fps**, 576×324: Kit rises from the bottom edge, two eye darts, tail sways, settles |
| `asrouter/…/kit-peek.svg`, `kit-peek-bottom.svg` | Menu messages (peek from an edge) | static |
| `asrouter/…/kit-checkmark.svg` + `-flipped` | "Set as default" menu message; the flipped file is the RTL variant; a remote `kit-checkmark-tail` variant exists on the settings CDN | static |
| `aboutwelcome/…/kit-sit-look-up.svg`, `br-kit-with-heart.svg`, `mr-kit-smart-window.svg` | Onboarding screens | static, positioned by CSS variables |
| `sidebar/kit-*`, `profiles/kit-profiles-*`, `kit-signed-out.svg`, `kit-meditating.svg`, `agent-watch-kit.svg` | Sidebar empty states, profiles menu, account button, AI window | static, light/dark pairs |

## What they actually do, in numbers

* **Head tilt loop:** ±2–4° about a pivot at the neck, 5 s period, eased;
  one blink per loop as a path morph; loops, but only while a celebration
  is on screen, and the asset turns itself off under Reduce Motion.
* **One-shot easter egg:** 4.04 s, shown once per conversation, with a
  JS timer slightly longer (4 200 ms) "to leave slack for setTimeout
  drift so the tail of the animation isn't clipped". Overshoot-and-settle
  rotations (13.2° → −1.4° → −2°) are springs drawn by hand as keyframes.
  Gradients change in steps rather than interpolating.
* **Mask intro:** 1.6 s with a 6 % anticipation rise before the exit;
  the follow-up fade waits for it (`animation-delay: var(--intro-kit-duration)`).
* **GIF:** 25 fps, 3.36 s: rise, two eye darts of one or two frames
  each, tail sway, settle. Eye darts are the cheapest expressiveness in
  the whole set.

## Gating rules visible in the code

* "The kit head-tilt loops on its own (CSS animation inside the SVG), so
  it is swapped in only during an earned moment." (Privacy.jsx)
* The celebration is a **one-shot award** acknowledged even when the
  animation is skipped under reduced motion, "so the award doesn't stay
  pending and replay on the next tab".
* Empty and error layouts never celebrate: "celebrating would burn the
  one-shot award on an animation nobody sees."
* The easter egg plays once per conversation and resets with it.
* Every animated asset carries its own reduced-motion guard, because an
  `<img>` cannot be reached by page JS.

## What Apature takes

1. **Bake, don't compute, where a moment is fixed.** Firefox ships
   finished animations (SVG keyframes, a GIF) and plays each once. Our
   springs stay for interruptible moments (focus, redirect); the big
   set-piece moments (first launch rise, war dance) can be baked frames.
2. **Earned moments only, once, with a hard timer** slightly longer than
   the clip. We already have the five-second cap; add the once-per-session
   flag and the "acknowledge even if skipped" rule.
3. **A short loop is allowed only inside a bounded celebration** and must
   stop with it. Our WCAG reading stands: nothing loops beside the
   working surface.
4. **Eye darts** (one or two frames, at a change of attention) and a
   **6 % anticipation rise** are the two cheapest, most Kit-like details;
   both are now in the motion lab's "peek" event.
5. **Overshoot by keyframe or by spring** gives the same shape; Kit's
   13.2° → −1.4° → −2° is a spring with bounce ≈ 0.15.
6. **RTL and light/dark variants are separate files**, not transforms.
7. **The character stays small**: 116×130 in the widget, 180 px in the AI
   window, 91 px in the menu message. Full-size only in onboarding.

Sources: searchfox.org/firefox-main (search "kit", "kit-circle-animated",
"kit-peek", "kit-checkmark"); raw files from github.com/mozilla-firefox/firefox
at `browser/extensions/newtab/content-src/components/Widgets/Privacy/Privacy.jsx`,
`browser/components/aiwindow/ui/components/kit-mention/kit-mention.mjs` and `.css`,
`browser/components/privatebrowsing/content/private-browsing-mask-intro.css`,
`browser/themes/shared/privatebrowsing/aboutPrivateBrowsing.css`,
`browser/components/asrouter/modules/OnboardingMessageProvider.sys.mjs`,
and the SVG/GIF assets listed above.
