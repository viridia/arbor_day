# Curve editor: requirements

A curve here is a scalar function of one variable, y = f(x), shaped by hand from a few points.
The first user is arbor_day, a generator of low-poly trees with a Bevy app for authoring them.
There, curves replace settings that today can only vary linearly along a limb. The editor is also
meant for other Bevy editors and, once its API settles, for `bevy_ui_widgets` and feathers.

Three parties are involved:

- The **consumer** owns the curve's data and samples it. In arbor_day, that is a field of a
  tree's species and the generator that reads it.
- The **editor** is the widget that shows the curve and sends edits to the consumer.
- The **host** is the UI that contains the editor, such as a popup opened from a property row,
  or a panel.

Priority: **M** must, **S** should, **C** could. An **opportunistic** use is supported only if
the design allows it cheaply, even approximately; it never shapes the design. A **kept possible**
requirement is not built now, but the design must not rule it out, and is reviewed against it.
Requirements are numbered `CE<n>`; a number is never reused.

## Uses

| Use | x | y | Needs |
| --- | --- | --- | --- |
| arbor_day's Bias (built first): how a limb's side branches change from its base to its tip, in length, leaf size and aim | Place along the limb, 0 to 1 | The field's value, in its units and range | A few points; never overshoots them; a popup from a row in the property panel; the tree regrown live while dragging |
| arbor_day's crown profile (opportunistic): the outline of the tree's crown | Height, from the crown's base to its top | Width, as a share of the crown's radius | Rounded or pointed ends and straight sides, approximately; today's five fixed shapes (cone, ellipsoid and so on) as presets |
| CSS-style easing | Normalized time, with the ends fixed at (0, 0) and (1, 1) | Progress, which may overshoot | A single cubic Bézier segment whose two handles keep x within 0 to 1; or steps |
| Particle scalars | Normalized particle age, or the emitter's time, looping | Size, alpha, speed, emission rate | Held or repeated past the ends. When sampled on the GPU, limited to the segment kinds and number of points the shader's encoding supports |
| Animation tracks (kept possible) | Seconds, over an open span | One channel: an axis of translation or scale, an Euler angle, or a quaternion component | Interpolation and tangent mode per point; several channels in one view; zoom, pan, frame snapping, a playhead |

## Requirements

### The curve

- **CE1 M** A curve is a list of points joined by segments, with `f32` x and y, and x never
  decreasing. Two points may share an x, which makes a step; three may not.
- **CE2 M** The consumer describes each axis: its bounds, each either clamped or open; its units
  and precision; and its snap step. Every point stays within clamped bounds.
- **CE3 M** The consumer declares which segment kinds and handle modes a curve may use, and the
  editor offers only those. The segment kinds are constant, linear and cubic. A cubic segment has
  a handle at each end, with a length as well as a direction, so it can leave a point vertically
  and covers CSS's `cubic-bezier`. Each handle has a mode:
  - **auto-clamped**: computed so the curve never overshoots its points, and hidden;
  - **aligned**: set by hand, kept opposite the point's other handle so the curve stays smooth;
  - **free**: set by hand, independent of the other handle.

  Handles are kept from making the curve double back on itself, so it stays a function of x.
- **CE4 M** The consumer sets the minimum and maximum number of points, and may pin the end
  points' x, or both their x and y.
- **CE5 M** Beyond its ends, a curve holds its end values, continues at its end slopes, or
  repeats; the consumer declares which.
- **CE6 M** A curve is sampled as a `bevy_curve` `Curve<f32>`, giving its value and slope anywhere
  in its span. The same curve and x give the same result on every platform.
- **CE7 M** A selection and a drag survive points being added or removed elsewhere on the curve.
  When the consumer replaces the whole curve, with a preset or a reset, the selection is cleared
  or trimmed to the points that remain; on undo, the host restores it (CE17).
- **CE8 M** The consumer owns its curve's storage and encoding; the editor sees only the copy it
  is given. In arbor_day, a curve field serializes by reflection as a short list of points, and a
  field that was a single number converts to a curve that gives the same result, so saved files
  keep their meaning.
- **CE9 M** Each consumer sets a default curve; arbor_day's Bias curves default to flat.

### Edits

- **CE10 M** The editor sends each edit to the consumer, which applies, clamps or refuses it. The
  editor then draws the curve as the consumer holds it, never a preview of its own.
- **CE11 M** The edits are: move points, insert a point, delete points, move a handle, and set a
  segment's kind.
- **CE12 M** Each edit in a drag carries the drag's phase: start, in progress, finish or cancel. A
  move carries its offset from where the drag started, not from the previous edit, so a point
  clamped at a bound follows the pointer back when the pointer returns.
- **CE13 M** Cancelling a drag, by Escape or by the editor closing mid-drag, returns the curve to
  its state when the drag started.
- **CE14 M** The consumer receives each edit while a drag is in progress, not only when it ends.
- **CE15 M** An insert carries the pointer's position. The consumer decides where the new point
  goes (on the curve at the pointer's x, or at the pointer itself), or refuses it.
- **CE16 M** The editor holds the selection in a component the host can read. The host changes it
  by command: select all, select none, or select given points. The editor notifies the host
  whenever the selection changes.
- **CE17 S** Where the host has undo, a drag is one undo step, and a selection change is not a
  step. Instead, the host stores the current selection with its most recent step, updating it
  whenever the selection changes. Undoing a step restores the curve together with the selection
  stored with it.

### The editor

- **CE20 M** The editor depends on Bevy alone: not on arbor_day's types, nor on `bevy_immediate`,
  the immediate-mode UI library that arbor_day's panel uses. It lives in arbor_day, as a module
  of its own, until it moves upstream.
- **CE21 M** The editor is built from headless parts, each usable on its own, for showing and
  manipulating curves, as `bevy_ui_widgets` splits a menu into menu, items and popover. A
  feathers editor assembles them. Whether a headless widget also assembles them depends on
  whether its API proves legible.
- **CE22 M** The headless parts own whatever the editor needs in order to work, and leave every
  visual choice to the consumer.
  - Headless: the mapping between the curve's units and layout pixels; the curve's geometry, as a
    path in layout pixels; where grid lines and labels fall; gestures; and which point or handle
    the pointer is over, within a hit radius the consumer sets.
  - The consumer's: how points, handles, the curve, the grid and labels look, and what draws them.

  The headless parts publish their state as components (the mapping, the selection, the hovered
  point or handle, the gesture in progress) for the consumer to style from. They work whether or
  not the consumer spawns entities for points and handles.
- **CE23 M** One gesture is active at a time: idle, moving the selected points, moving a handle,
  or selecting by rectangle.
- **CE24 M** Clicking a point selects it. Dragging moves it, kept between its neighbours' x.
  Double-clicking empty space inserts a point, and double-clicking a point deletes it. Delete or
  Backspace deletes the selection.
- **CE25 M** When exactly one point is selected, its x and y are shown in feathers number inputs,
  which can be typed into or scrubbed. With none or several selected, the inputs are hidden or
  disabled.
- **CE34 S** When that point has handles set by hand (aligned or free), each one's x and y are
  shown in the same way, a row per handle.
- **CE37 M** Handles are shown only on selected points, and only where set by hand. Dragging a
  handle moves that handle alone, whatever is selected; handles are never selected themselves.
- **CE26 M** The editor embeds in a host's layout at whatever size the host chooses: inline in a
  property panel, expanded from a row that shows the curve as a thumbnail, or in a popup from such
  a row. arbor_day embeds it inline, so it covers none of the 3D view.
- **CE27 S** A grid, with each axis labelled in its units at its ends and middle.
- **CE28 S** Arrow keys move the selection by one step, or by a larger step with a modifier; Tab
  moves the selection to the next point. Neither applies while a number input has focus.
- **CE29 S** Commands to reset to the default curve, and to apply presets the consumer offers.
- **CE30 S** Several points can be selected, by modifier-click or by rectangle, and move together.
- **CE31 C** One modifier constrains a drag to one axis; another snaps it to the grid.
- **CE32 C** A repeating curve is drawn faintly beyond its span.
- **CE33 C** Copy and paste a whole curve between editors whose consumers accept it.

### Kept possible

- **CE35** Several curves in one editor, as an animation editor shows a transform's channels:
  - a selection or a drag may span curves, each selected point naming its curve;
  - each edit names the curve of every point it touches, and each curve's consumer judges its own
    points;
  - the curves share the x axis but may each have their own y axis, the view showing one y axis
    at a time or scaling each curve to the view's height.
- **CE36** Zoom and pan: the view is a transform of its own, within the axes' bounds, and
  gestures and edits are expressed in the curve's units, never the view's. The host may set the
  view. Scrollbars, if any, are the host's, drawn from the view; the inline editor has none.

### The plot

What the feathers editor draws:

- **CE40 M** The curve is drawn antialiased, at a constant width on screen, with no visible facets
  at the editor's size. A kink stays a sharp point, and a step a vertical line.
- **CE41 M** Points and handles are drawn over the curve, with the selected and hovered ones
  marked, in the feathers theme's colors.
- **CE42 M** The row's thumbnail is drawn the same way.
- **CE43 S** The plot draws on WebGL2 as well as WebGPU, or is documented as WebGPU only.

## Non-goals

- Color, or any value other than a scalar. A gradient editor is a widget of its own, which may
  later share the curve editor's axis and selection.
- A field choosing its source (a constant, a random range or a curve), as Unity's particle fields
  do. That choice belongs to the field, not the curve.
- Rotation. The editor edits scalar channels; combining them into a rotation is the animation
  system's job.
- Curves that are not functions of x, such as paths and closed shapes.

## Research before the design

Blender first: its F-curve editor and its curve-mapping widget (RGB curves, falloff), to learn
how points are added, which interpolation and handle types exist, how they are shown, and what to
cut. Then, for what Blender lacks: Unity's `AnimationCurve` (weighted tangents), CSS easing
pickers and Godot's `Curve`. The write-up is
[curve-editor-prior-art.md](curve-editor-prior-art.md).

Read so far:

- Concept sketches of a curve editor and a particle editor
  (`https://claude.ai/artifact/ATq6tUP9KTm8NbQhQqrWM6`), rough rather than a design. From them:
  the widget holds only transient state (the view, hover, the drag's starting snapshot, the
  selection rectangle) while the app holds the points; edits carry a phase; each instance has an
  axis description.
- The node-graph editor in `bevy_reactor` (`github.com/viridia/bevy_reactor`,
  `crates/bevy_reactor_nodegraph/src/gesture.rs`). From it: a single `Gesture` value for the
  interaction in progress, and the `DragAction` phases. In `examples/graph.rs`, `on_connect`
  shows the app judging each connection as the user drags it, and drawing it, valid or not.
- The particle system of faery, a three.js game engine (`github.com/viridia/faery`,
  `packages/engine/src/particles/`). Its curves are sampled in the vertex shader, as up to eight
  points joined linearly; its emission rate runs over the emitter's time, looping; and each field
  is either a constant or a curve.
