# Decisions

Why arbor_day is shaped the way it is. [design.md](design.md) says how it works and does not say
why; this is the other half.

**What earns an entry.** A decision belongs here only if reversing it would break something
expensive: the species' meaning or format, every plant's output, the shape of a subsystem, or a
pattern Panoply takes. The test is that the reversal cost can be named in a sentence. If it cannot,
it is a code comment, not a decision.

**What an entry keeps.** What was decided, what it rules out, and what reversing it would cost,
with each fact that makes a rejected alternative worse. A revised entry states the decision as it
now stands, not how the argument got there; git holds the history.

Numbers are identities, cited across the docs and the code, and never reused; a withdrawn entry is
struck and kept, saying where its content went.

---

### D1 — Bevy main, named by branch

Bevy is `git = "https://github.com/bevyengine/bevy.git", branch = "main"`, the commit held in
`Cargo.lock`, bumped with the family (Panoply, the `bevy_immediate` fork, `bevy_action_map`,
`bevy_remote_driver`), one chunk per bump. Cargo treats a branch and a rev as different sources, so
any other form builds a second Bevy beside the dependencies'. **Reversing it**, to a release, waits
for one with what the app uses (0.21), and moves the whole family at once.

### D2 — `bevy_immediate` and feathers for the UI

The panel is immediate-mode, `bevy_immediate` from the fork `viridia/bevy_immediate` (branch
`bevy-main`), drawing `bevy_feathers` widgets. `bevy_remote_driver` drives plans, by git from
`viridia/bevy_remote_driver`. **Reversing it** rewrites the panel, and with it the patterns Panoply
takes (D3).

### D3 — The shell is the spike for Panoply's editor

The authoring app came before the generator, with a stub in its place, so its patterns (panel
layout, viewport, settings, plans) are a `bevy_immediate` spike for Panoply's editor (Panoply's X3).
A pattern that settles is recorded in the ledger for Panoply to take. **Reversing it** leaves
Panoply's editor to start again.

### ~~D4~~ — Withdrawn

That the requirements moved here from Panoply (2026-10-11) is history; `requirements.md` opens
with it.

### D5 — The view is a viewport following a UI node

The 3D view is a camera `Viewport` following a node (TD1.1), not a `ViewportNode` render target,
and the UI has a camera of its own: UI lays out within its camera's viewport, so on the 3D camera
it would shrink into the view pane. Panoply's X3 viewport takes this form. **Reversing it**
rewrites the view, its picking, and Panoply's viewport with it.

### D6 — App state in `bevy_settings`

Window state, the panel's width, the HUD's toggles and the color picker's state are saved with
`bevy_settings` (TD1.2), with no driver-specific name: the driver gives each run a fresh directory.
The panel keeps a fixed width in pixels, since a flex weight resizes it with the window.
**Reversing it** loses every user's saved state and the deferred save that protects it from Cmd+Q.

### D7 — A turntable camera of our own

The view uses its own turntable (TD1.3), not Bevy's pan-orbit camera, which rotates and pans
freely: every control is one more thing to reset, so there is no roll or pan, and Frame resets the
rest. It uses the game's field of view (40°, Panoply's `DEFAULT_FOV`) and frames from the game's
elevation (45°), so plants are seen as players will see them. Tilt is allowed because the hand
expects it, and Frame returns it to the game's elevation. Framing once rather than following the
subject shows an edit at its true size; following made a taller tree look smaller. A drag moves
along one axis, chosen by decayed sums; a lock chosen per gesture felt stuck, since scroll events
carry no phase and momentum kept the gesture alive. The module is shaped like
`bevy_camera_controller`'s, so it can be offered upstream (X7). **Reversing it**, to pan-orbit,
loses the fixed vertical axis and the game's framing.

### D8 — Fixed keys, not `bevy_action_map`

Keys are fixed, with no rebinding or saved bindings, so an observer of focused input does (TD1.4),
not `bevy_action_map`. **Reversing it** adds the dependency and a bindings format for no use yet.

### D9 — Field attributes shape the panel

The panel reads attributes from the species' types, after the author's `bevy_reactor_prop_inspect`
(`../bevy_reactor_3/crates/bevy_reactor_prop_inspect/src/attribute.rs`), the design borrowed, not
the code (TD1.5). Every species field has a `Caption` rather than showing its Rust name. Every number
is a feathers number input; the slider is not used. A field's `ValueRange` is its hard limit:
nothing here normally sits in a range while occasionally leaving it, so a soft limit is never a
narrower range. X25 may replace ours with `bevy::annotations`. **Reversing it** rewrites every
species field's attributes and the panel's reading of them.

### D10 — A varied parameter is a `Fuzzy`

A varied parameter is a fuzzy number (TD2.2): a peak, bounded support either side, and a shape.
Rejected names: `Varied` (vague), "window" and "error bars" (read otherwise), `Tolerance` (one of
its fields would share the name), `FuzzyNumber` (rules out a later `Fuzzy<Color>`). Where too few
values are drawn for a shape to show, such as the leaves of one sprig, a varied parameter is an
`Interval` (C38): center ± spread, drawn uniformly, its row a `Fuzzy`'s with an empty space where
the shape toggle would be. **Reversing it** changes the species' format and every varied row.

### D11 — A parametric growth model per level, in a crown envelope

The growth model is parametric per level, with a crown envelope that sets limb reach and foliage
placement, limbs bending toward directions measured against world up, leaving their parent at an
angle from it, and cards lying along their twigs (D27). The survey behind it is in
[growth-model.md](growth-model.md). **Reversing it** rewrites the generator and the species.

### D12 — One species resource, rebuilt whole on any edit

The species is one resource, and any change rebuilds the whole plant (TD2.1): most parameters change
much of the tree, so finer change detection would save little. Plans read the result and the
species it was grown from as components, since the driver reads components, not resources, and a
resource is already a component in this Bevy. The leaf texture, far costlier to make, can become a
second resource if dragging stutters. **Reversing it** means change detection per field and a
generator that rebuilds in parts.

### ~~D13~~ — Withdrawn

An enum field as a feathers select is mechanism: TD1.5.

### ~~D14~~ — Withdrawn

Phyllotaxis as limbs per node and a facing is part of D26.

### D15 — Three fixed limb levels and a count shown

Limb levels are a fixed array of three and a count of those shown, 1 to 3, not a list that grows
and shrinks: a hidden level keeps its parameters, so hiding one and showing it again loses nothing,
with no `enabled` flag per level. A fourth level would be twigs too thin to mesh. A level's
parameters mean something only at its depth, so there is no reordering. Saved, only the shown
levels are written (C10), so a hidden level lasts a session, not a save. Rejected: the species as
a list of grown levels with hidden ones stashed in the editor, the same in files but a larger
change. **Reversing it** changes the species' format and the panel's levels.

### D16 — Our own random numbers, a stream per subsystem

Random numbers come from our own SplitMix64 (TD2.2), not the `rand` crate: `rand` reserves the
right to change the values its conversions give between versions, and it is shared with Bevy's
tree, so a Bevy bump could change every plant. Each subsystem draws from a stream of its own, so
changing one leaves the others' numbers alone (requirements, Parameters, determinism, variants).
**Reversing it** changes every plant.

### D17 — Tracing in steps, and rings by visible error

A limb's course is traced in 2 cm steps (TD3.4), sagging and rising by amounts per crown radius,
so a species scaled up keeps its form, each step solved exactly, so the step size does not show,
as seedling's `flex` did. Sag scales with the sine of the angle from up, as a cantilever's load
does, so an upright limb does not sag; a bend toward a target angle turned an upright limb most. Wander then displaces the course as
a lightning bolt is drawn, by midpoints between its base and its end, both fixed, so the tip stays
where the course puts it at any wander, and a limb zigzags about a line it keeps returning to, as a
tree steers back toward light and up. Kinks fall about one spacing in crown radii apart on every
level; a zigzag reads by its period, so a long limb kinks several times and a twig once or not at
all. Each stretch points no further off its course than the wander, so no turn is sharper than
twice it, and no displacement is larger than a segment of two spacings gets, so a trunk, being
long, zigzags evenly about its course rather than turning on one large corner, as a load-bearing
stem keeps near a straight line. An axis leaves along its course for about a spacing, so a trunk
rises from the ground and a limb leaves at its angle. Less faithful to how a shoot grows, each internode turning from the last, but these are
paintings, not photographs. Kinks offset from the course one stretch after another walked the tip
off it at random, on the default tree at the most wander by 0.3 to 0.9 m and 10 to 13% lower;
tying the spacing to a density made the wander vanish on sparse levels; turns blended smoothly
added up, so limbs hooked and coiled.

Rings are hinted, as font hinting keeps only what a pixel shows: a point is kept only where it lies
further than the bark's detail threshold from the line kept, since the eye reads a branch by where
it goes and by its corners more than by gentle curvature. Rings every so many degrees of turn were
the same however the tree is seen. Most trees are meant to have two levels; three are the costly
exception, their twig count the cost (X24), and an author makes a crown denser with larger leaves
rather than more branches, so costs are judged at the densities that give it. **Reversing it**
changes every tree's bark and triangle count.

### ~~D18~~ — Withdrawn

Where varied values are drawn, and their clamping to `ValueRange`, is mechanism: TD2.2.

### ~~D19~~ — Withdrawn

Varying a value only where its instances are many is a guideline: G2.

### D20 — Leaf textures painted with `tiny-skia`

Leaf textures are painted on the CPU with `tiny-skia`, a pure-Rust port of part of Skia, which
covers seedling's Canvas2D subset almost one to one (paths, fills and strokes, transforms, linear
gradients, paint alpha), and on the CPU, so the command-line export (X1) can use it. Rejected:
`vello` (GPU, so not for the command line), `vello_cpu` (likely tiny-skia's successor, its API still
moving; revisit when it settles), `raqote` (quieter), Cairo (a C library to build and ship), `lyon`
(tessellates, does not rasterize). **Reversing it** rewrites the painter.

### D21 — Nodes by density per crown radius

A level's nodes are a density per crown radius of the parent's part inside the crown, not per
metre, as bend is measured against the crown (D17): a species scaled up keeps its form and its
triangle count. Seedling's interval was absolute, as real internodes nearly are. **Reversing it**
changes what every level's density means.

### D22 — Radii by one taper, top-down; every stem drawn

Radii come top-down from the trunk's, which the author sets (TD3.5): a level's radius share sets
that level alone, as the requirements ask. One taper for all wood, as seedling had. The pipe model
(a stem's radius raised to an exponent, shared among the limbs leaving it) made raising a level's
radius thin the trunk past each node and every later limb; it returns as a stack operator writing
radii (X50), bringing back the trunk narrowing more where many limbs leave it. Bottom-up from the
twigs made the trunk's radius follow the limb count. A taper per level is X32.

Every stem is drawn, none narrower than the bark's detail threshold (2 cm by default, about a pixel
at the game's scale), as font hinting keeps a stem a pixel wide: the aim is verisimilitude, not
realism (requirements, What it is for). Only the drawing thickens; the radii stay true. A cutoff
instead left leaves floating off visible tips, and twigs bare. The threshold is also the rings'
tolerance (D17), one setting for what the game can show. Sides stay per level for Panoply's
inverted-hull outline, which extrudes along vertex normals, so the line swells and thins with the
view (50% to 100% at 3 sides, 71% at 4, 87% at 6): 3 for thin levels, 4 or more for the trunk and
the first level. **Reversing it** changes every tree's radii, and with the threshold, its triangle
count.

### D23 — The trunk's height is the plant's one size

The trunk's height in metres is the plant's one size, and the crown is measured against it, its top
a multiple and its base a share, so a ratio suits a herb as well as a tree, and moving the crown
never moves the trunk. The stems' length as a share of the envelope's height made raising the crown
drag the trunk with it. The trunk's heading is drawn from a stream of its own, so neighbouring
trees of one species lean different ways. The height is the length along the stems from the
ground to each tip, through any fork (D31), which wander leaves alone, so an upright stem's tip
reaches it at any wander. **Reversing it** changes every species' sizes.

### D24 — The crown is the author's

Each crown setting changes one thing, and the crown is never resized to fit the stems. A stem keeps
its length whatever its splay and lean, so leaning droops it, and the lean stops at level so a stem
never sinks (C48). Nodes lie along the part of their parent inside the crown, so the crown base alone
sets where a stem's limbs begin, and a stem that misses the crown bears none. Rejected: fitting the
crown to the stems, so no setting could leave them outside it, which made lean lengthen the stems
and widen the crown, and raising the crown base lengthen splayed stems to several metres.
**Reversing it** changes how every crown and stem setting reads.

### ~~D25~~ — Withdrawn

Changing a field's meaning before adding a field is a guideline: G1.

### D26 — Children placed where they compete least

Children are placed by competition among siblings on one parent and a facing per level (TD3.3),
not by a fixed divergence: a simple form of space colonization's competition for light. It is how
botany explains phyllotaxis (Hofmeister's rule, that a new bud forms furthest from the last ones;
Douady and Couder, that an inhibition fading with distance yields the golden angle, crossing pairs
and alternating whorls), so the spiral comes out of the rule, and the same rule serves lateral
limbs, which a divergence served poorly. Limbs per node and the facing give the spiral, crossing
pairs and alternating whorls, each field changing one visible thing. The facings are the patterns
botany names on sloping limbs (Hallé, Oldeman and Tomlinson); no tree fans vertically. The constants
were tuned in simulation: gentler crowding falls to 180° turns, sharper turns erratic.

Rejected: facing from the flank with a divergence of 180°, which flattened lateral limbs only when
set and left the trunk's spiral fixed; facing as a weighted preference beside competition, which
up and down could not apply to a pair; room to the envelope for light, which near the crown's edge
turns children inward; full space colonization (X12), with limbs on different parents competing,
which stays deferred. **Reversing it** changes every plant's arrangement and the levels' fields.

### D27 — A card is a sprig along a twig, at every branch end; a rosette where it is steep

A card is a sprig of leaves on a painted stem, lined up with a real twig, after seedling
(growth-model.md, Foliage): its base on the twig, its axis along it, its face as near world up as
the axis allows (TD3.8, TD3.9). Seen edge-on, a card then loses leaves but never a twig. One cluster
grows at the tip of every branch end, whatever its level, so no visible end is bare; taking off
the tip leaves leaves them all bare. Card size is a share of the crown radius, so a scaled species keeps its
form (D21). Denser foliage comes from another limb level. The tip's card reaches back so its axial
fold lies at the tip, as seedling's did, and the twig runs straight under it (D17).

A steep tip bears what the species chooses, by default a rosette: several cards from the tip,
splayed around the twig toward level. A level shoot twists its leaves into one flat spray, which
one card models; an upright one bears them all around, each blade turned toward level, which one
card along it showed on end, dark from the game's camera, with nothing on its other sides. The
choice is a rosette of two or three, one card as on a level twig, for crops and shrubs seen from
above, or none. A count rising with steepness, two from 45° and three from 75°, was rejected:
a third card showed only in a narrow range of settings.

Rejected: clusters at a density along each twig within a shell of the envelope, where any density
put cards along twigs, none made the plant bare, and those near a tip ran past the tip's card
(X30); the last grown level only, which left an earlier level's childless limbs bare; a free turn
about the normal, which read as scattered leaves; every limb bearing clusters, which put leaves on
boughs; pitching the one card toward level, which turned it to one side and left the others
bare; a third limb level to fill the crown's top, which roughly doubles the triangles. Seedling's
per-level leaf flag waits for a preset that needs it. **Reversing it** changes every plant's
foliage and what its fields mean.

### D28 — The HUD changes the preview, not the species

Controls that change the preview rather than the species sit over the view (TD1.6), after Google
Maps' controls turned upside down, so the panel holds only the species. A camera button moves while
held; a step on the press, then smooth movement after a delay, felt wrong. Its buttons are our own,
in fixed translucent grays rather than theme tokens: the theme is for a light and dark switch,
which a HUD over the scene does not follow, so forking feathers' button and registering tokens
would buy nothing. A toggle's state is saved, as the panel's width is and the camera is not (D7).
**Reversing it** puts the view's controls back in the panel, where the species belongs.

### D29 — The leaf texture is seedling's, with five changes

The leaf texture (C38) is seedling's (`genLeafShape.ts`, `genLeafTexture.ts`, and
`createLeafStamps` in `MeshGenerator.ts`), the look the author tuned. Kept: a leaf is a cubic
Bézier half-outline, mirrored, split into segments; each segment's end is pushed outward
(serration) and toward the tip (rake), with jitter, and a straight line back to the curve makes the
tooth; serration also sharpens the tip. A sprig is a fan (count, angle, a size taper per step) and
pairs down the stem at the fan's angle. Each leaf is filled with an inner and an outer color, in a
lateral or axial gradient in its own frame, shifted in hue and lightness at random per leaf. Every
leaf is stroked black first, then each is filled and stroked translucent dark, so the black shows
only around the cluster and the leaves within it are parted by faint lines. The texture is square,
fitted to the sprig. Changed:
- Colors are painted as picked. Seedling painted linear values as sRGB, so its leaves came out far
  darker than their swatches (poplar-green's outer `rgb(80,201,80)` painted near `rgb(20,149,20)`),
  and its stored colors are not targets.
- Lengths are fractions of the leaf's length, which is not a field: seedling's length mattered only
  against constants in absolute units, so changing it changed the teeth and the outline's weight.
  The outline's width is in texture pixels.
- A painted stem, which seedling coded but left off: a midrib in the bark color, outlined, from the
  texture's bottom edge (the card's base on the twig, D27) through the pairs to the fan, its width a
  constant.
- The angle and spacing variations are `Interval`s (D10); the color shift per leaf is in Oklch
  rather than HSL, so a step of lightness looks alike on every hue.
- Shape, arrangement and color draw from streams of their own (D16); seedling's one stream
  reshuffled the leaf on any edit.

The fan's angle still sets the pairs', until a preset needs them apart. The droops shape the card,
not the texture. Seedling's 47 models used every field, jitter least (3). **Reversing it** loses
the look the author tuned.

### D30 — A parent runs on to its own tip

A stem or limb bearing limbs runs on past its last node to its own tip, its nodes spread from
its level's start to there, and the tip bears leaves as any branch end does (D27, TD3.6). Only a
fork cuts an axis (D31). Rejected: a carry-on, the parent ending at its last node and one more
limb of its children's level carrying it on, so the trunk turned into a branch rather than ending
in a post. It switched the axis's sag, sides, wander and reach at the joint, started the next
level's laterals at that level's start, leaving a bare stretch at each joint, and needed a reach
of its own, since at the level's reach a leader stood above the crown and every limb ended in a
trident. Placement settings varying along the parent (X53, C116) are to shorten and steepen the
upper laterals instead. **Reversing it** brings back the carrier, its reach and the switch at
each joint.

### D31 — Forks after Weber and Penn's splits

A limb forks with its level's chance at one point and ends there, and copies of it carry it on,
the same level as it rather than its children, so they reach the envelope and bear limbs as it
would (TD3.7): two make a Y from level to upright by the roll, three to five a ring, as a shrub's
low split. Copies do not fork again. The trunk forks the same way, its copies stems (TD3.2): a
codominant split partway up, or at its base a clump, which replaced the trunk's stem count and
splay (G1), several splayed stems being a fork at the ground. A fork at an axis's base leaves no
stub; its copies take its place. **Reversing it** removes the Y, the split trunk and the clump
from the shapes a species can take.

### D32 — No file compatibility before the first species ships

Until X15's gate, the first species used in Panoply, the species schema changes freely: a chunk
that renames, moves or removes a species field breaks every file written before it, and no
version or migration covers it. Its Landed row says "Breaks files", so a saved species or working
document is known not to survive it. Reading stays strict, so a stale file fails with an error
naming the key rather than loading wrong (C10). Compatibility starts at the gate, as X15 and
`docs/patches.md` (The file) lay out. **Reversing it** means a migration for every schema change
from C10 on, for files that are only test artifacts.

### D33 — Operators are added on purpose; the default is bare

A new species, and a limb level added to one, starts with no operator on: no lean, sag, rise,
wander or fork. An author adds each behaviour a species needs, so a species with straight limbs
carries no rows or concepts for bending ones, and every behaviour in a species was chosen rather
than inherited. Tip leaves are the exception, an ornament slot on in the default, since leaves are what make a plant read as alive; a bare one is a dead
tree, which the tool also makes, by taking them off. Anything with a value
that does nothing is an operator, however common; the frame keeps only what has no such value,
as reach and radius, which no species does without. **Reversing it** gives every species the
default's behaviours to undo, and the panel the rows for them.

### D34 — One file type: the saved file is the glTF

Save writes the exported glTF, the species a JSON extra on its scene (C10's format), and Open
reads the species back from it (C40, C41). There is no separate species file, so no new extension
to invent, and no JSON file of another kind to open by mistake: a file that opens is a plant, and
the mesh the game loads is never out of step with the species that made it. **Reversing it** adds
a second file type, its extension, and the work of keeping the two in step.
