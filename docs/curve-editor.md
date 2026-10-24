# Curve editor: requirements

A curve here is a scalar function of one variable, y = f(x), shaped by hand from a few points.
The first user is arbor_day, a generator of low-poly trees with a Bevy app for authoring them,
where a curve replaces a number that today changes in a straight line. It is meant for other Bevy
editors too, and for `bevy_ui_widgets` and feathers once it settles, so it depends on Bevy alone.

Two parties share a curve: the **consumer**, the code that owns the curve's data and samples it
(in arbor_day, a field of a tree's species and the generator that reads it), and the **editor**,
the widget that shows the curve and
sends edits to the consumer. The **host** is whatever places the editor: a popup from a property
row, a panel.

Priority: **M** must, **S** should, **C** could. An **opportunistic** use is met if the design
allows it cheaply, approximately if need be, and never shapes the design. A **kept possible**
requirement is not built, but the design must not rule it out, and is checked against it.
Requirements are numbered `CE<n>`; a number is never reused.

## Uses

| Use | x | y | Needs |
| --- | --- | --- | --- |
| arbor_day's Bias, first: how a limb's side branches change from its base to its tip, in length, leaf size and aim | Place along the limb, 0 to 1 | The field's value, in its units and range | A few points; never overshoots them; a popup from a row in the property panel; the tree regrown live while dragging |
| arbor_day's crown profile, opportunistic: the outline of the tree's crown | Height, crown base to top | Width, as a share of the crown's radius | Round ends, a point, a column, approximately; today's five fixed shapes (cone, ellipsoid and so on) as presets |
| CSS-style easing | Normalized time, ends fixed at (0, 0) and (1, 1) | Progress, which may overshoot | One cubic Bézier, its two handles' x within 0 to 1; steps |
| Particle scalars | Normalized age, or emitter time looped | Size, alpha, speed, emission rate | Linear only and at most a few points when sampled on the GPU; held or repeated past the ends |
| Animation tracks (kept possible) | Seconds, an open span | One channel: an axis of translation or scale, an Euler angle, a quaternion component | Interpolation and tangent mode per point; several channels in one view; zoom, pan, frame snapping, a playhead |

## Requirements

### The curve

- **CE1 M** Points joined by segments, x and y `f32`, x never decreasing. Two points may share an
  x, which makes a step; three may not.
- **CE2 M** The consumer describes each axis: its bounds, each clamped or open; its units and
  precision; its snap step. Every point lies within clamped bounds.
- **CE3 M** The consumer declares the segment kinds a curve may use, and the editor offers only
  those. The kinds: constant; linear; a shape-preserving cubic, which never overshoots its points;
  a cubic Bézier with a handle on each side of a point, which can leave a point vertically and
  covers CSS's `cubic-bezier`.
- **CE4 M** The consumer sets the fewest and the most points a curve holds, and may fix the end
  points' x, or x and y.
- **CE5 M** Past its ends a curve holds its end values or repeats; the consumer declares which.
- **CE6 M** Sampled as a `bevy_curve` `Curve<f32>`, value and slope, anywhere in the span. The same
  curve and x give the same result on every platform.
- **CE7 M** Each point keeps one identity across edits, so a selection and a drag survive points
  added or removed elsewhere.
- **CE8 M** Serialized by reflection, a few points as a short list. A field that was a single
  number converts to a curve with the same result, so saved files keep their meaning.
- **CE9 M** Each consumer sets a default curve; arbor_day's Bias curves default to flat.

### Edits

- **CE10 M** The editor sends each edit to the consumer, which applies, clamps or refuses it. The
  editor draws the curve the consumer holds afterwards, never a preview of its own.
- **CE11 M** The edits: move points, insert a point, delete points, move a handle, set a segment's
  kind.
- **CE12 M** A drag's edits carry its phase: start, in progress, finish, cancel. A move carries the
  offset from where the drag started, so a point clamped at a bound follows the pointer back.
- **CE13 M** Cancel, by Escape or by the editor closing mid-drag, returns the curve to how it was
  when the drag started.
- **CE14 M** The consumer receives each edit as a drag goes, not only at its end.
- **CE15 M** An insert carries the pointer's place. The consumer decides where the point goes, on
  the curve at the pointer's x or at the pointer, or refuses it.
- **CE16 M** The editor holds the selection, as a component the host may read, and changes it on
  the host's commands: select all, select none, select given points.
- **CE17 S** A drag is one undo step, where the host has undo.

### The editor

- **CE20 M** Depends on Bevy alone: not on arbor_day's types, nor on `bevy_immediate`, the
  immediate-mode UI library arbor_day's panel is built with. It lives in arbor_day as a module of
  its own until it moves upstream.
- **CE21 M** Headless parts, each usable alone, for showing and manipulating curves, as
  `bevy_ui_widgets` splits a menu into menu, items and popover; a feathers editor assembles them.
  Whether a headless widget also assembles them depends on its API being legible.
- **CE22 M** The headless parts decide what the editor's working depends on, and leave every
  visual choice to the consumer. Headless: the mapping between the curve's units and layout
  pixels; the curve's geometry as a path in layout pixels; where grid lines and labels fall;
  gestures; which point or handle is hovered, hit within a radius the consumer sets. The
  consumer's: the look of points, handles, curve, grid and labels, and what draws them. The
  headless parts publish their state as components (the mapping, the selection, the hovered point
  or handle, the gesture) for the consumer to style from, and work whether or not the consumer
  spawns entities for points and handles.
- **CE23 M** One gesture at a time: idle, moving the selected points, moving a handle, selecting by
  rectangle.
- **CE24 M** Click selects a point. Dragging moves it, kept between its neighbours' x.
  Double-click inserts. Delete or Backspace deletes the selection.
- **CE25 M** The selected point's x and y in feathers number inputs, typed or scrubbed.
- **CE26 M** Opens as a popup from a property row that shows the curve as a thumbnail, as the
  color input opens its picker. Other hosts embed the editor at a size they choose.
- **CE27 S** A grid, the ends and middle of each axis labelled in its units.
- **CE28 S** Arrow keys move the selection by a step, with a modifier for a larger one; Tab moves
  the selection to the next point. Neither fires while a number input has focus.
- **CE29 S** Reset to the default curve, and presets the consumer offers.
- **CE30 S** Several points selected, by modifier-click or by rectangle, move together.
- **CE31 C** A modifier holds a drag to one axis; another snaps it to the grid.
- **CE32 C** A repeating curve drawn faintly past its span.

### Kept possible

- **CE35** Several curves in one editor, as animation shows a transform's channels:
  - a point's identity includes its curve, and a selection and a drag may span curves;
  - each edit names the curve of each point it touches, and the consumer of each curve judges it;
  - the curves share the x axis, and each may have its own y axis, the view showing one at a time
    or each normalized to the view's height.
- **CE36** Zoom and pan: the view is a transform of its own within the axes' bounds, and
  gestures and edits are in the curve's units, never the view's.

### The plot

What the feathers editor draws.

- **CE40 M** The curve drawn antialiased at a constant width on screen, with no facets visible at
  the editor's size: a kink a sharp point, a step a vertical line.
- **CE41 M** Points and handles drawn over the curve, the selected and hovered ones marked, in the
  feathers theme's colors.
- **CE42 M** The row's thumbnail drawn the same way.
- **CE43 S** Draws on WebGL2 as well as WebGPU, or the widget says it is WebGPU only.

## Non-goals

- Color, or any value other than a scalar. A gradient is a widget of its own, which may later share
  the editor's axis and selection.
- A field choosing its source (a constant, a random range, a curve), as Unity's particle fields
  do. That belongs to the field, not the curve.
- Rotation. The editor edits scalar channels; making a rotation of them is the animation system's.
- Curves that are not functions of x: paths, closed shapes.

## Research before the design

Blender first: its F-curve editor and its curve-mapping widget (RGB curves, falloff), for how a
point is added, which interpolation and handle types exist and how they are shown, and what to
cut. Then, for what Blender lacks: Unity's `AnimationCurve` (weighted tangents), CSS easing
pickers, Godot's `Curve`. The output is a short write-up, before the design.

Read so far:

- Concept sketches of a curve editor and a particle editor
  (`https://claude.ai/artifact/ATq6tUP9KTm8NbQhQqrWM6`), rough, not a design: the widget holds
  only transient state (view, hover, drag snapshot, rectangle), the app the points; edits with a
  phase; an axis description per instance.
- The node-graph editor in `bevy_reactor` (`github.com/viridia/bevy_reactor`,
  `crates/bevy_reactor_nodegraph/src/gesture.rs`): one `Gesture` value for the interaction in
  progress, and `DragAction` phases. In `examples/graph.rs`, `on_connect`: the app judges each
  connection the user drags and draws it, valid or not.
- The particle system of faery, a three.js game engine (`github.com/viridia/faery`,
  `packages/engine/src/particles/`): curves sampled in the vertex shader, eight linear points; an
  emission rate over the emitter's time, looped; each field a constant or a curve.
