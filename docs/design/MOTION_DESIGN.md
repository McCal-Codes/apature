# Mascot motion design (fluid, one-shot, interruptible)

2026-10-14. Synthesis of two research threads (animation craft; ferret
behaviour) plus the interactive prototype `docs/design/motion/ferret-motion-lab.html`
(published as a Claude artifact). Tags: **measured/primary** (a study or a
vendor's own document), **reported** (talks, press), **inferred** (ours).
The implementation section (§5) is filled in from the Rust/egui thread.

## 1. What "fluid" means (primary sources)

**Apple, WWDC 2018 "Designing Fluid Interfaces"** (reported from the
transcript): instant response ("people are really, really sensitive to
latency"), redirectable and interruptible ("the thought and gesture
happen in parallel"), momentum, and feel over duration: "We actually like
to avoid using duration when we're describing elastic behaviors … The
spring is always moving, and it's ready to move somewhere else." Springs
are parameterised by damping ratio and response; "we recommend starting
with 100 % damping."

**Apple, WWDC 2023 "Animate with springs"** (reported): the model is
duration + bounce; springs "are the only type of animation that maintains
continuity both for static cases and cases with an initial velocity", so
a retargeted spring keeps its velocity and interruption is seamless.
Duration is a **perceptual** duration, not the settling time. Bounce
0.15 "doesn't feel very bouncy", 0.3 is noticeable, "be cautious about
values higher than around 0.4"; when unsure use bounce 0. Conversion:
stiffness = (2π/duration)², damping = 4π(1−bounce)/duration, mass 1.

**Material 3 Expressive** (primary, androidx tokens): spatial springs may
overshoot, effects springs (colour, opacity) must not. Expressive default
spatial = damping 0.8, stiffness 380 (≈ duration 0.32 s, bounce 0.2);
fast = 0.6/800 (≈ 0.22 s, bounce 0.4); slow = 0.8/200; effects = damping
1.0. "A damping value of 1 completely removes spring bounce."

**Disney / Williams** (reported): volume preservation under squash and
stretch (no canonical percentage; Pixar's patent exposes a 0–1 "squamount");
anticipation, action, reaction; parts further from the root drag more and
settle later; blink ≈ 8 frames at 24 fps with lids closed about 2 frames
and the lids offset by a frame; blink "every time they change eye
direction"; eye darts in 1–2 frames.

**Practice** (reported): Duolingo runs body-pose and mouth state machines
in parallel in Rive, with idle blinks and nods; the Lily character blends
8 head × 8 body idle clips so no loop repeats, under a megabyte. Nintendo's
Switch UI keeps animations "as short as possible" and couples each to a
sound that confirms an action already taken. macOS's Dock bounces once
for an informational request and forever for a critical one, which is
the anti-pattern everyone remembers.

## 2. The rules Apature keeps

1. Motion starts only from a completed user action or a system event the
   user caused (job done, card mounted, failure, empty folder, first run).
   No idle loop, not even a blink; a blink happens only as part of an
   event, at an eye-direction change.
2. Every event is one-shot and returns to rest on its own. Aim for
   ≤ 1.5 s; sleep is the one exception; 5 s is the hard cap (WCAG 2.2.2
   applies to auto-started motion longer than five seconds beside other
   content).
3. A new event retargets the springs mid-flight. Nothing queues, nothing
   snaps.
4. Bounce never above 0.4; body ≤ 0.3; only star-eyes may touch 0.35.
   Colour, opacity and eye swaps are effects springs: bounce 0.
5. Anticipation is one or two frames on twos (60–100 ms), never a hold.
6. Drag hierarchy: ears/tufts lag 2–3 frames, tail tip 3–6 frames, never
   more than one tail-length behind.
7. Squash ≤ 12 %, volume preserved (stretch the other axis by the inverse).
8. Travel is small relative to the window and never in the periphery; the
   mascot never moves on the contact sheet, preview, caption or metadata
   panes.
9. Reduce Motion: no translation or scale; cross-fade to the end pose in
   ≤ 300 ms; nothing flashes.
10. A "Show the mascot" setting ships with the first pose.

## 3. Event table (Apple duration / bounce; 60 fps)

| event | total | anticipation | main spring | squash | follow-through | Reduce Motion |
|---|---|---|---|---|---|---|
| hello (wave) | ≤ 1.2 s | 80 ms, 4 % dip | 0.45 s / 0.25 | 6 % | tail +3 f, ears +2 f | fade in 200 ms |
| stash (card) | ≤ 0.8 s | 60 ms reach | 0.40 s / 0.15; card 0 bounce | 8 % on the catch | tail +4 f | card cross-fade 200 ms |
| picked (star eyes) | ≤ 1.0 s | blink | eyes 0.35 / 0.35; body 0.40 / 0.20 | 10 % | tail +4 f, ears +2 f | eye swap 150 ms |
| war dance (job finished) | ≤ 1.6 s | 100 ms, 12 % squash | hops 0.40 / 0.30; landing 0.35 / 0.15 | 12 % / 8 % | tail puff + 5 f | static pose, fade 200 ms |
| oops (failure) | ≤ 0.7 s | none | flinch 0.30 / 0, ≤ 4 pt travel; ears back | 4 % | tail +3 f | opacity dip 150 ms |
| sleep (empty folder) | ≤ 2.5 s | none | settle 0.80 / 0.10; lids 10 f ease-out | 3 % | tail curl +6 f | fade 300 ms |

## 4. The ferret's own moves (verified behaviour → pose)

* **Weasel war dance** ("a frenzied series of erratic leaps", arched
  back, frizzed tail, bumps into furniture, after catching a toy or a
  stolen object, with "dooking"): the success animation. Three sideways
  hops with a landing squash and the tail puffed, decaying over 1.4 s.
* **Dead sleep** (limp, "looking like a doll", hard to wake): the empty
  state. Ferrets sleep 14–18 h a day and are crepuscular, which also
  suits a deadline tool.
* **Speed bump** (drops flat mid-play for a few seconds, pops back up):
  a short wait under 2 s, if ever used; owner slang, single source.
* **Stashing** ("intense drive to collect objects and hide them", carried
  in paws and teeth, scooted with the front paws): card mounted and the
  Picks pile.
* **Tunnelling** (tunnels and digging are the enrichment ferrets "most
  enjoyed" in the RVC owner survey): ingest, if the toast ever gets room
  for a tube.
* **Tail semaphore** (relaxed = calm, fast twitch = alert, puffed with
  bounding = joy, puffed with hunching = fear, tucked = nervous): the
  mouthless design works because the tail carries the emotion, which is
  exactly Kit's tail rule.

Names that checked clear or nearly clear in that thread: **Dook** (the
happy sound; owners get the joke), **Cutline**, **Mitts** (the AFA white-paw
pattern), **Fitch** (old English for polecat; big trademark neighbours).
Kit is literally a baby ferret and is Firefox's name; avoid.

## 4b. What Firefox actually ships for Kit

Read from the source (`docs/research/kit-animation.md`): animation
is baked into the SVG assets as CSS keyframes and played once via `<img>`
(a 5 s head-tilt loop gated to "earned moments", a 4.04 s one-shot easter
egg shown once per conversation with a 4 200 ms timer, a 1.6 s mask intro
with a 6 % anticipation rise, a 3.36 s GIF at 25 fps with two eye darts),
each asset carrying its own reduced-motion guard. Apature keeps springs
for interruptible moments and may bake the set pieces (first-launch rise,
war dance) as frames; both follow the same gating: earned moment, once
per session, hard timer, acknowledged even when skipped.

## 5. Implementation in Apature (egui 0.33, glow backend)

**What epaint can and cannot do (primary, docs.rs 0.33):** cubic and
quadratic bezier shapes with `flatten`, `PathShape` with fill and stroke,
but **fill is only supported for convex polygons** and there is **no
gradient fill** of any kind. Gradients exist only as per-vertex colours on
a `Mesh` (egui's own colour picker does this), as a UV colour callback on
strokes, or as a texture sampled through mesh UVs. Vertex colours
interpolate in gamma space. Dithering is on by default in eframe 0.33.
Shapes are re-tessellated on the CPU every frame; `animate_value_with_time`
and `animate_bool_with_time` are plain time interpolations. There is no
spring or keyframe system, so springs are ours: ten lines of semi-implicit
Euler, or `charmed-harmonica` (MIT) / `animato` (MIT or Apache).

**Runtimes, checked 2026-10-14:** `rive-rs` (MIT) last committed July 2025,
pinned to a pre-release Vello, README lists mesh gaps and clip bugs: not
production. `velato` 0.12 (Apache or MIT) renders Lottie on Vello 0.10 but
has colour-stop caveats and no layer effects. Vello's GPU path needs wgpu,
which means leaving the glow backend; `vello_cpu` exists but filters are
limited. `rlottie` and `dotlottie-rs` bring a C++ toolchain. Pre-baked
sprite strips cost 9 MiB per 0.6 s state at 256 px and 36 MiB at 512 px.

**Decision (inferred): procedural parts in epaint, driven by springs, with
the SVG gradients cached as textures.** At first run, rasterise each
part's gradient and the glow once with `resvg` (MIT or Apache; already
behind `egui_extras`' svg feature) at 2× into textures. Each frame, flatten
the spring-driven beziers, build a `Mesh` per part whose UVs map the part's
bounding box onto its gradient texture, and submit it. Edges stay vector
crisp, the gradients are the SVG's own, the cost is a few hundred
triangles on the existing backend, and the JPEG decode pool is untouched.
Concave parts (the mask) are split into convex pieces or triangulated with
`lyon` (MIT or Apache). Fallback under Reduce Motion or on a profiling
spike: a static pre-baked frame cross-faded with `animate_bool_with_time`.

**Follow-through:** a three-joint tail where each joint is a spring whose
target is its parent's angle plus the rest offset, stiffness falling and
lag rising toward the tip (ω ≈ 18, 14, 10 rad/s; ζ ≈ 0.55, 0.45, 0.35),
converted to a cubic with Catmull-Rom tangents. The prototype uses the
same idea with two lagged springs and reads as fluid.

**Reduce Motion on macOS:** `objc2-app-kit` (MIT/Apache/Zlib),
`NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion()`,
polled about once a second (gpui does the same) or observed through
`NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification` on the
workspace notification centre; winit exposes nothing. The app already
honours the setting for its own motion through `App::anim`; the mascot
uses the same gate.

**Pseudo-code (per frame):**

```rust
struct Spring { pos: f32, vel: f32, omega: f32, zeta: f32 }
impl Spring {
    fn step(&mut self, target: f32, dt: f32) -> f32 {
        let a = -2.0 * self.zeta * self.omega * self.vel
              - self.omega * self.omega * (self.pos - target);
        self.vel += a * dt; self.pos += self.vel * dt; self.pos
    }
}
// Apple duration/bounce → omega = 2π/duration, zeta = 1 − bounce.
fn mascot(ui: &mut egui::Ui, m: &mut Mascot) {
    let dt = ui.ctx().input(|i| i.stable_dt).min(0.1);
    if m.event_elapsed() > m.hold() { m.retarget(Pose::Rest); }      // one-shot
    for (param, spring) in m.springs.iter_mut() { spring.step(m.target[param], dt); }
    let tail = m.tail.follow(m.body_angle(), dt);                      // lagged chain
    for part in m.parts(&tail) {                                       // flattened beziers
        let mut mesh = Mesh::with_texture(part.gradient_tex);
        part.triangulate_into(&mut mesh, |p| part.uv(p));
        ui.painter().add(Shape::Mesh(mesh.into()));
    }
    if m.live() && !m.reduce_motion { ui.ctx().request_repaint(); }
}
```

**Where it is drawn:** the empty sheet (full size), the About window
(full size), the Progress card after a run (48 pt), the card-mounted and
export-done toasts (20 pt), the inspector marks row for "picked" (24 pt,
600 ms). Never on the contact sheet, preview, caption or metadata panes.

Sources for this section: docs.rs epaint/egui/egui_extras 0.33 pages
(`CubicBezierShape`, `PathShape`, `PathStroke`, `Mesh`, `Brush`,
`TessellationOptions`, `Context::animate_*`); github.com/linebender/resvg,
vello, velato; github.com/rive-app/rive-rs; docs.rs charmed-harmonica,
animato, objc2-app-kit; ryanjuckett.com "Damped Springs"; Jakobsen,
"Advanced Character Physics"; gpui-base `reduce_motion.rs`.
