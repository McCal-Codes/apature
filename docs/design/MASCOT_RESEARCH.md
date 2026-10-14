# Mascot research: what makes a character work, and which animal

2026-10-14. Sources are linked at the end; claims marked *measured* come
from the studies cited, *reported* from brand-team interviews and press,
*inferred* is our reading. The owner has provisionally chosen the owl
("Lumen") and asked whether another animal might be better.

## 1. Why things look cute (the science)

**Baby schema (Kindchenschema).** Lorenz's list, confirmed in part by
later experiments (Sternglanz 1977; Glocker 2009, *measured*): a large
head relative to the body, big eyes set **below the horizontal midline of
the face**, a high rounded forehead, chubby cheeks, small nose and mouth,
round body, short thick limbs, soft surface. Glocker's group found
cuteness ratings rise with the strength of the schema and that it
motivates caretaking. A 2022 morphing study (*measured*) found the
**low position of the features and a narrow jaw** mattered and forehead
height did not. Implication for any Apature character:

* eyes in the lower half of the head, not the middle;
* head ≥ 50 % of total height in the "full-body" version;
* no neck, short limbs, every corner rounded;
* one small feature for nose/beak, mouth optional.

**Kawaii minimalism** (*reported*): two dot eyes and a tiny or absent
mouth stay expressive because the proportions do the work; Hello Kitty's
missing mouth lets people project their own mood. Minecraft's 2026 cub
update made baby mobs cuter by giving them **one-pixel eyes** instead of
two, and the new Blub mob is a palm-sized glowing jelly with almost no
features. Fewer features, placed low, read cuter than more features.

## 2. How good software mascots are built (lessons)

| brand | lesson | source |
|---|---|---|
| Duolingo, Duo (2019 redesign) | Fix species and colour first; **silhouette is the most important element**; move to geometric flat shapes so anyone can animate it; think "character, not logo"; test many expressions and poses before locking the design; the mascot talks to the user, so it needs a personality | Duolingo blog, Design Week |
| Firefox, Kit (2026) | Built on what users already associated with the brand; **no mouth**, emotion carried by eyes, posture and tail; appears at onboarding, feature discovery and small wins, "then steps back so you can keep moving"; never competes with the UI; drawn by hand | Mozilla blog, Abduzeedo, Creative Lives |
| Mailchimp, Freddie | A mascot that must sit beside a wordmark becomes a **single-colour silhouette**; the illustrated version lives elsewhere | Creative Bloq |
| Go gopher | A character can be openly licensed (**CC BY 4.0**, attribution to Renee French) while the logo stays separate; "the gopher is a character, not the logo" | go.dev |
| GitHub Octocat | The opposite: registered trademark, no use as an app icon by others, written permission for merch | Octodex FAQ |
| Minecraft Blub | Tiny, glowing, jiggling; cuteness from scale, restraint and motion | Minecraft wiki |

**What Apature takes from this (inferred):**

1. Decide the silhouette before the face. It must read at 16 px and as a
   one-colour cut-out.
2. Eyes only, or eyes plus a beak. No mouth in the mark; a mouth is
   allowed in full-body poses but is not required.
3. Flat geometric parts from one library so poses stay consistent (the
   generator scripts in `docs/design/brand` already work this way).
4. Appearance rules: onboarding, empty states, finished jobs, failures;
   never on the contact sheet, preview, caption fields or toolbar.
5. Licence: ship the character art under **CC BY 4.0** credited to the
   project, keep the name and the app icon as the project's marks (the Go
   model). Record this in THIRD_PARTY/LICENSE when adopted.

## 3. The owl problem

Owls are the most crowded animal in software branding: **Duolingo** (the
most recognised app mascot of the decade, green, big eyes), **Hootsuite**
(Owly), **Tripadvisor** (owl logo), plus Hooters and many universities.
Lumen's differentiators are the dark body, lavender lens-ring eyes and the
camera story, and at 16 px a dark round owl with two big light eyes still
risks reading as "a Duo variant". That is a real cost, not a fatal one:
Firefox chose a fox knowing foxes are everywhere, because the association
already existed. Apature has no existing association, so the owl is a
choice made from scratch, and the crowding counts against it.

## 4. Other animals, scored

Scale 1–5. *Theme* = a camera idea lives in the animal's own anatomy or
behaviour. *Free* = no major software or photo brand owns it. *Cute* =
baby-schema potential. *Icon* = legible silhouette at 16 px. *Range* =
poses and expressions available without props.

| animal | theme | free | cute | icon | range | the story |
|---|---|---|---|---|---|---|
| Owl (Lumen) | 4 | **1** | 5 | 5 | 4 | sees in the dark; binocular lens eyes |
| **Kingfisher** | 5 | 5 | 4 | 4 | 4 | the decisive moment: waits, then dives once, never misses; the sharpest shooter on the river; Birda (a birding app) uses one, no major brand |
| **Lightning bug / firefly** | 5 | 3 | 4 | 3 | 3 | makes its own light: the flash; night worker; "shutterbug" kin. Adobe Firefly owns the *name* in creative software, so the character could never be called Firefly |
| **Mantis shrimp** | 5 | 5 | 3 | 3 | 4 | 12–16 photoreceptor types and polarised-light vision, "the best eyes in nature" (*measured*, with the caveat that it discriminates colour worse than we do); the most original story, hardest to make cute |
| Magpie | 3 | 4 | 3 | 4 | 4 | collects shiny things: picks the keepers; Apache Magpie exists but is obscure |
| Hummingbird | 4 | 3 | 4 | 3 | 3 | speed, 1/4000 s wingbeats; Google's "Hummingbird" is an algorithm name only |
| Raccoon (Bandit) | 3 | 4 | 4 | 4 | 5 | night, hands, mask as viewfinder; cute and poseable, weak camera link |
| Cat (Kit) | 3 | 2 | 5 | 4 | 4 | GitHub's Octocat and Firefox's *Kit* (the name) crowd it |
| Otter | 2 | 2 | 5 | 3 | 3 | Otter.ai owns the name in software |
| Chameleon | 4 | 3 | 3 | 3 | 3 | SUSE's gecko is close |
| Red panda | 2 | 1 | 5 | 3 | 3 | literally Firefox's namesake |
| Capybara, axolotl | 1 | 4 | 5 | 3 | 3 | cute and fashionable, no photography story; axolotl is Minecraft's |

**Reading:** the owl wins on cuteness and legibility and loses on
ownership. Three animals beat it on story and are free: the kingfisher,
the lightning bug and the mantis shrimp. Sketches of those three next to
Lumen are in `mascot/animals-2/`.

## 5. Recommendation

* If the brand should feel **warm and safe**: keep Lumen, and lean hard
  on the differentiators: charcoal body, lavender lens rings, low-set
  eyes, no mouth in the mark, dark-ground icon tile. Accept the Duolingo
  adjacency.
* If the brand should feel **sharp and its own**: the **kingfisher**. It
  is the only candidate whose story is the craft itself (patience, then
  one decisive, accurate shot), it is unclaimed, it takes the system-blue
  accent naturally in Apature Light, and its silhouette (big head, long
  beak, stubby tail) is distinct at 16 px.
* The mantis shrimp is the one to keep as a **secondary character** or
  an easter egg: its eyes are the best fact in the whole brief.

Whichever is chosen, build it the Duolingo way: silhouette first, then
geometric parts, then twenty expressions on paper before one in the app.

## Sources

* Glocker et al. 2009, "Baby schema in infant faces induces cuteness
  perception and motivation for caretaking in adults", Ethology (PMC3260535)
* Frontiers in Psychology 2022, face-space morphing study of cuteness (10.3389/fpsyg.2022.979341)
* Discover Magazine, "Why babies are so cute" (baby schema history)
* Duolingo blog, "Reshaping Duo"; Design Week, Jan 2019; Creative Bloq on Feather Bold
* Mozilla blog, "Meet Kit, your companion for a new internet era" (2026); Abduzeedo; Creative Lives in Progress
* Creative Bloq, Mailchimp rebrand
* go.dev/doc/gopher (CC BY 4.0); Octodex FAQ; GitHub logo guidelines
* Minecraft Wiki, Blub; Dexerto on the Blub reveal
* Fabrik Brands, famous owl logos; Branition, owl logos
* The Wire Science / Discover on mantis shrimp vision; Wikibooks Sensory Systems
* Birda species guide (kingfisher app logo); Apache Magpie brand page
