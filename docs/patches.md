# Patches

A design for forms as data (ledger X22): a plant described by a patch, a file of operators
whose inputs are formulas over the author's own controls, run in Rust over an abstract plant
and then realized as meshes. Not built.

## Why a language

A form is a set of relations, not a set of values. "Height" should raise the trunk, widen the
crown, thicken the stems and thin the twigs at once; "droop" should lower every level's angle and
bend target together, more on outer levels. Dials on the engine's fields cannot say this: each
field is one number, and the author keeps the relations in their head. Presets (X18) save values,
not relations, so a preset scaled up is a different tree.

After Reaktor, a patch has a front panel of controls, and formulas wire each control to as many
operator inputs as it should move, combined with other controls by arithmetic. After Blender's
geometry nodes, the operators act on a plant, not on numbers: each takes a plant and returns one,
as Merge Points takes a geometry and returns another. A patch is edited as a node graph (Node
graph, below) and saved as a file: operators in order, their inputs written as spreadsheet
formulas in strings, so the file stays readable and needs no AST spelled out in JSON.

Formulas never reach the vertices. Placing rings, tracing paths, sharing wood and meshing stay in
Rust, so the language is never a slow, clumsy copy of the generator. A formula is evaluated once
per plant or once per element (an axis, a node), never per ring or vertex.

## Three layers

1. **Controls**: the panel. Each has an id, a caption, a range, a default and units, and becomes a
   row as a species field does now (D9's attributes, read from the patch rather than the type).
2. **Operators**: a pipeline over the abstract plant (below), each input a number or a formula.
3. **Realization**: a Rust library turning the finished abstract plant into bark and cards:
   the drawn minimum radius (D22), tubes, cards. Its settings are the game's, not the form's (the
   detail threshold, sides per level), so it sits outside the patch. Hinting (D17) shares the
   detail threshold but runs at the end of each level, since the next level is placed on it.

## The abstract plant

Operators act on a skeleton, and meshes are made from it once they have all run. The skeleton
is a list of axes, an axis in botany's sense: one stem of any order, the trunk, a limb or a
twig, not a direction.

A limb is two things. Its **placement** says where it starts and which way it leaves: on a
parent, at a place along it, an azimuth and an angle; or for a stem, on the ground, at a point and
a direction. Its **composition** says what it is from there: its course, its length, its wander,
its radius, its children. The trunk differs from a limb only in its placement, so every
operator on composition serves both. The generator has one `Axis` for both
(`src/generator/skeleton.rs`), its `kind` saying how it came to grow.

The skeleton as built is TD3.1's. Under stacks, what each ornament places (Stacks, below) is a
field of its own, as its slot is in the species, today's cards being the tip leaves':

```rust
pub struct Skeleton {
    pub axes: Vec<Axis>,
    /// Placed on the grown axes, after every level has grown: one field per ornament (C103).
    pub tip_leaves: Vec<Card>,
    pub twig_leaves: Vec<Card>,
    pub radiating: Vec<Card>,
    pub fruit: Vec<Placement>,
}

pub struct Axis {
    pub level: usize,
    pub kind: AxisKind,
    /// The parent, by index, and the distance along it where this axis leaves; none for a stem.
    pub parent: Option<(usize, f32)>,
    /// Base to tip, a ring at every trace step until hinted (D17), at least two.
    pub rings: Vec<Ring>,
    /// How far it grows, as placed.
    pub reach: f32,
    /// Before any draw; each operator forks its own stream from it (D16).
    pub rng: Rng,
    /// By index: its limbs, or its fork's copies.
    pub scions: Vec<usize>,
}

pub enum AxisKind { Stem, Lateral, ForkCopy }

pub struct Ring {
    pub center: Vec3,
    pub tangent: Vec3,
    /// Square to the tangent, carried along without twisting.
    pub side: Vec3,
    /// Distance along the axis from its base; the last ring's is its length.
    pub at: f32,
    pub radius: f32,
}

/// A leaf card whose base lies `at` metres along an axis, pointing `direction` (D27).
pub struct Card {
    pub axis: usize,
    pub at: f32,
    pub direction: Vec3,
}

/// The same for a mesh, such as a fruit, rather than a card.
pub struct Placement {
    pub axis: usize,
    pub at: f32,
    pub direction: Vec3,
}

// In the species, beside the levels: one slot per ornament, as a level's operators are.
pub struct Ornaments {
    pub tip_leaves: Toggle<TipLeaves>,
    pub twig_leaves: Toggle<TwigLeaves>,
    pub radiating: Toggle<Radiating>,
    pub fruit: Toggle<Fruit>,
}
```

- The start is the first ring and the length the last ring's `at`, so neither is stored. The
  start frame is the parent's frame at the attachment.
- An axis keeps the full trace through its level's operators; hinting (D17) simplifies it as the
  level's last stage, before the next level is placed, so scions start on the bark as meshed.
- Whether an axis bears scions, or tip leaves, follows from its `scions`.
- Size does not pass from parent to scion. A scion near its parent's tip is already shorter, its
  reach a share of the distance to the envelope, which narrows toward the top of most profiles,
  and thinner, its radius a share of the tapered parent's there. Where a form wants more, a size
  bias scales reach by place.

Operators come in three kinds:

- **Tracing**: the frame's growth traces an axis from its start, and bend and weep are read while
  it does, since changing the course changes where the envelope stops it. They cannot be laid
  over a finished path.
- **Displacing**: wander moves rings, then carries the sides along again, which the skeleton
  provides as one function for every operator that moves rings. Wander clips at the envelope
  again.
- **Restructuring**: scions add axes, and a fork (D31) cuts an axis and adds others.
  These grow the list while walking it, so axes are named by index, not borrowed.

Operators run level by level, since scions are placed on their parent's finished path; the pipe
model alone waits for the whole skeleton.

The skeleton is plain data, not entities: the generator stays a pure function of the species, as
a library without the ECS would need, an edit rebuilds thousands of axes without spawning, and an
operator reads a parent while writing a scion by index. Selecting a limb in the editor, or a plan
reading one, can spawn a view of the skeleton later.

Every random draw comes from the element's stream forked by the operator's id and the input's
name (D16), so adding an operator or an input reshuffles nothing else.

## Operators

Each operator names the axes it acts on by tag (`on`), as Blender's selections and Houdini's
groups do. Built-in tags: `ends` (axes bearing none) and `all`.

Placement:
- `stems`: attachments on the ground, spaced around (count, splay, heading).
- `nodes`: attachments along the axes `on`, within the crown: density, per node, start, facing,
  skew (D21, D26, TD3.3). Competition stays inside the operator; facing is an input.

Composition:
- `grow`: attachments into axes: length or reach to the envelope, bend toward, bend (D17's
  course), stopping at the envelope.
- `wander`: displaces each path between its base and the end of its course, both fixed, by
  midpoints, as a lightning bolt (C84), on straight or bent paths alike.

Whole plant:
- `pipe`: radii top-down from the stems (D22), each axis taking its `share` attribute, in
  place of the taper.
- `clusters`: a card at the tip of each axis `on` (usually `ends`) (D27).

The current generator fuses some of these: a limb is traced, kinked and clipped in one loop. As
operators they run in sequence, which moves some tips slightly (X15's version covers it).

## Stacks

A stage before formulas: optional operators on each level, their inputs plain values, with no
controls layer. Each operator has a fixed place in a level's sequence, so the author chooses
which operators a level has, never their order.

Each level, the trunk included, has a fixed frame and its operators. The frame is what every
level needs: for the trunk its stems, height, radius and sides; for a limb level its angle,
reach, radius and sides. Without operators a level grows straight along its angle.

A behaviour is an operator when it has a value that turns it off (a fork's chance of 0, no sag,
no wander), however many species use it (D33); what has no such value, as reach and radius, is
the frame's, and an
operator's own settings (a foliage card's facing and shape) are its fields. The operators, in
their fixed order:

1. **Bias** (C116, C130): the reach changing with the node's place along the parent, about the
   middle node's, and the direction aimed at the envelope's outline, for a form whose envelope
   does not shorten or steepen its upper scions enough.
2. **Sag** and **Rise**: the course turning down by weight and up toward light (D17, C94).
   Without them the course is straight. On the
   trunk it curves the stem after the tilt it leaves the ground at (the splay, and the lean's
   tilt), which gives it the heading a bend keeps.
3. **Weep**: the course allowed past level and below the envelope, for arching stems (C48) and
   pendulous levels (X19).
4. **Wander**: the course displaced as a lightning bolt (C84), with an amplitude (today's
   wander) and a frequency in kinks per crown radius, so a level can kink faster or slower than
   the fixed spacing every level has now. Its options: a roughness falling off at smaller
   scales, and kinks at the nodes, which needs nodes laid out before the trace and so
   strains the order.
5. **Flare**: the trunk only, its stems widening toward the ground (C122, TD3.2).
6. **Fork**: D31's five fields; on the trunk, stem forks (C75).
7. **Scions**: the next level, placed on this one's axes. It has no settings of its own: how
   the level is placed (density, per node, facing, skew, start: the placement half of a limb)
   is that level's Arrangement, since an author tunes a level as one thing, as SpeedTree's
   generators carry their own generation settings. It shows as the next level's pane, not as a
   heading. Not offered on the last of three limb levels (D15). Space colonization (X12) would be
   a second way to place a level, a mode of its Arrangement.

Levels grow in turn: a level's frame and operators, then its scions' level.

- A level's header has an add button (a boxed plus) opening a menu of the operators the level
  lacks; choosing one inserts it in its place, and a ✕ in an operator's header removes it.
- C33's light headings, not nested boxes: each level a subpane and each operator a heading in
  it, after the operator's icon, its eye and ✕ at the heading's right. Making each level a
  feathers pane and each operator a subpane in it was weighed and withdrawn: the headings read
  well enough.
- Scions replace the − and + on the levels' headers (D15): below a divider in the last shown
  level's add menu, after the operators, the next level ("Limbs 2") shows its pane, its
  Arrangement rows first. Operators the level has are left out of the menu, or shown grayed,
  judged when it is built. The ✕ on a limb
  level's own header hides it and the levels after it, which keep their values, as D15's hidden
  levels do; the level is what was added, so it is what is removed, and Scions has no heading
  left empty but for its ✕.
- An operator a level lacks shows no rows: a twig level without wander has no wander fields. A
  new behaviour's fields go in an operator where they can (G4).
- Each operator is a field of its level, in the fixed order, so a level has at most one of each:
  a `Toggle<T>`, its settings and its state: off, disabled or on, as Blender's modifiers are
  removed, hidden or shown. The add menu turns it on, ✕ off, and an eye switches it between on
  and disabled, so an author compares a plant with and without it and loses nothing; an operator
  removed and added again keeps its values for the session, as a hidden level does (D15); files
  write the operators that are not off. Each level's defaults hold every operator's settings,
  varying by depth as the levels' defaults do. The generator calls an operator's function (C82)
  where it is on. The property editor shows its rows unless it is off, and lists it in the add
  menu while off.
- Levels stay D15's array of three and its count of those shown: a level's Scions are on while
  the count reaches past it, so paths stay `limbs.levels[1].angle` and only D15's panel changes.

The crown is fixed at the head, the plant's, not a level's. At the foot is a second stack, the
**ornaments** (C103): what is placed on the grown skeleton without changing it, so its order
never moves a limb. Like a level's operators, each ornament is a slot, one of each kind. The
first is today's foliage, the tip leaves: cards on every branch end, whatever its level (D27),
with its size, droops and upright tips. Others would be
leaves along twigs (X30), radiating cards (C54) and fruit, each a placement (tips, along a
level's twigs, at nodes, radiating) and what it draws. Cards draw from regions of one leaf
texture, since each texture is a material in Panoply and would split its merged flora meshes
(requirements, Foliage), a constraint on C38's painter; fruit would be small meshes in vertex
color, merging with the bark. The Foliage header's add menu lists the slots that are off.

### The file

A species file is the species written by reflection (C10): JSON as the glTF extra, TOML in
settings. The default species, abridged, angles in radians:

```json
{
  "generator": 1,
  "seed": 1, "detail": 0.02, "taper": 0.6,
  "crown": { "profile": "Ellipsoid", "top": 1.25, "base": 0.4, "radius": 2.5 },
  "trunk": {
    "stems": 1, "splay": 0.0, "height": 5.0, "radius": 0.2, "sides": 6, "lean": { "angle": 0.26 },
    "wander": { "angle": 0.14 }
  },
  "limbs": [
    {
      "density": 8.0, "per_node": 1.0, "facing": "Around", "skew": 0.35, "start": 0.0,
      "angle": { "center": 1.05, "spread": 0.17, "shape": "Bell" },
      "reach": { "center": 0.9, "spread": 0.1, "shape": "Bell" },
      "radius": 0.5, "sides": 4,
      "sag": { "amount": 0.26 },
      "wander": { "angle": 0.17 }
    }
  ],
  "ornaments": {
    "tip_leaves": { "size": 0.25, "lateral_droop": -0.28, "axial_droop": 0.3, "upright": "Rosette" }
  }
}
```

- **Levels**: the shown levels only, as C10's `LimbLevels` writes them; a level's Scions are on
  while the list reaches past it.
- **Operators**: one that is on is written as its settings, under its slot's name; one disabled,
  as its settings after `"enabled": false`; one that is off is not written. A level leaves out
  the toggles that are off, by type data on the level, as `LimbLevels` has. Ornaments are written the same way, so
  a bare plant has no `tip_leaves`.
- **Reading**: reflection reads a struct with fields missing and rejects one with a field it does
  not know (`bevy_reflect`'s `visit_struct`). The file is applied onto the template, the species
  with every operator off and each level's frame at its depth's default, so a missing operator is
  off and grows as a file written before it existed did (X15's third measure). The template is
  the new species' default with its tip leaves off, the one slot a new species has on (D33). A file from a newer generator, naming
  an operator this one lacks, fails to load rather than loading wrong.
- **Versions**: a new version of an operator is another slot beside the old (`wander_2`), at most
  one of the two on; the old one is read and shown while on but left out of the add menu (a
  `Legacy` attribute), so an old file grows as it did and its author upgrades on purpose. The
  slot's name is the file's key, so the key names the version. `generator` counts the changes
  outside operators (the skeleton, stage order, realization), X15's second measure.
- **Stream keys**: an operator draws from each axis's stream forked by its kind's key, one key for
  all its versions, since a version replaces its predecessor and never sits beside it. The streams
  in use keep their numbers (`WANDER_STREAM` and the rest, `src/generator/skeleton.rs`), so the
  default species grows as before across stacks; a new operator's key is the FNV-1a hash of its
  kind's name, as D16 names streams, far from the small indices scions take. Inside an operator,
  draws stay keyed by input name (`draw(&reach, "reach", …)`).
- **Slots, not a list**: one of each operator per level, so the panel stays a walk of reflected
  fields with one new widget kind (the `Toggle` group, its ✕ and the header's add menu), and field
  paths, which row keys, C7's undo records and plans' `at=` read, stay fixed. A list, an operator
  twice in one level, needs list and data-enum widgets and ids that survive a removal, which
  patches' steps (X22) need anyway, so it waits for them.

C10 need not wait for stacks. Each level's fields are grouped into its operators' structs, every
one always on (`limbs.levels[0].bend.toward`, `limbs.levels[0].fork.chance`, C96), each shown as a heading in
the level's subpane. C10's file then has the stacks file's shape with every operator written, and
when stacks wrap the structs in `Toggle`, such a file reads as every operator on and grows as
before; stacks change no file.

Formulas then come as a later stage: any input may become a formula, and the levels and their
operators become the patch's steps, with controls added above them.

## Formulas

An input is a number or a string formula:

- Arithmetic, comparison, `if(cond, a, b)`, and functions: `lerp`, `clamp`, `min`, `max`, `smoothstep`.
- Degrees written as `°` (`45°`), stored as radians.
- Controls by id; patch values (`crown.radius`, `seed`).
- Per element, in inputs evaluated per attachment or axis: `t` (the place along the parent
  inside the crown, 0 to 1), `depth`, `index`, `parent.length`.
- Draws: `bell(center, spread)` and `uniform(center, spread)`, drawn per element, replacing
  `Fuzzy` (D10) as values.

`t` is what X22 once called a curve over the parent's length: angle by place is
`angle: "lerp(70°, 40°, t)"`, with no field of its own.

## A patch

```json
{
  "controls": {
    "height":   { "caption": "Height", "range": [0.5, 30], "default": 6, "precision": 1 },
    "fullness": { "range": [0, 1], "default": 0.5, "units": "ratio_percent" },
    "droop":    { "range": [0, 1], "default": 0.2, "units": "ratio_percent" }
  },
  "crown": {
    "profile": "ellipsoid",
    "radius": "height * 0.4",
    "base": "0.4",
    "top": "1.2"
  },
  "steps": [
    { "stems": { "tag": "trunk", "count": 1, "splay": "0°" } },
    { "grow": { "on": "trunk", "length": "height", "bend_toward": "15°", "bend": "0.3" } },
    { "wander": { "on": "trunk", "angle": "8°" } },
    { "nodes": { "on": "trunk", "tag": "bough",
                 "density": "lerp(2, 6, fullness)", "start": "0.2", "facing": "around" } },
    { "grow": { "on": "bough",
                "angle": "bell(lerp(50°, 80°, droop) - 20° * t, 10°)",
                "reach": "bell(0.9, 0.1)",
                "bend_toward": "lerp(80°, 130°, droop)", "bend": "0.4" } },
    { "wander": { "on": "bough", "angle": "10°" } },
    { "pipe": { "exponent": 2.5, "share": "if(depth == 0, 1, 0.5)" } },
    { "clusters": { "on": "ends" } }
  ]
}
```

Each step is an object with one key, the operator, as a MongoDB pipeline stage is, and as
`bevy_reflect` and serde write an enum by default.

A species is then a patch, the controls' values and a seed: the glTF extra (C10) holds those, and
the patch's name and version (X15).

## Node graph

The editor shows a patch as a graph with two kinds of wire, drawn differently:

- **Structure**: the plant, or the axes a step makes, flowing from one operator to the next. A
  scion node fed from the trunk outputs limbs, which other operators then take. A step's `on`
  tag is its structure wire. One output may feed several nodes, so a trunk bears its lower and
  upper limbs from two scion nodes with different settings.
- **Values**: inputs to operators. A scalar or a color; a per-element value, a function of the
  element evaluated for each (`t`, `depth`, `index`), as Blender's fields are, of which a curve
  over `t` is one (C145, replacing Bias); and a draw (`bell`, `uniform`), replacing `Fuzzy`. A
  formula is a value wire written as text.

Two cases a graph must meet:

- **Crowding across nodes**: competition (D26) places a node's limbs against those already on
  the parent, so two scion nodes on one parent would not see each other's. Either a scion node
  takes several ranges, each with its own settings, or competition works on the parent rather
  than inside a node.
- **Whole-plant steps**: `pipe`, prune (C133) and `clusters` on every end need every axis, so
  structure wires join before them, by a join node or by those steps standing at the end.

A structure wire can also be split by its elements' properties. A demultiplexer node fed from a
scion node sorts the limbs by the angle of each one's tip, with an output for each range, and
each output feeds a different foliage operator. So a graph gives a steep tip its look
conditionally, where the species builds one rule into its cards (C70b).

SpeedTree's generation editor is a graph of structure alone: generator nodes each spawning on
its parent, their parameters set on each node, many as curves along the parent. So a graph
comes in three steps, each useful alone: structure only, today's levels and operators as nodes,
which already gives a trunk two scion nodes; per-element inputs (C145's curves); then value wires
and the front panel's controls.

## Not in the language

Rings, vertices and meshing; competition's constants; stream bookkeeping; the order of draws
inside an operator. Loops and user functions: an author repeats a step for a third level, and a
second form is a second file.

## Open

- **Format**: JSON, TOML or RON for the file. JSON matches the glTF extra; TOML reads better by
  hand.
- **Parser**: `nom` with a hand-written grammar over it: formulas are expressions only, so a
  grammar generator such as `peg` earns its keep only if the patch becomes a language of its
  own rather than formulas in JSON or TOML. The evaluator is ours, so evaluation stays
  deterministic, and errors name the step and input. Expression crates (`evalexpr`,
  `fasteval`) were the other option. A new dependency; confirm when step 3 starts.
- **Expressions as strings**, not as JSON trees in MongoDB's style
  (`{ "$add": [ "$height", 2 ] }`): that form suits expressions that programs write and people
  seldom read, and an angle such as `bell(lerp(50°, 80°, droop) - 20° * t, 10°)` would nest four
  objects deep.
- **Editing**: a patch is a Bevy asset, hot-reloaded, edited in a text editor while the preview
  follows; the panel shows the controls only. A developer view listing the steps can come later.
- **Today's species** stays as it is: a patch is a species of its own kind, tagged apart from it
  in the glTF extra, so neither is read as the other.

## Order of work

1. The skeleton, with the panel and species unchanged: golden meshes (C78), one axis type
   (C79), the skeleton meshed after growing (C80), a radius per vertex (C81), operators as
   functions (C82); then, changing output, hinting as each level's last stage (C83) and wander as a
   displacement (C84).
2. Stacks (C96 to C99): the operators on the panel, today's per-level fields split between the frames
   and default operators, so the default species grows as before.
3. Formulas, controls and the patch file; the default patch reproduces step 2's plant.
4. A second form written as a patch (X18's first preset that dials cannot reach).
