# Curve editor: design

How the curve editor in [curve-editor.md](curve-editor.md) is built. The survey behind it is
[curve-editor-prior-art.md](curve-editor-prior-art.md); the mock is
`https://claude.ai/artifact/JhDqLFzUGJo2neenUGNrQN`. Requirements are cited as `CE<n>`.

The code is one module of arbor_day, `src/curve/`, in five files, each a layer that depends only on
those above it:

| File | Holds | Depends on |
| --- | --- | --- |
| `model.rs` | The curve, its constraints, sampling, applying an edit; `CurveValue` | `bevy_math`, `bevy_curve`, `bevy_reflect`, `bevy_ecs` |
| `edit.rs` | The events between editor, consumer and host | `model.rs`, `bevy_ecs` |
| `view.rs` | The mapping, the curve's path in pixels, grid lines | `model.rs`, `bevy_ui` |
| `gesture.rs` | Pointer and key handling, hover, selection | `edit.rs`, `view.rs`, `bevy_picking` |
| `feathers.rs`, `plot.rs` | The feathers editor and the path material | all of the above, `bevy_feathers` |

The first four are the headless parts (CE21); none depends on arbor_day or `bevy_immediate`
(CE20).

## 1. The curve

```rust
pub struct ScalarCurve {
    pub points: Vec<CurvePoint>,
    pub extrapolation: CurveExtrapolation,
}

pub struct CurvePoint {
    pub x: f32,
    pub y: f32,
    /// The segment from this point to the next; ignored on the last point.
    pub segment: CurveSegment,
    pub left: DirectionHandle,
    pub right: DirectionHandle,
}

pub enum CurveSegment { Constant, Linear, Cubic }
pub enum DirectionHandle { Auto, Aligned(Vec2), Free(Vec2) }
pub enum CurveExtrapolation { Hold, Continue, Repeat }
```

Each cubic segment leaves and enters its points along a **direction handle**, Adobe's name for
the line from a point to the control point that sets the curve's direction and pull there. A
handle's `Vec2` is its offset from its point, in the curve's units. `Aligned` and `Free` store
their offset; `Auto` computes it (1.3). Points are ordered by x; two may share an x, making a step
(CE1). The consumer stores the curve in a form of its own and translates (CE8): this is the
editor's form, not a file format.

### 1.1 The constraints

The consumer's declaration of what its curve may be (CE2 to CE5):

```rust
pub struct CurveConstraints {
    pub x: CurveAxis,
    pub y: CurveAxis,
    pub segments: Vec<CurveSegment>,
    pub handles: Vec<HandleMode>,
    pub points: RangeInclusive<usize>,
    pub pin_ends: PinEnds,          // None, X, or XY
    pub extrapolation: Vec<CurveExtrapolation>,
}

pub struct CurveAxis {
    pub min: f32,
    pub max: f32,
    pub clamped: bool,              // open bounds bind the view, not the points
    pub step: f32,                  // snapping, arrow keys, grid
    pub precision: f32,             // where a dragged point lands, what the inputs show; 0 for any
}
```

The sets are short lists, read with `contains`, so the model takes no set crate. Units are the
feathers editor's business: the axis carries numbers only, its precision too, in the curve's own
units, and the host gives the feathers editor the number-input units it already uses for the
field. The step is coarse, for snapping on request; the precision is the finest a value lands on.

### 1.2 Preparing and sampling

A `ScalarCurve` is not sampled directly. `ScalarCurve::prepare` resolves it into a
`PreparedCurve`, one cubic Bézier per segment, its four control points in the curve's units:

- **Constant**: all four at the first point's y; the next point starts the next segment, so the
  curve steps there.
- **Linear**: controls at a third and two thirds of the way along the line.
- **Cubic**: controls at each point plus its handle, auto handles computed first (1.3).

`PreparedCurve` implements `bevy_curve`'s `Curve<f32>` over the x span, with its extrapolation applied
past the ends, and gives value and slope (CE6). A segment whose control points' x lie at thirds
of its span is marked unweighted: its y is a cubic polynomial in x, evaluated directly. A weighted
segment, with an aligned or free handle, finds t for x by Newton's method, bracketed by bisection, a fixed
number of iterations. Both use only addition, multiplication, division and `sqrt`, so a curve and
an x give the same result on every platform. A cubic is evaluated in powers of t from differences
of its controls, so equal controls give exactly their value: a flat Size scales by exactly 1.

The prepared form is the curve's single encoding. The GPU takes the same array of control points
(the translation layer of the ledger's note), so a shader needs one evaluator, or two if it skips
the weighted solve; the consumer's constraints simply leave out what its shader lacks.

### 1.3 Auto handles

Auto handles follow Fritsch and Carlson's monotone cubic interpolation: each point's slope starts
as the mean of its two secants, is zero where the curve turns (a secant changes sign or is zero),
and is limited to three times either secant. The result never overshoots its points (CE3), a
stronger promise than Blender's Auto Clamped makes (survey, finding 1). An end point takes the
one-sided three-point slope, limited the same way. An auto handle's x length is a third of its
segment's span, so an all-auto curve is unweighted.

### 1.4 Keeping the curve a function

A cubic segment's x is monotone in t whenever both its inner control points' x lie within the
segment's span (the property that keeps CSS's `cubic-bezier` a function). So a right handle's x
offset is kept within 0 and the span to the next point, and a left handle's within the span to the
previous point and 0. Applying an edit clamps to this; `prepare` clamps again, so a curve read
from a file cannot break it.

### 1.5 Applying an edit

`ScalarCurve::apply(&mut self, op: &CurveEditOp, constraints: &CurveConstraints) -> Applied` is the standard
consumer's judgement (CE10): clamp each point to the clamped axes, between its neighbours' x
(CE24), within the pins, and an aligned or free handle's control point within a clamped y axis,
which keeps the curve within it, since a segment lies within its control points' hull; refuse an insert past `points.end()` or a delete below `points.start()`,
and a kind or mode the constraints leave out; place an insert on the curve at the pointer's x (CE15,
survey finding 4), each coordinate rounded to its axis's precision, as a drag rounds. It returns whether the curve changed, and for an insert, the new point's index.
A consumer with other rules writes its own; arbor_day uses this one.

## 2. Edits and events

The editor holds a copy of each curve it shows, as a component, `CurveValue(ScalarCurve)`, beside
its `CurveConstraints`. It never changes that copy: it sends edits, and the consumer writes the copy back
once it has judged them (CE10), as a slider's host writes `SliderValue`. A `curve_self_update`
observer, like `slider_self_update`, applies edits to the copy for a host that wants nothing more.

```rust
#[derive(EntityEvent)]
pub struct CurveEdit {
    #[event_target]
    pub editor: Entity,
    pub curve: Entity,
    pub op: CurveEditOp,
    pub phase: CurveEditPhase,      // Start, Update, Finish, Cancel; one-off edits are Finish
}

// In model.rs: an edit to one curve, its points by index.
pub enum CurveEditOp {
    Move(Vec<(usize, Vec2)>),                       // proposed positions
    MoveHandle(usize, HandleSide, Vec2),            // proposed offset
    SetPoint(usize, Vec2),                          // typed into an input
    Insert(Vec2),                                   // the pointer's position
    Delete(Vec<usize>),
    SetSegment(usize, CurveSegment),
    SetHandleMode(usize, HandleSide, HandleMode),
    Replace(ScalarCurve),                           // preset, reset, paste
}

pub struct PointRef { pub curve: Entity, pub index: usize }
```

**A move proposes positions, not a delta.** At a drag's start the editor snapshots the selected
points; each `Update` proposes snapshot plus the pointer's total offset, as the slider applies a
drag's total `distance` to the value it saved at the start. So a point the consumer clamps at a
bound follows the pointer back (CE12), and the consumer keeps no drag state. `Cancel`, from Escape
or from the editor being despawned mid-drag, proposes the snapshot itself (CE13). The consumer
receives every `Update` (CE14) and regrows the tree from each.

**The curve is named in every reference**, as the entity holding the curve's `CurveValue`. With one curve it is the editor itself; several curves (CE35) are child entities,
each with its own value and constraints. A selection spans them by naming each point's curve
(`PointRef`), and a move across curves is one event per curve, so `apply` and the op stay
within one curve. Nothing in the single-curve design changes when the second curve arrives.

### 2.1 Selection

The editor holds it in `CurveSelection { points: Vec<PointRef> }` (CE16). Handles are never
selected: one is grabbed for a drag and let go, so the selection is points alone. The host changes it by triggering `SelectCurve` on the editor with `SelectedPoints::All`, `None`
or `Given(..)`, and the editor triggers `CurveSelectionChanged` at the end of any frame the
selection changed in, except its first, for a host keeping the selection with its undo steps
(CE17).

Selection is indices, adjusted as text editors adjust a cursor (CE7). The editor remembers the
curve as it last saw it and the insert or delete it last sent (`SeenCurve`); when the consumer
writes `CurveValue` with one point more, the new point is the first that differs and is selected,
and with the deleted points gone, the rest shift down past them. Any other change of point count
is a replacement: the selection is trimmed to the points that remain.

## 3. The view

`CurveView` maps between the curve's units and the plot node's layout pixels. Today it is the
axes' bounds stretched to the node; zoom and pan (CE36) would make it a transform of its own,
within the bounds, and nothing else would change, since gestures and edits are in the curve's
units.

From the prepared curve and the view, `view.rs` keeps **`CurvePath`** up to date on the plot node
(`CurvePlot`), a descendant of the entity holding the curve's `CurveValue` and constraints: the
curve as a polyline in layout pixels. Each segment is halved, over its own
  parameter, until each piece lies within a quarter pixel of the curve, so a kink stays a point and
  a step two points at one x (CE40). A path holds at most `MAX_PATH_POINTS` (257), the path
material's uniform array; a curve needing more is flattened again at twice the tolerance until
it fits.

The grid is fixed: a line at each quarter of each axis, the ends and middle labelled (CE27), as
the mock draws it. It needs no headless part: the feathers editor places lines and labels by
percentages of an inset node.

## 4. Gestures

Observers on the plot node turn pointer and key events into hover, selection and edits. The
interaction in progress is one value, as in `bevy_reactor`'s node graph (CE23):

```rust
pub enum CurveGesture {
    Idle,
    MovePoints { curve: Entity, snapshot: Vec<(usize, Vec2)>, offset: Vec2, axes: [CurveAxis; 2] },
    MoveHandle { curve: Entity, index: usize, side: HandleSide, snapshot: Vec2, offset: Vec2, axes: [CurveAxis; 2] },
    SelectRect { curve: Entity, from: Vec2, to: Vec2, base: Vec<PointRef> },
}
```

They are on the plot node, `CurveGestures`, beside `CurveHover` and `CurveHitRadius`.

- **Hit testing** is geometric: the nearest point or handle within `CurveHitRadius` pixels, a
  point winning a tie. The result is published as `CurveHover`, a `CurveTarget`, for styling
  (CE22); a press tests afresh.
- **Handles** are shown, and hit, only on selected points, and only where aligned or free and
  shaping a cubic segment (`ScalarCurve::manual_handle`, CE37). A handle's move proposes its
  offset, snapshot plus the pointer's, rounded as a point's position is.
- **Press** on a point selects it (with a modifier, adds it), and a drag from there moves the
  selection, handles and all; on a handle, moves that one handle and leaves the selection as it
  is; on empty space, starts a rectangle, which takes points only (CE24, CE30). Shift, Ctrl and
  Cmd are the modifier. Each proposed position, snapshot plus offset, is rounded to its axes'
  precision, as a slider rounds a drag, so what a point stores is what its inputs show; until the
  pointer moves, points stay where they were. The move starts at the press, so a press without a
  drag sends `Start` and
  `Finish` with the points where they were; it ends at the drag's end or the release, whichever
  comes first. The editor captures the pointer for it, as the slider does.
- **A rectangle** runs from the press to the pointer, in the curve's units, so a view that zooms
  leaves it alone. The selection is the points inside it, after those selected at the press
  (`base`) when adding, updated as it is dragged. The feathers editor draws it in the selection's
  color, outlined and faintly filled.
- **The plot despawned mid-drag** sends `Cancel` with the snapshot, if the editor is still there.
- **Double-click**, a `PointerPress` whose `count` is even, so two quick double-clicks are two,
  inserts a point on empty space and
  deletes one on a point; the consumer refuses a pinned end's delete.
- **Keys**, while the plot has focus: Delete and Backspace delete the selection; arrows move it by
  the axis step, Shift for ten steps, rounding only the coordinate moved; Tab selects the next
  point and Shift+Tab the previous, as a radio group moves its focus, and past either end lets
  focus leave the plot; Escape cancels a move, proposing its snapshot, or a rectangle, restoring
  `base` (CE28). Escape leaves the pointer captured until its release.

## 5. The feathers editor

`FeathersCurveEditor` is a scene component, as feathers' widgets are; the host inserts the curve
and its constraints beside it. Its fields, kept on the component, say how values are written:
each axis's number-input units, and labels for x in place of its values; by default, plain
numbers. Its props say which inputs are spawned: `@point_inputs`, their captions, `None` for a
curve whose points are never typed. It assembles, top to bottom:

- **A header row**: the host's caption and disclosure toggle, and `curve_menu(items)`, a ⋯ menu
  the host places at the row's right end and aims at the editor with `CurveMenuTarget` (CE29,
  CE33). The host's items (presets, Reset) come first, above a divider; feathers has no
  submenus, so each preset is an item of its own. An item holding `CurveMenuReplace(ScalarCurve)`
  sends a `Replace` when picked, so the host needs no handler; it observes its other items
  itself. Then the editor's own Copy curve and Paste curve, through an app-wide clipboard
  rather than the system's, so Paste is disabled while nothing is copied.
- **The plot**: a node with `CurvePath` drawn by the path material, over the grid's lines as
  one-pixel nodes, the y labels in a gutter to its left and the x labels under it, and a node per
  point and per visible handle, placed from the view and styled from `CurveHover` and
  `CurveSelection` (CE41). Nodes keep the points to feathers' own styling. The path material
  draws the curve, and each selected point's stem, from its In handle through the point to its
  Out handle, as a node of its own with a `CurvePath`, under the markers.
- **An input row**: feathers number inputs for the selected point's x and y, disabled unless
  exactly one point is selected (CE25); and below them, a row each for that point's shown
  handles, In and Out, their offsets from the point as stored, captioned `In Δx` and so on
  (CE34), so they hold still while the point moves. An input sends `SetPoint` or `MoveHandle`,
  `Update` while scrubbing, `Finish` on commit. Each rounds to its axis's precision, its decimal
  places worked out through its units (0.01 shown as a percentage is 1%: no decimals).

`FeathersCurveThumbnail` is the plot alone, small, with no gestures: 48 × 24 at the row's right
end, the open plot's 2:1, so the curve keeps its slopes; the editor's background, stroke and grid,
its lines inside the ends, so closed and open read as one widget and a flat curve reads as a
graph. No border, as feathers' color plane and wheel have none. It is what a collapsed row shows
(CE26).

### 5.1 The path material

`plot.rs` holds `PathMaterial`, after `bevy_reactor`'s `DrawPathMaterial`, stroking lines only, in
the node's own coordinates, its color a theme token (`CurveStroke`). The lines are a uniform array
of 256 rather than a storage buffer, so the plot draws on WebGL2 as well as WebGPU (CE43); 4 KiB,
within WebGL2's 16 KiB uniform block. Its fragment shader tests every line; a few hundred over a
plot of 300 × 170 pixels is cheap.

## 6. arbor_day

- **The fields.** `Bias::size` is a `PlaceCurve`, arbor_day's form for a number along a parent:
  `[place, value]` pairs, written as such, x the place from the first node to the last, y the
  share of the reach. It translates to a `ScalarCurve` of cubic, auto segments for the editor and
  the generator, and back from the editor's edits, the exemplar of a consumer storing only what
  its constraints allow (CE8). Today's `1 + size` at the first node and `1 − size` at the last
  convert to two points, which with auto handles is that same straight line, so every tree grows as
  before; `scripts/size_curve.py` converts `reference/`. `bias_sizes` samples the curve at each
  node's place; `bias_leaves` divides by the greatest over the nodes instead of `1 + |size|`.
  Leaves and Aim follow as their own chunk.
- **The constraints** are the form's (`PlaceCurve::constraints`): x from 0 to 1 to hundredths,
  the ends pinned, cubic segments, auto handles, holding past the ends; y from the field's
  `@ValueRange`, its step from `@Step`, its precision from `@Precision` in the field's units (1%
  for Bias). An attribute for x waits for a curve over another domain (X63). The menu offers
  Reset alone, to the field's value in its struct's `Default`, read by reflection, so a curve
  field whose struct reflects none has no Reset; presets wait for the crown profile.
- **The row** is a disclosure: collapsed, the toggle, caption and a thumbnail on one line;
  expanded, the feathers editor below it (the mock's first frame), its inputs captioned Position and
  the field's caption, x labelled Base, Middle and Tip. Whether a row is open is its toggle's
  `Checked`, so it lasts as long as the row.
- **The edits** reach the row as a color input's do: an observer stores each `CurveEdit` on the
  editor, and the immediate-mode row takes them each frame, applies them to the species field with
  `ScalarCurve::apply`, and writes `CurveValue` when the field differs. The species changing
  regrows the tree.

## 7. Testing

- **Unit tests** (`cargo test`): sampling (Fritsch–Carlson never overshooting, steps, the
  extrapolations, the same result for a weighted and an unweighted form of one curve);
  `apply`'s clamps and refusals; selection adjusting across insert and delete; and the flattener,
  every piece within 0.25 px of the curve, a kink one point, a step two points at one x.
- **A step in `plans/widgets.py`**, no new plan: expand a Bias curve, select a point by clicking
  its marker and type its value, insert with two clicks and delete with Delete; then set a curve
  holding a step, a kink and a steep cubic, and screenshot it, judged by eye for facets and even
  width. The driver cannot drag, so a drag is checked by hand.
- **`examples/curve_editor.rs`**, one row per configuration the editor supports: an editor
  beside its curve as its app stores it, rewritten on each edit, since what an app stores is
  its own choice (CE8). Rows: implicit slopes stored as `[x, y]` pairs; one cubic with aligned
  handles and its ends pinned, stored as CSS's `cubic-bezier(x1, y1, x2, y2)`; mixed segments
  come with C159c, several curves with X66. `curve` is arbor_day's library, for
  the example's sake. `plans/curve_editor.py` types into each row and reads its stored form.

## 8. Checked against what is kept possible

- **Several curves (CE35)**: every reference names its curve (2), the view maps x for all and y
  per curve, and normalizing is a per-curve y range in the view.
- **Zoom and pan (CE36)**: the view is already the only place pixels meet curve units (3).
- **The crown (opportunistic)**: a width over height with rounded ends is cubic segments whose end
  handles are free and vertical (1.4 permits an x offset of zero), and the five profiles become
  presets. Nothing in the design was added for it.
