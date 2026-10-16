# Patches

A design for forms as data (ledger X22, X18): a plant described by a patch, a file of operators
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
as Merge Points takes a geometry and returns another. A patch is a file, not a node graph:
operators in order, their inputs written as spreadsheet formulas in strings, so the file stays
readable and needs no AST spelled out in JSON.

Formulas never reach the vertices. Placing rings, tracing paths, sharing wood and meshing stay in
Rust, so the language is never a slow, clumsy copy of the generator. A formula is evaluated once
per plant or once per element (an axis, a node), never per ring or vertex.

## Three layers

1. **Controls**: the panel. Each has an id, a caption, a range, a default and units, and becomes a
   row as a species field does now (D9's attributes, read from the patch rather than the type).
2. **Operators**: a pipeline over the abstract plant (below), each input a number or a formula.
3. **Realization**: a Rust library turning the finished abstract plant into bark and cards:
   hinting (D17), the drawn minimum radius (D22), tubes, cards. Its settings are the game's, not
   the form's (the detail threshold, sides per level), so it sits outside the patch.

## The abstract plant

Operators act on a skeleton, and meshes are made from it once they have all run. The skeleton
is a list of axes, an axis in botany's sense: one stem of any order, the trunk, a limb or a
twig, not a direction.

A limb is two things. Its **placement** says where it starts and which way it leaves: on a
parent, at a place along it, an azimuth and an angle; or for a stem, on the ground, at a point and
a direction. Its **composition** says what it is from there: its course, its length, its wander,
its radius, its children. The trunk differs from a limb only in its placement, so every
operator on composition serves both. The code already leans this way: `Stem` and `Limb` are the
same struct but for a level, and `Parent` reads either.

```rust
pub struct Skeleton {
    pub axes: Vec<Axis>,
    pub cards: Vec<Card>,
}

pub struct AxisId(u32);

pub struct Axis {
    pub level: usize,
    /// The parent and the distance along it where this axis leaves; none for a stem.
    pub parent: Option<(AxisId, f32)>,
    pub kind: AxisKind,
    /// Before any draw; each operator forks its own stream from it (D16).
    pub rng: Rng,
    /// Base to tip, at the trace's step (D17), at least two.
    pub vertices: Vec<Vertex>,
    pub scions: Vec<AxisId>,
}

pub enum AxisKind { Stem, Lateral, CarryOn, ForkCopy }

pub struct Vertex {
    pub position: Vec3,
    /// Distance along the axis from its base; the last is its length.
    pub at: f32,
    pub tangent: Vec3,
    /// Square to the tangent, carried along without twisting (`Ring::side`).
    pub side: Vec3,
    pub radius: f32,
}

/// A leaf cluster at a distance along an axis (D27).
pub struct Card {
    pub axis: AxisId,
    pub at: f32,
}
```

- The start is the first vertex and the length the last vertex's `at`, so neither is stored.
  The start frame is the parent's frame at the attachment.
- A radius on every vertex replaces the generator's base and top radii and the stretch kept at
  the base radius: the taper and the pipe model (D22) each write radii.
- The skeleton keeps the full trace; hinting (D17) simplifies it when meshed.
- Whether an axis bears scions, or leaves (`Card`s) at its tip, follows from its `scions`.
- Size does not pass from parent to scion. A scion near its parent's tip is already shorter, its
  reach a share of the distance to the envelope, which narrows toward the top of most profiles,
  and thinner, its radius a share of the tapered parent's there. Where a form wants more, a size
  bias scales reach by place, as angle bias does angle.

Operators come in three kinds:

- **Tracing**: the frame's growth traces an axis from its start, and bend and weep are read while
  it does, since changing the course changes where the envelope stops it. They cannot be laid
  over a finished path.
- **Displacing**: wander and the shear back to the course move vertices, then carry the sides
  along again, which the skeleton provides as one function for every operator that moves
  vertices. Wander clips at the envelope again.
- **Restructuring**: scions add axes, carry-on (D30) and fork (D31) cut an axis and add others.
  These grow the list while walking it, so axes are named by index, not borrowed.

Operators run level by level, since scions are placed on their parent's finished path; the pipe
model alone waits for the whole skeleton.

The skeleton is plain data, not entities: the generator stays a pure function of the species, as
X1's crate without the ECS needs, an edit rebuilds thousands of axes without spawning, and an
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
  skew (D21, D25, D26). Competition stays inside the operator; facing is an input.

Composition:
- `grow`: attachments into axes: length or reach to the envelope, bend toward, bend (D17's
  course), stopping at the envelope.
- `wander`: kinks about each path's course (D17), on straight or bent paths alike.
- `carry_on`: cuts each axis `on` at its last node and grows one more of its children's tag on
  along it (D30).
- `shear_to_course`: moves a path's tip back to where its course ends (C62), stems only.

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

A behaviour is an operator when it has a value that turns it off and most species leave it
there (a fork's chance of 0, no bend, no wander); what every level needs is the frame's, and an
operator's own settings (a foliage card's facing and shape) are its fields. The operators, in
their fixed order:

1. **Angle bias**: the angle changing with the node's place along the parent (C55). **Size
   bias** beside it: the reach changing the same way, for a form whose envelope does not
   shorten its upper scions enough.
2. **Bend**: bend toward and bend (D17's course). Without it the course is straight. On the
   trunk it curves the stem after the tilt it leaves the ground at (the splay, and the lean's
   tilt, X36), which gives it the heading a bend keeps.
3. **Weep**: the course allowed past level and below the envelope, for arching stems (C48) and
   pendulous levels (X19).
4. **Wander**: kinks about the course (D17), with an amplitude (today's wander) and a
   frequency in kinks per crown radius, so a level can kink faster or slower than the fixed
   spacing D17 gives every level now. Its options: the tip held on the course (C62), and kinks
   at the nodes (X23), which needs nodes laid out before the trace and so strains the order.
5. **Fork**: D31's five fields; on the trunk, stem forks (C75).
6. **Scions**: the next level, placed on this one's axes (density, per node, facing, skew,
   start: the placement half of a limb). Not offered on the last of three limb levels (D15).
   Space colonization (X12) would be a second way to place it, a mode of Scions or an operator
   excluding it.
7. **Taper**: the level's own taper in place of the plant's (X32).
8. **Pipe model**: the level's axes narrowed by the wood their children take (D22), in place of
   the taper (C73). It needs every level grown, so it runs after them.

Levels grow in turn: a level's frame and operators, then its scions' level.

- A level's header has an add button (a boxed plus) opening a menu of the operators the level
  lacks; choosing one inserts it in its place, and a ✕ in an operator's header removes it.
- No more than two boxes deep, in feathers' own order: each level a feathers pane, its frame's
  rows in the pane's body and each operator a subpane in it. The pane's header is built for
  tool buttons (the add button, and dividers between groups of them), and its body raises the
  surface for subpanes. The other top-level groups (Plant, Crown, Foliage) become panes too,
  so the panel is one column of panes. Should that prove heavy or repetitive, the fallback is
  C33's lighter headings: each level stays a subpane and each operator a heading in it, the ✕
  at the heading's right.
- Scions replace the − and + on the levels' headers (D15): adding scions adds a level's subpane
  after this one, its placement fields first, as the Arrangement group shows them now. Removing
  scions hides the levels above, which keep their values, as D15's hidden levels do.
- An operator a level lacks shows no rows: a twig level without wander has no wander fields. A
  new behaviour can be a new operator rather than a field on every level, which eases D25.
- Each operator is a field of its level, in the fixed order, so a level has at most one of each:
  a `Toggle<T>`, its settings and whether it is on. The add menu turns it on and ✕ off, so an
  operator removed and added again keeps its values for the session, as a hidden level does
  (D15); files write only operators that are on. Each level's defaults hold every operator's
  settings, off, varying by depth as the levels' defaults do. The generator calls an operator's
  function (C82) where it is on. The property editor gains a group shown while its `Toggle`
  is on and listed in the add menu while off. C10 serializes them; so settle this before C10.
- Levels stay D15's array of three and its count of those shown: a level's Scions are on while
  the count reaches past it, so paths stay `limbs[1].angle` and only D15's panel changes.

The crown is fixed at the head and the foliage at the foot, both the plant's, not a level's:
cards on every branch end, whatever its level (D27). More than one leaf texture serves two
ends: leaves on interior stems, inside the crown (X30), and variety among the tips' leaves, each
tip drawing one of several sprigs. Radiating cards (C54) are a kind of the foliage's too. Each
texture is a material in Panoply, which would split its merged flora meshes, so the leaf
textures are regions of one texture (requirements, Foliage), a constraint on C38's painter.

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

`t` is what X22 once called a curve over the parent's length: angle by place (C55) is
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
    { "carry_on": { "on": "trunk" } },
    { "pipe": { "exponent": 2.5, "share": "if(depth == 0, 1, 0.5)" } },
    { "clusters": { "on": "ends" } }
  ]
}
```

Each step is an object with one key, the operator, as a MongoDB pipeline stage is, and as
`bevy_reflect` and serde write an enum by default.

A species is then a patch, the controls' values and a seed: the glTF extra (C10) holds those, and
the patch's name and version (X15).

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
- **Today's species**: becomes the default patch, its controls one to one with today's fields,
  so the panel and plans are unchanged until a second form exists.

## Order of work

1. The skeleton, with the panel and species unchanged: golden meshes (C78), one axis type
   (C79), the skeleton meshed after growing (C80), a radius per vertex (C81), operators as
   functions (C82); then, changing output, hinting at meshing (C83) and wander as a
   displacement (C84). Useful alone: it is most of X1.
2. Stacks (X31): the operators on the panel, today's per-level fields split between the frames
   and default operators, so the default species grows as before.
3. Formulas, controls and the patch file; the default patch reproduces step 2's plant.
4. A second form written as a patch (X18's first preset that dials cannot reach).
