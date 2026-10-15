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

What replaces `Species` as the thing operated on is a skeleton: axes, each a path with a radius
and some attributes, and the attachments that place them.

A limb is two things. Its **placement** says where it starts and which way it leaves: on a
parent, at a place along it, an azimuth and an angle; or for a stem, on the ground, at a point and
a direction. Its **composition** says what it is from there: its course, its length, its wander,
its radius, its children. The trunk differs from a limb only in its placement, so every
operator on composition serves both. The code already leans this way: `Stem` and `Limb` are the
same struct but for a level, and `Parent` reads either.

- **Attachment**: a place to grow from. A parent axis and a place along it (or the ground), a
  direction, a tag, a random stream. Made by placement operators; consumed by `grow`.
- **Axis**: a path of points with frames (today's `Ring`s), base and top radius, the tag of the
  attachment it grew from, its depth, its random stream, and named attributes later operators
  read (a radius share, sides). Its children are attachments on it.
- **Crown**: the envelope (D11), a field operators read: reach to it, clip to it, the part of a
  parent inside it.
- **Cards**: leaf clusters placed on axes.

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

A stage before formulas, after Blender's modifier stack: the operators shown as panels, their
inputs plain values, with no controls layer. The panel is the pipeline.

Each level (the trunk, then each limb level) has a fixed frame and a stack. The frame is
placement and growth, which every level needs: for the trunk its stems and height, for a limb
level its nodes and `grow`'s fields. The stack holds the operators between growing a level's axes
and placing the next level's nodes on them (`wander`, `shear_to_course`, and later angle by place
and forks), so any order the panel allows is a valid one. Levels grow in turn: frame, stack, then
the next level up.

- An "Add" select at the foot of a level's stack inserts an operator, its panel a subpane like a
  level's. Its header has ▲ and ▼ to reorder, ✕ to remove, and an enable toggle, which keeps the
  operator's values while it is off, as a hidden level keeps its own (D15). Drag and drop can
  replace the arrows later.
- An operator a level does not use shows no rows: a twig level without wander has no wander
  field. A new behaviour can be a new operator rather than a field on every level, which eases
  D25.
- The stack is a list of enums with data. The property editor gains a widget for one (D13's
  select chooses the variant), and C10 serializes it; so settle stacks before C10.

Some operators need the whole skeleton and run after every level has grown, whatever their place
in a stack, which then says which levels they apply to rather than when they run. The pipe model
is one: in a level's stack it narrows that level's axes by the wood their children take (D22),
top-down, in place of their own taper (C73). Levels without it keep the taper.

The crown is fixed at the head. Foliage is a module like the rest, in the stack of the level that
bears it, on branch ends by default; a second module with its own leaf texture gives interior
leaves (X30). Each texture is a material in Panoply, which would split its merged flora meshes, so
the modules paint regions of one texture (requirements, Foliage), a constraint on C38's painter.

Formulas then come as a later stage: any input may become a formula, and the stacks become the
patch's steps, with controls added above them.

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

Each step is an object with one key, the operator, as a MongoDB pipeline stage is. This is how
`bevy_reflect` and serde write an enum by default, so a stack saved by reflection (C10) and a
patch's steps take the same form.

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

1. The generator split into operators over the abstract plant and a realization library, in Rust,
   driven by today's `Species` through a fixed pipeline. No language yet; golden tests (X15)
   hold the output, apart from the fused loop's tips. Useful alone: it is most of X1.
2. Stacks (X31): the operators as panels, today's per-level fields split between the frames and
   default stacks, so the default species grows as before.
3. Formulas, controls and the patch file; the default patch reproduces step 2's plant.
4. A second form written as a patch (X18's first preset that dials cannot reach).
