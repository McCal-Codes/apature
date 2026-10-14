# Mascot research for Apature

Second, extended pass, 2026-10-14. Six research threads (cuteness science,
software mascot case studies, animal vision facts, name availability,
in-app motion rules, newsroom culture) ran in parallel; this document is
the synthesis. Claims are tagged **measured** (a study), **reported**
(brand teams, press, primary pages) or **inferred** (our reading).
Anything a thread could not confirm is marked UNVERIFIED. Sources are at
the end of each section.

## 0. The short version

1. **Cuteness is mostly geometry, and it does not wear off.** Big head,
   eyes set low and large (about 40 % of head height is the ceiling that
   Mickey Mouse arrived at over fifty years), round body, stubby limbs,
   very few features. The response is pre-attentive and survived repeat
   exposure in the one product study that tested it. (§1)
2. **Software mascots that last live on the splash, the empty state and
   the small win, and never on the working surface.** Clippy and the
   "evil Duo" meme were both interruptions of the task; Kit and Kiki step
   aside. Photo tools have no mascots at all, which is an open lane and a
   warning. (§2, §4)
3. **A looping idle animation next to other content is a WCAG Level A
   problem**, not a nicety. Mascot motion must be one-shot, under five
   seconds, on user events only, and off entirely under Reduce Motion.
   Ship a "no mascot" switch. (§4)
4. **Licence the character CC BY 4.0 (or CC0) with a no-trademark
   clause, and keep the name and app icon as the project's marks.**
   This is the Go gopher / Android / Kiki model. (§3)
5. **"Lumen" is taken three times over inside photography** (Lumen
   Camera, Lumen Forge, Rollei Lumen). Rename the character. (§5)
6. **The owl is defensible on the science and crowded on the shelf**
   (Duolingo, Hootsuite, Tripadvisor). Two animals beat it on story and
   ownership: the **jumping spider**, which has the most camera-like eye
   in nature, and the **homing pigeon**, which newspapers really used to
   fly film from the ballpark to the darkroom. The kingfisher collides
   with a major Swift image library; the firefly with Adobe Firefly; the
   eagle with an existing photo organiser and with the owner's own paper. (§6)
7. **Two verified facts are brand gold:** lavender oil was the solvent in
   Niépce's first photographs (the accent colour is photography's
   origin), and the firefly is Pennsylvania's state insect. (§7)

## 1. Why things read as cute (science)

**Lorenz's releasers, as Gould quotes them (reported):** "a relatively
large head, predominance of the brain capsule, large and low-lying eyes,
bulging cheek region, short and thick extremities, a springy elastic
consistency, and clumsy movements."

**Gould's Mickey Mouse measurements (measured, from the essay):** over
three design stages, eye height went from 27 to 42 % of head length,
head length from 42.7 to 48.1 % of body length, and nose-to-front-ear
from 71.7 to 95.6 % of nose-to-rear-ear. The rival Mortimer had a head
29 % of body length and a snout 80 % of head length. Because Mickey's
head is a fixed circle, juvenility came from moving the ears back, not
from reshaping the head.

**Glocker et al. 2009 (measured):** manipulating infant photos towards
the schema raised cuteness ratings and caretaking motivation. **Frontiers
2022 (measured):** low placement of features and a narrow jaw predicted
cuteness; forehead height did not.

**Miesler, Leder & Herrmann 2011, car fronts (measured):** headlights
+20 %, grille −20 %, intake narrower and taller; babyfaced cars rated
cuter (d = 0.54), with smile-muscle activation, **no habituation across
exposures**, and no change in attractiveness. Their advice: exaggerate
the key stimulus, "such as very large headlights."

**Eyes (measured, mixed):** enlarging eyes 15 % helped only by avoiding
small eyes; beyond that looked unnatural in realistic faces (Yao 2022).
A 2024 *Cognition* study found larger irises with smaller pupils most
attractive, and a 2025 primate-portrait study found bright irises cuter
and friendlier only with dilated pupils: a bright ring with a medium-large
dark centre is the safe reading. One consistent highlight at the same
clock position on both eyes reads as alive and looking at you (Kitaoka,
Anstis); mismatched highlights read cross-eyed.

**Craft rules that match the science (reported):** Bruna's Miffy is dots
for eyes and a cross for a mouth, front-facing, hundreds of sketches per
expression; Kirby is "a simple circular design, so anyone could draw
him"; Pikachu was slimmed not for cuteness but because the anime needed
"human-like gestures"; Kumamon is a black body, red cheeks and a
"constant look of surprise"; Hello Kitty's missing mouth lets people
"project their feelings onto the character" (Sanrio spokespeople) and was
blamed by the *NYT* for her weakness as an animated character. Pixar's
*Up* is the documented shape-language case: circles roll and move fast,
squares are stable.

**Jumping spiders (reported):** perceived as less frightening than other
spiders because of "neotenic traits such as two larger eyes and very
short legs" (Frontiers in Arachnid Science 2025, citing Bixler 2015): two
large forward eyes plus short limbs override an unappealing body plan.

**Does cute UI help? (measured, thin):** the oft-cited "USC study, 23 %
more engagement" does not exist in any findable paper. The real figure is
System1/IPA Databank advertising data (mascot campaigns 37 % more likely
to grow share), observational, no controls. A 2022 HCII poster (N = 166)
found favourable responses to cute mobile aesthetics. Nothing tests cute
agents and error forgiveness directly.

**Implications for Apature:** eyes at or below the head's midline, ring
(iris) bright lavender with a dark centre and one shared highlight; head
at least half the height of the full-body figure; no mouth in the mark;
one primary shape (circle); design the error and empty-state faces first,
because anthropomorphism peaks when a system is unpredictable (Epley,
Waytz & Cacioppo 2007).

Sources: Gould, "Mickey Mouse Meets Konrad Lorenz" (Natural History 1979,
PDF at cognitionandculture.net); Glocker 2009 (PMC3260535); Frontiers in
Psychology 2022 (10.3389/fpsyg.2022.979341); Miesler, Leder & Herrmann,
International Journal of Design 2011 (ijdesign.org/…/1023/363); Yao 2022
(Frontiers 10.3389/fpsyg.2021.674456); PsyPost on pupil/iris studies;
Epley 2007 (PubMed 17907867); Wikipedia "Twelve basic principles of
animation", "Hello Kitty"; CGW "The Shape of Animation" (2009); PBS on
Dick Bruna; shmuplations Kirby 1993 interview; Monocle and nippon.com on
Kumamon; Frontiers in Arachnid Science 2025 (frchs.2025.1717365); Adweek
on System1/IPA.

## 2. How software mascots are built and where they live

| mascot | origin | licence / mark | in the product? | lesson |
|---|---|---|---|---|
| Duolingo Duo | 2013 reshaping, 2019 simplification | corporate mark | logo, streak persona, in-lesson cast | silhouette first ("weak overall shape can't be rescued by details"); every illustration built from rounded rectangle, circle, rounded triangle, "pointy shapes are off-brand"; Rive state machines with idle blinks and 20+ visemes per character; the "evil owl" meme was rescued by marketing, not product |
| Firefox Kit (2026) | JKR + Marco Palmieri, pencil first, proportions from the logo | not published | onboarding, feature discovery, small wins, "then steps back"; new-tab wallpaper | no mouth; "companion, not a commentator"; "not interactive for now"; name collides with kit.com (ConvertKit) |
| Krita Kiki | Tyson Tan 2013 | GPL + CC BY-SA, "no definitive design", fan art invited | splash screen every release since 2.8, website header | a mascot can be the project's face from the splash alone |
| GIMP Wilber | Kuosmanen 1997, Aryeom 2024 | CC BY-SA 4.0; "Wilber Construction Kit" .xcf in the source tree | app icon and splash | mascot-as-icon works when it survives 16 px; ship a construction kit |
| Blender Suzanne | 2002 Easter egg test mesh | logo is a mark; Suzanne UNVERIFIED | inside the product as a useful asset | the most durable in-product mascot is useful, not decorative |
| Rust Ferris | Karen Rustad Tölva | CC0, "unofficial", outside the trademark policy; six poses | docs and community only | CC0 outside the policy = maximum adoption, zero enforcement |
| Go gopher | Renee French | CC BY 4.0; "a character, not the logo" | docs, community | separate character from logo in writing |
| Android Bugdroid | Irina Blok | CC BY 3.0 + "you may not file trademark applications for … the Android robot" | OS Easter eggs, marketing | CC BY plus a no-trademark clause is the cleanest open model |
| Linux Tux | Larry Ewing 1996 | "acknowledge me … if someone asks" | kernel boot logo | thirty years on, lawyers still wince; use a real licence |
| PostgreSQL Slonik, Docker Moby, Mastodon elephant | community / corporate | registered marks (Mastodon's character licence UNVERIFIED) | marketing | even community projects register the character once it has value; decide the owner early |
| Reddit Snoo | Ohanian 2005 | part of the logo; blank head, orange-red eyes, no fingers | subreddit-customised Snoos are core identity | user remixing scales under a few hard rules |
| Discord Clyde / Wumpus | | brand assets needing permission | Wumpus in empty states and onboarding; Clyde as system bot | two characters with distinct jobs |
| Twitter Larry | iStock clip art, redrawn 2009, retired 2023 | contested | logo only | never build on stock art |
| Mailchimp Freddie | 2018 COLLINS silhouette | "proprietary" | brand only | a single-colour silhouette sits beside a wordmark; the 2020 illustration layer was removed because every execution needed bespoke art direction |
| Clippy | 1997 | | the working surface, uninvited | focus groups were negative and women found the characters "leering"; the shipped trigger model was cruder than the Bayesian one designed; off by default within four years |
| Headspace, Notion, Linear, Raycast, Capture One, Photo Mechanic, Lightroom, darktable, RawTherapee, digiKam | | | **none** | "deliberately none" is UNVERIFIED everywhere; photo tools are instrument brands |

**Placement pattern (inferred from the table):** in-product mascots
cluster at first run, empty states, success confirmations and
notifications. They are almost never in the error path or on the hot
working surface. Both famous failures were interruptions of the task.

**Production norms (reported):** Duolingo's reshaping took months of daily
iteration; silhouette, then quarter-turn, then movement; colour and
species fixed up front. Ferris ships six poses and that is a complete set
for a community project. Mailchimp shipped a "party pooper mode" to turn
humour off after prolonged-exposure complaints.

Sources: blog.duolingo.com "reshaping-duo", "shape-language-duolingos-art-style",
"world-character-visemes"; blog.mozilla.org "meet-kit"; discourse.mozilla.org
Kit thread; krita.org/en/about/mascot; docs.gimp.org introduction;
lists.blender.org 2009 Suzanne thread; rustacean.net; rustfoundation.org
trademark policy; go.dev/blog/gopher; developer.android.com brand
guidelines; kernel.org FAQ; postgresql.org trademark policy;
joinmastodon.org/trademark; discord.com/branding; Wikipedia "Twitter logo",
"Office Assistant"; Neowin on Clippy focus groups; mailchimp.com brand
assets; dia.tv Mailchimp; axbom.com on party-pooper mode; advertisingweek
on Headspace; notion.com brand campaign post.

## 3. Licensing model for an open-source mascot

The pattern that spreads: character art under **CC BY 4.0** (Go, Android,
Kiki/Wilber with SA) or **CC0** (Ferris), with an explicit **no-trademark
clause** ("you may not file trademark applications for the character or
derivatives"), while the **product name and app icon** stay the project's
marks under a short policy. Keep the artist's copyright assignment or
licence separate from the trademark question, and decide the owner (the
author or a foundation) before the character has value: PostgreSQL and
Mastodon both ended up registering theirs.

Checklist before naming anything: USPTO Trademark Search (TESS retired
Nov 2023) and EUIPO TMview in classes 9 and 42 (25 and 16 if merch);
phonetic variants; Mac App Store, GitHub org and repo, crates.io, npm;
domains and handles.

## 4. Motion and placement inside a professional tool

**Standards (reported):** Apple HIG Motion: "Don't add motion for the sake
of adding motion … may make them feel disconnected or physically
uncomfortable"; Apple's Reduce Motion criteria name "scaling, spinning, or
peripheral motion" as triggers and say decorative animation should
"stop entirely" under the setting. **WCAG 2.2.2 Pause, Stop, Hide (Level
A):** moving or blinking content that starts automatically, lasts more
than five seconds and is presented in parallel with other content needs a
pause mechanism unless essential; the intent text names attention-deficit
users. WebKit's motion post names peripheral horizontal movement as a
vestibular trigger; the Game Accessibility Guidelines say background
animation "can distract greatly … particularly for ADHD."

**Practice (reported):** Duolingo blends eight head and eight body idle
clips at random to avoid repetitive loops, under a megabyte per character,
driven by lesson events. Kit's tail carries emotion "even in still
moments." Clippy's Bayesian timing model was replaced before shipping
because it "was deemed too cautious." Human blink rate is 17.6 per minute
with a 4.3 s mean interval (J Neurosci 2011); animators use fast close,
slower open, randomised interval. Lottie ran at ~17 fps versus Rive ~60 fps
on a mid-range phone in one benchmark; `rive-rs` exists but is Vello-only
with no production-readiness statement. For egui, a short sprite strip
with timer-driven frames is the only zero-dependency path.

**What photojournalists say (reported):** "Granted it may not have the
prettiest interface, but there is a lot of power under the hood"; Photo
Mechanic "because it is wicked fast"; "14 minutes to download my photos …
and upload to Team USA"; "feels a bit like working in Windows 2000."
Nobody asked for personality; nobody minds it as long as it never costs a
keystroke.

**Rules for Apature (inferred):**

1. The mascot never animates in the contact sheet, preview, caption or
   metadata panes.
2. Any animation is one-shot, finishes within 5 s and never loops.
3. It moves only on user-caused events: job done, ingest finished, error,
   empty folder, first run.
4. No unprompted appearances and no idle fidgeting while the user works.
5. Under Reduce Motion: a static frame, or an opacity or colour change only.
6. Blink, if used, only in the empty state and About: 100–150 ms,
   randomised 3–6 s, stopping after 5 s.
7. Squash and stretch at most 5 %; small travel; never in the periphery.
8. A "no mascot" setting ships with it.
9. Pre-baked frames; no animation runtime until measured against decode
   load with the frame-time HUD.
10. Warmth lives in copy and empty states, not in the cull loop.

Sources: developer.apple.com HIG Motion and App Store Connect Reduced
Motion criteria; w3.org WCAG 2.2.2 Understanding; webkit.org motion post;
alistapart.com "Designing Safer Web Animation"; gameaccessibilityguidelines.com;
rive.app Duolingo Lily post; erichorvitz.com Lumière; xenon.stanford.edu
Clippy thesis; PMC3156585 (blink rate); callstack.com Lottie vs Rive;
github.com/rive-app/rive-rs; fstoppers and blog.jeffcable.com quotes;
shutterbug.com Photo Mechanic review.

## 5. Names

**"Lumen" (reported):** Lumen Technologies, the Lumen metabolism app, the
Laravel Lumen framework, Unreal Engine 5 Lumen, Lumen Database, Lumen5,
an Apple Arcade game, a macOS brightness utility, and three photography
apps: **Lumen Camera** (manual/RAW iPhone camera, 2026), **Lumen Forge**
(exposure calculator) and **Rollei Lumen** (studio lights). No owl named
Lumen exists in software, but in-category crowding makes it a weak choice.

**"Apature" (reported):** no app, npm or crate called Apature; a dormant
GitHub user and an unrelated `apatureai` org (TypeScript UI-review
tooling, June 2026); apature.com (1996, parked), .io/.net (2023, no
site); .app and .dev likely free (UNVERIFIED). Apple's Aperture
(discontinued 2015) and Aputure (cinema lighting) are near-homophones;
search engines autocorrect to Aperture. Reported, not advice.

**Less crowded character names (reported, checked by URL):** Dek, Cutline,
Lede are the clearest of the newsroom set; Loupe (GNOME image viewer),
Sprocket (HP), Pica (font manager), Chimp (Mailchimp), Fixer, Stringer
(Stringr), Byline are taken or near-taken. Unchecked: Hed, Grain, Tilt,
Wire, Journal.

**Animal name collisions (reported):** Kingfisher = onevcat/Kingfisher,
a ~24k-star Swift image-caching library in Apature's own ecosystem, plus
Kingfisher plc and the beer; Firefly = Adobe Firefly (generative AI, the
opposite of culling), Firefly III, Firefly Aerospace; Tarsier = Reworkd's
`tarsier` and ByteDance's Tarsier video models; Mantis shrimp and jumping
spider (`salticidae` npm package only) are clear fields; a homing pigeon
has no software mascot found (UNVERIFIED beyond this pass).

## 6. The animals, re-scored with verified facts

Scale 1–5. *Story* = a verified fact about vision, light, timing or
selection. *Free* = no major software or photo brand owns the animal or
its obvious name. *Cute* = baby-schema features present. *Icon* = reads at
16 px. *Range* = poses and expressions without props.

| animal | story | free | cute | icon | range | the verified fact |
|---|---|---|---|---|---|---|
| **Homing pigeon** | 5 | 5 | 3 | 4 | 4 | NY Journal-American flew film from Yankee Stadium from 1935: twelve pigeons, canister taped to the leg, nine-minute flight, DiMaggio in print 38 minutes after he scored (Oct 1938); San Antonio Express did the same; Reuters began with pigeons in 1850. Newsroom-native, nobody owns it |
| **Jumping spider** | 5 | 5 | 4 | 3 | 4 | principal eyes are fixed Galilean telescopes with a movable four-tier retina of ~1,500 receptors; acuity 0.04°, best of any land invertebrate; judges depth by image defocus between green layers (Nagata 2012, *Science*): literally focus-by-blur. "Lucas the Spider" proves the cuteness; a fraction of users are arachnophobic |
| Owl | 4 | 1 | 5 | 5 | 4 | tubular eyes locked in a bony ring, cannot move; 270° head rotation with four vascular adaptations (Johns Hopkins 2013); rod-dominated retina; asymmetric ears in nocturnal species. Crowded: Duolingo, Hootsuite, Tripadvisor |
| Tarsier | 4 | 4 | 5 | 3 | 3 | each eye weighs more than its brain; eyes fixed, head rotates ~180°; obscure, looks startled; name taken in ML |
| Kingfisher | 4 | 2 | 4 | 4 | 4 | two foveae per eye; red oil droplets in aquatic species' cones; "switches modes mid-dive" and dive-success rates are UNVERIFIED pop science; the Shinkansen nose story is secondary. Name clash with the Swift library |
| Mantis shrimp | 4 | 5 | 3 | 3 | 4 | twelve colour receptor types plus polarisation = sixteen classes; sees circularly polarised light; but discriminates colour worse than humans (Thoen 2014): a liability for a culling tool if anyone looks it up; the Oatmeal owns the joke |
| Firefly | 3 | 1 | 3 | 3 | 3 | light yield is 41 %, not "nearly 100 %" (Ando 2008); *Photinus carolinus* flashes in synchrony; Pennsylvania's state insect (1974). Adobe Firefly makes it near-fatal for a photo tool |
| Eagle | 4 | 1 | 1 | 3 | 2 | 132–143 cycles/degree acuity vs ~60 human (Reymond 1985). eagle.cool is an existing image organiser for photographers, and the Butler Eagle owns the mark locally: conflict, not clever |
| Chameleon | 3 | 2 | 3 | 3 | 3 | only vertebrate with a negative lens (Ott & Schaeffel 1995); eyes less independent than assumed (2015). SUSE, Kameleoon |
| Cuttlefish, hummingbird, cat, fox, falcon, frog, barreleye | | | | | | strong facts (W pupil; non-spectral colours; tapetum; magnetic pounce alignment 73 % vs 18 %; 242 mph stoop; single photons; rotating tubular eyes) but each fails on cuteness, crowding or relevance to selection |

**Reading.** The owl still wins on cuteness and legibility and still loses
on ownership. The pigeon is the only candidate whose story is *the
photojournalist's own history*, verifiable, charming and unowned. The
jumping spider has the most camera-like eye in nature and a cuteness
precedent, with one real audience risk.

## 7. Newsroom culture to draw on

**Verified:** "shutterbug" dates from 1938 (amateur-coded, dated among
pros); "chimping" was written up by Robert Deutsch in the September 1999
SportsShooter newsletter, named for the "ooh-ooh-ooh"; "the decisive
moment" is a Cardinal de Retz line surfaced by Tériade for the 1952 book;
"f/8 and be there" is folklore (earliest print 1973); -30- is a live
ritual sign-off; hed, dek, lede, TK, graf are deliberate misspellings so
they stand out in copy; every Pulitzer photo 1942–54 was shot on a Speed
Graphic; the press card in the hatband is a movie cliché. Nikon has an
official mascot in Japan (Nikon-chan, plush keychain Aug 2026). The NPPA
logo is a lens, no eagle. Safelights are red or amber because paper is
blind to red. **Lavender oil** dissolved Niépce's bitumen and washed the
unexposed areas of the first photographs (1826–27), and the 1832
physautotype used lavender-oil residue as the light-sensitive agent
itself. **The firefly** (*Photuris pensylvanica*) is Pennsylvania's state
insect by the Act of 10 April 1974.

**Ideas only a newspaper photographer would clock (inferred):** a pigeon
with a film canister on its leg, gone when the job is "filed"; -30- on the
done-culling toast and TK on the empty state; "Filed." as the export
confirmation; a changing bag, a grease pencil and a lanyard press pass as
the only props; an anti-chimping pose during ingest; tooltips in desk
jargon, "spray-and-pray detected: 42 near-duplicates."

Sources: etymonline "shutterbug"; Wikipedia "Chimping", "Weegee", "Butler
Eagle"; petapixel on the decisive moment and the pigeon photo editor;
baseballhall.org "Photos for the birds"; pressgazette.co.uk on Reuter's
pigeons; ajrarchive.org on -30-; cjr.org "leading questions"; bhphotovideo
Speed Graphic; Kodak K-4 darkroom illumination guide;
bicentennial-of-photography.org physautotype; legis.state.pa.us Act 59 of
1974; petapixel on Nikon-chan; fstoppers and jeffcable quotes.

## 8. Decision options

**Option A, keep the owl.** Rename it (Dek, Cutline or Lede are clear;
Hed and Wire unchecked), move the eyes to the lower half of the head, make
the lens rings the dominant feature with one shared highlight, no mouth in
the mark, charcoal body and lavender rings on a dark tile. Accept the
Duolingo adjacency and never animate it on the working surface.

**Option B, the homing pigeon.** The story is the owner's own trade, the
film canister on the leg is a prop that explains itself, "filed" states
write themselves, and nobody in software owns it. Cuteness needs work:
big round head, eye set low and large, stubby body, no neck. Name
candidates: Wire, Journal, Souths (220 South Street, UNVERIFIED
availability).

**Option C, the jumping spider.** The sharpest "camera eye" story in
nature and a proven cute precedent, but some users will not love a spider
on first launch, so it would need the mascot-off switch to be prominent.
Best kept as the easter egg or the "focus" character if B wins.

**Whatever wins:** fix species and colour; silhouette first, tested as a
black fill at 16 and 128 px; six to ten poses; CC BY 4.0 plus a
no-trademark clause; the ten motion rules in §4; the empty state, About
and finished-job toast as its only homes; a setting to turn it off.

Sketches of B and C next to the owl: `mascot/animals-3/`.

## 9. "I really like Kit": what to take from Firefox's mascot

The owner's steer after reading the first pass. What Kit actually does,
from Mozilla's announcement and the published turnaround (reported):

* **Rendered, not flat.** Kit is a soft gradient body (the Firefox logo's
  orange to yellow to magenta) with gentle shading, on a deep purple
  ground; it looks drawn, not assembled from shapes. The team drew it in
  pencil first and derived its proportions from the logo.
* **No mouth.** Eyes are large and dark with a coloured catchlight; the
  catchlight carries the look. Ears and the tail do the acting.
* **One signature detail.** The tail "carries motion and emotion even in
  still moments" and wraps into a heart on the onboarding card.
* **Few, strict appearances.** Getting started, discovering a feature, a
  small win, "then steps back so you can keep moving"; "a companion, not a
  commentator"; no punchlines; optional as a wallpaper.
* **A turnaround sheet** of front, three-quarter and side views was
  published, so the character is drawable from any angle.

Applied to Apature (inferred), with sketches in `mascot/kit-style/`:
a soft charcoal-to-lavender gradient body on the darkroom ground (our
palette is already Kit's purple world); big dark eyes with a lavender
catchlight instead of lens rings; no mouth; the **ear tufts** as the owl's
signature detail (up at rest, perked for a small win, back when content)
with the wings as the second; for the pigeon, the film canister and the
tail fan. Appearances follow §4. Note that this is the opposite rendering
language from the flat/voxel/pixel treatments in `lumen-styles/`; the
pixel sprite stays for 16 px, everything else goes Kit's way.

## 10. The ferret (owner's suggestion, 2026-10-14)

**Facts (reported, dictionary sources):** "ferret" comes from Vulgar Latin
*furittus*, "little thief", from *fūr*, thief (1350–1400); the verb first
meant to hunt with ferrets (early 1400s) and the figurative **"ferret
out"**, to find something by persistent searching, dates from the 1570s.
Pet ferrets keep the wild instinct to **stash** small objects (keys,
phones, jewellery) in dark, quiet places, well fed or not.

**Why it fits better than it first sounds (inferred):** culling *is*
ferreting out: digging through two thousand frames for the three that
matter. Stashing is ingest in one gesture. A sable ferret's colouring is
already the Apature palette: cream body, charcoal mask and points. The
mask reads as a viewfinder band without being one, and at 16 px a head
with a mask is a one-colour silhouette. The long body and tail give it
Kit's "signature detail" for free.

**Conflicts (reported):** FerretDB's logo is a stylised ferret and its
team cites "ferret out" for the same reason; it is a database, a
different category, but it is open source and well known. Stoat.chat has
a stoat mascot named Toast (trademarked), and the 2026 Winter Olympics
mascots are stoats, so the stoat is crowded this year; no pine marten or
ermine mascot was found. "Scoop" as a name collides with the Scoop
package manager for Windows; "Stash" with git stash; "Pip" with Python's
pip (UNVERIFIED as a mascot-name problem, but it is the first thing a
developer will say).

**Scores (same scale as §6):** story 5 (verified idiom + behaviour),
free 3 (FerretDB), cute 5 (round head, low eyes, mask, short legs), icon 4
(head-only mark), range 5 (long body, tail, paws, the donut sleep).
Sketches: `mascot/ferret/`.

## 11. Ferret behaviour, verified (2026-10-14)

Weasel war dance (erratic leaps, arched back, frizzed tail, bumps into
furniture, after a catch; with "dooking"); dead sleep (limp "like a doll",
hard to wake; ferrets sleep 14–18 h, crepuscular); speed bump (drops flat
mid-play for a beat; owner slang, one source); stashing (carried in paws
and teeth, scooted with front paws); tunnelling (the enrichment ferrets
"most enjoyed", RVC owner survey); tail semaphore (relaxed calm, twitch
alert, puffed + bounding joy, puffed + hunched fear, tucked nervous).
Anatomy: body ≈ 3–4× shoulder height, short legs, small eyes, short ears;
sable with the dark mask is the most common and most recognisable colour;
AFA patterns include mitts, blaze, panda. Culture: Kodo and Podo
(*Beastmaster*), the *Kindergarten Cop* ferret, Furret (Pokémon, the 2019
"Furret walk"), Lesfurets.com's Hervé and François; weasel-villain
tropes do not bleed onto domestic ferrets ("cute and harmless"). Names
that checked clear or nearly: **Dook, Cutline, Mitts, Fitch**; Kit is a
baby ferret and Firefox's name, avoid. "FERRET" is a registered US mark for
inspection cameras (Ferret Tools), low risk for a culling app, noted.
Motion design built on this: `MOTION_DESIGN.md` and the motion lab.

## 12. "A fox, or something close to Kit" (owner, 2026-10-14)

**The fox problem (reported, primary policy text):** the Firefox logo is
an orange fox curled around a globe (since 0.8, 2004) and Kit is now a
second fox; Mozilla's trademark guidelines list the Firefox logo among its
marks and say third parties must not use "marks, logos, or designs that
are confusingly similar to Mozilla trademarks." MetaMask's mascot is also
a fox (low-poly, 2016, chosen for "self-custody and trust"). GitLab's
logo is a tanuki that "many Japanese people" mistake for a fox. Mozilla's
mobile browser was code-named Fennec. A warm-gradient fox with big dark
eyes and no mouth for a creative app is, in plain terms, a Kit lookalike.

**Fox-adjacent and unclaimed (reported):** no software or app mascot was
found for the **pine marten**, the **ringtail** (Arizona's state animal;
Rocky the Ringtail is a state-parks mascot) or the **dhole**. What each
brings:

| animal | Kit-likeness | story | notes |
|---|---|---|---|
| **Pine marten** | high: pointed face, round ears, bushy tail, cream bib | a mustelid like the ferret ("ferret out" family), lives in the dark woods, agile | the closest honest neighbour to Kit's silhouette without being a canid |
| **Ringtail** | medium: pointed ears, long ringed tail, enormous eyes | strictly nocturnal, eyes built for low light; the ringed tail is a signature detail nobody has | reads "raccoon" if the rings dominate; keep the face |
| **Dhole** | high: it is a wild dog that looks like a fox | obscure, so no story the public knows | the most fox-like body that is not a fox; the obscurity cuts both ways |
| Fox, fennec | identical | | reference only, see above |

Sketches in the placeholder's Kit-like rendering: `mascot/fox-adjacent/`.
Recommendation for the artist brief: ask for a **pine marten** as the
primary direction (it keeps the ferret's "ferret out" lineage and gets
Kit's silhouette), with the ringtail as the alternate. A straight fox is
off the table unless the owner wants to accept the lookalike risk.

## 13. Decision: Fen, a fennec fox (owner, 2026-10-14)

After the fox-adjacent round the owner chose a **fennec** and the name
**Fen**, with two reference styles: a storybook ink-and-pastel character
and an ultra-minimal brush-stroke mark. Placeholder drawn in both
(`mascot/fennec-storybook/`), brand kit and portal rebuilt around it
(`brand/`). Neighbours to keep in mind: Firefox's Kit and logo (foxes),
Mozilla's retired "Fennec" codename, Pokémon's Fennekin, Zootopia's
Finnick (the owner stopped the lookups mid-way, so this list is from
memory and UNVERIFIED). The ears make the fennec visually distinct from
Kit at every size, which is the practical defence; the dot-eye storybook
face also departs from Kit's rendered eyes. Owner's rule for every
version: natural, not creepy (dark eyes with a catchlight, rounded tips,
real-animal expressions). Name check for "Fen" still to do.
