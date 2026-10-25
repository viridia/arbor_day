# Curve editor: design

How the curve editor in [curve-editor.md](curve-editor.md) is built. The survey behind it is
[curve-editor-prior-art.md](curve-editor-prior-art.md); the mock is
`https://claude.ai/artifact/JhDqLFzUGJo2neenUGNrQN`. Requirements are cited as `CE<n>`.

The code is one module of arbor_day, `src/curve/`, in five files, each a layer that depends only on
those above it:

| File | Holds | Depends on |
| --- | --- | --- |
| `model.rs` | The curve, its constraints, sampling, applying an edit | `bevy_math`, `bevy_curve`, `bevy_reflect` |
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
handle's `Vec2` is its offset from its point, in the curve's units. `Auto` stores no offset, so
a Bias curve serializes as its points' x, y and segment, a short list (CE8). Points are ordered by
x; two may share an x, making a step (CE1).

### 1.1 The constraints

The consumer's declaration of what its curve may be (CE2 to CE5):

```rust
pub struct CurveConstraints {
    pub x: CurveAxis,
    pub y: CurveAxis,
    pub segments: EnumSet<CurveSegment>,
    pub handles: EnumSet<HandleMode>,
    pub points: RangeInclusive<usize>,
    pub pin_ends: PinEnds,          // None, X, or XY
    pub extrapolation: EnumSet<CurveExtrapolation>,
}

pub struct CurveAxis {
    pub min: f32,
    pub max: f32,
    pub clamped: bool,              // open bounds bind the view, not the points
    pub step: f32,                  // snapping, arrow keys, grid
}
```

Units and precision are the feathers editor's business: the axis carries numbers only, and the
host gives the feathers editor the number-input units it already uses for the field.

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
segment, with a hand-set handle, finds t for x by Newton's method, bracketed by bisection, a fixed
number of iterations. Both use only addition, multiplication, division and `sqrt`, so a curve and
an x give the same result on every platform.

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
(CE24), within the pins; refuse an insert past `points.end()` or a delete below `points.start()`,
and a kind or mode the constraints leave out; place an insert on the curve at the pointer's x (CE15,
survey finding 4). It returns whether the curve changed, and for an insert, the new point's index.
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
    pub op: CurveEditOp,
    pub phase: Phase,               // Start, Update, Finish, Cancel; one-off edits are Finish
}

pub enum CurveEditOp {
    Move(Vec<(PointRef, Vec2)>),                    // proposed positions
    MoveHandle(PointRef, HandleSide, Vec2),         // proposed offset
    SetPoint(PointRef, Vec2),                       // typed into an input
    Insert(CurveRef, Vec2),                         // the pointer's position
    Delete(Vec<PointRef>),
    SetSegment(PointRef, CurveSegment),
    SetHandleMode(PointRef, HandleSide, HandleMode),
    Replace(CurveRef, ScalarCurve),                 // preset, reset, paste
}

pub struct PointRef { pub curve: CurveRef, pub index: u32 }
```

**A move proposes positions, not a delta.** At a drag's start the editor snapshots the selected
points; each `Update` proposes snapshot plus the pointer's total offset, as the slider applies a
drag's total `distance` to the value it saved at the start. So a point the consumer clamps at a
bound follows the pointer back (CE12), and the consumer keeps no drag state. `Cancel`, from Escape
or from the editor being despawned mid-drag, proposes the snapshot itself (CE13). The consumer
receives every `Update` (CE14) and regrows the tree from each.

**The curve is named in every reference.** `CurveRef` is the entity holding the curve's
`CurveValue`. With one curve it is the editor itself; several curves (CE35) are child entities,
each with its own value and constraints, and a selection or a move spans them by naming each point's
curve. Nothing in the single-curve design changes when the second curve arrives.

### 2.1 Selection

The editor holds it in `CurveSelection { points: Vec<PointRef> }` (CE16). Handles are never
selected: one is grabbed for a drag and let go, so the selection is points alone. The host changes it by triggering `SelectCurve::All`, `None` or `Points(..)` on the editor,
and the editor triggers `CurveSelectionChanged` whenever it changes, for a host keeping the
selection with its undo steps (CE17).

Selection is indices, adjusted as text editors adjust a cursor (CE7). The editor remembers the
insert or delete it last sent; when the consumer writes `CurveValue` with one point more or fewer
in that place, the editor shifts the selection past it, and selects an inserted point. Any other
change of point count is a replacement: the selection is trimmed to the points that remain.

## 3. The view

`CurveView` maps between the curve's units and the plot node's layout pixels. Today it is the
axes' bounds stretched to the node; zoom and pan (CE36) would make it a transform of its own,
within the bounds, and nothing else would change, since gestures and edits are in the curve's
units.

From the prepared curve and the view, `view.rs` keeps two components up to date on the plot node,
for whatever draws them:

- **`CurvePath`**: the curve as a polyline in layout pixels. Each segment is halved, over its own
  span, until each piece lies within a quarter pixel of the curve, so a kink stays a point and a
  step two points at one x (CE40).
- **`CurveGrid`**: the grid lines' pixel positions and values, from each axis's bounds and step,
  each marked major or minor, the ends and middle marked for labels (CE27).

## 4. Gestures

Observers on the plot node turn pointer and key events into hover, selection and edits. The
interaction in progress is one value, as in `bevy_reactor`'s node graph (CE23):

```rust
pub enum CurveGesture {
    Idle,
    MovePoints { snapshot: Vec<(PointRef, Vec2)>, anchor: Vec2 },
    MoveHandle { point: PointRef, side: HandleSide, snapshot: Vec2, anchor: Vec2 },
    SelectRect { anchor: Vec2, rect: Rect },
}
```

- **Hit testing** is geometric: the nearest point or handle within `CurveHitRadius` pixels, else
  the curve within the same radius, else empty space. The result is published as `CurveHover`,
  for styling and for the next press (CE22).
- **Handles** are shown, and hit, only on selected points, and only where set by hand (CE37).
- **Press** on a point selects it (with a modifier, adds it), and a drag from there moves the
  selection, handles and all; on a handle, moves that one handle and leaves the selection as it
  is; on empty space, starts a rectangle, which takes points only (CE24, CE30).
  The editor captures the pointer for the drag, as the slider does.
- **Double-click** inserts: a `PointerPress` whose `count` is 2.
- **Keys**, while the plot has focus: Delete and Backspace delete the selection; arrows move it by
  the axis step, a modifier for ten steps; Tab moves to the next point; Escape cancels a drag
  (CE28).

## 5. The feathers editor

`FeathersCurveEditor` is a scene component, as `FeathersSlider` is, requiring the headless parts.
It assembles, top to bottom:

- **A header row**: the host's caption and disclosure toggle, and a menu button with the
  consumer's presets, Reset, and Copy and Paste curve (CE29, CE33). Feathers has no submenus, so
  the presets are items of their own.
- **The plot**: a node with `CurvePath` drawn by the path material, the grid as a second instance
  of it, labels as text nodes at `CurveGrid`'s marked lines, and a node per point and per visible
  handle, placed from the view and styled from `CurveHover` and `CurveSelection` (CE41). Nodes keep
  the points to feathers' own styling and focus rings; the shader draws only lines.
- **An input row**: feathers number inputs for the selected point's x and y, disabled unless
  exactly one point is selected (CE25); and below them, a row each for that point's hand-set
  handles, In and Out, their x and y (CE34). An input sends `SetPoint` or `MoveHandle`, `Update` while scrubbing, `Finish` on
  commit.

`FeathersCurveThumbnail` is the plot alone, small, with no gestures: what a collapsed row shows
(CE26).

### 5.1 The path material

`plot.rs` holds `bevy_reactor`'s `DrawPathMaterial`, moved here (the ledger's note under C145),
with two changes: points in the node's own coordinates rather than relative to the path's bounds,
and the quadratic's division by zero guarded for the node graph's straight wires. The curve's path
is lines only. Its fragment shader tests every segment; a few hundred segments over a plot of
300 × 170 pixels is cheap. It reads a storage buffer, so the plot is WebGPU only, and says so
(CE43).

## 6. arbor_day

- **The fields.** `Bias::size` becomes a `ScalarCurve`, x the place along the parent, y the share
  of the reach. Today's `1 + size` at the first node and `1 − size` at the last convert to two
  points joined by a cubic with auto handles, which with two points is that same straight line, so
  every tree grows as before (CE8). `bias_size` samples the curve at the node's place; `bias_leaves`
  divides by the curve's greatest value over the nodes instead of `1 + |size|`. A script converts
  `reference/`. Leaves and Aim follow as their own chunk.
- **The constraints** come from the field's reflect attributes, as a number row's look does: a
  `@CurveAxes` attribute for x, `@ValueRange` for y, and the presets as a list of curves.
- **The row** is a disclosure: collapsed, the caption and a thumbnail on one line; expanded, the
  feathers editor below it (the mock's first frame). Which rows are open lives in the panel's UI
  state.
- **The edits** reach the row as a color input's do: an observer stores each `CurveEdit` on the
  editor, and the immediate-mode row takes them each frame, applies them to the species field with
  `ScalarCurve::apply`, and writes `CurveValue` when the field differs. The species changing
  regrows the tree.

## 7. Testing

- **Unit tests** (`cargo test`): sampling (Fritsch–Carlson never overshooting, steps, the
  extrapolations, the same result for a weighted and an unweighted form of one curve);
  `apply`'s clamps and refusals; selection adjusting across insert and delete; and the flattener,
  every piece within 0.25 px of the curve, a kink one point, a step two points at one x.
- **A step in `plans/widgets.py`**, no new plan: expand a Bias curve, drag a point and read the
  inputs back; then set, with `driver.set`, a curve holding a step, a kink and a steep cubic, and
  screenshot it, judged by eye for facets and even width.

## 8. Checked against what is kept possible

- **Several curves (CE35)**: every reference names its curve (2), the view maps x for all and y
  per curve, and normalizing is a per-curve y range in the view.
- **Zoom and pan (CE36)**: the view is already the only place pixels meet curve units (3).
- **The crown (opportunistic)**: a width over height with rounded ends is cubic segments whose end
  handles are free and vertical (1.4 permits an x offset of zero), and the five profiles become
  presets. Nothing in the design was added for it.
