# Curve editor: prior art

A survey of curve editors, for the requirements in [curve-editor.md](curve-editor.md): what each
does, what to take, and what to cut. Blender comes first, since its model is the reference;
Unity, CSS and Godot fill in what Blender lacks.

## Blender

### Graph editor (F-curves)

The animation curve editor. A key's interpolation applies to the segment after it:

- **Constant**, **Linear**, or **Bézier**, the default.
- **Easing** (sine, quadratic, cubic and so on, by strength) and **Dynamic Effects** (back,
  bounce, elastic), each with an easing direction: automatic, in, out, or in and out. These are
  Robert Penner's easing equations, which Bevy's `EaseFunction` also provides.

Each Bézier key has a left and a right handle, and each handle has a type:

- **Automatic**: placed to make a smooth curve.
- **Auto Clamped**: automatic, but clamped to prevent overshoots and direction changes between
  keys. The default.
- **Vector**: points at the neighbouring key, making straight segments.
- **Aligned**: set by hand, the two handles kept pointing in opposite directions, so the curve
  stays smooth at the key.
- **Free**: set by hand, each handle independent, so the curve can turn sharply at the key.

Handles are 2D: they have both a length and a direction. Blender shortens a segment's handles
when evaluating it if they would make the curve double back in time, so a Bézier F-curve stays a
function of time.

The sidebar shows the active key's interpolation, easing, frame and value, and each handle's type,
frame and value, as number fields. Keys are inserted at each curve's current value, or at the 2D
cursor; moved with axis constraints; and snapped to frames, seconds or markers. Before the first
key and after the last, a curve extrapolates as constant or linear; a Cycles modifier makes it
repeat.

With several curves shown, **Normalize** scales each curve's display to occupy −1 to 1, so curves
of different ranges can be compared and edited together. **Frame All** fits the view to every
key; **Frame Selected** fits it to the selection.

### Curve-mapping widget

The small curve editor embedded in property panels: RGB curves, brush falloff, the profile of a
bevel. It is closest to arbor_day's popup.

- **Adding**: a click on the curve where no point exists adds a point there.
- **Deleting**: X, or a button. The first and last points cannot be deleted.
- **Handle types per point**: Auto, Vector, and Auto Clamped (which prevents overshoot) for
  mapping curves; Free and Aligned only for profile curves.
- **Extend**: before the first point and after the last, horizontal or extrapolated.
- **Clipping**: an optional rectangle (min and max x and y) that keeps points within bounds.
- **Selected point**: x and y number fields.
- **View**: zoom in and out buttons, drag to pan, and reset view. Zooming out stops at the clipping
  rectangle.
- **Presets** for brush curves: Smooth, Round, Root, Sharp, Linear, Constant. **Reset Curve**
  removes the added points.
- **Copy and paste**: Ctrl-C and Ctrl-V over a widget copy the whole curve to another widget.

Blender evaluates mapping curves from a precomputed table of samples rather than from the points.

## Unity

`AnimationCurve` stores Hermite keys: each key a time, a value, and an in and out tangent, which
is a slope. Weighted tangents add an in and out weight per key, which turn each segment into a
Bézier whose handles have a length as well as a slope; unweighted, a tangent cannot go vertical.
Before and after its keys, a curve clamps, loops or ping-pongs.

In the curve editor, double-clicking the curve adds a key on it, and right-clicking a key sets its
tangent mode:

- **Clamped Auto**, the default: tangents set to pass smoothly through the key, adjusted as the
  key moves to avoid overshooting.
- **Free Smooth**: handles dragged freely, kept co-linear. **Flat** is Free Smooth with
  horizontal tangents.
- **Broken**: each side set separately, as Free, Linear (pointing at the neighbouring key) or
  Constant.

Dragging a tangent handle by hand switches the key to Free Smooth.

A particle field is a `MinMaxCurve`: a constant, a curve, random between two constants, or
random between two curves, the curve scaled by a multiplier. Unity's curve fields show a
thumbnail in the inspector and open the editor in a popup with a presets library.

## CSS easing

`cubic-bezier(x1, y1, x2, y2)` is one Bézier segment from (0, 0) to (1, 1); the handles' x must lie
within 0 to 1, and their y may overshoot. `steps(n, jump)` is a staircase. `linear()` takes a list
of stops, a piecewise-linear curve with optional x for each stop. Browser easing editors (Chrome's
DevTools, for one) show the two handles, named presets (`ease`, `ease-in` and so on), and a
preview of the motion.

## Godot

`Curve` holds points, each a position and a left and right tangent with a mode for each side: Free,
set by hand, or Linear, aimed at the neighbouring point. The tangents are slopes, unweighted.
`min_domain` and `max_domain` bound x; `min_value` and `max_value` bound y, though tangents can
carry the curve past them between points. `sample` evaluates from the points; `sample_baked`
from a cache of `bake_resolution` samples, 100 by default.

## Findings

1. **Shape-preserving is a handle mode, not a segment kind.** Blender's Auto Clamped and Unity's
   Clamped Auto are cubics whose handles are computed so the curve does not overshoot. So the
   segment kinds can be three, constant, linear and cubic, with a mode on each cubic handle:
   auto-clamped (computed, the default), aligned and free (set by hand). Bias curves are cubics
   whose handles are all auto-clamped and hidden. Blender's guarantee is weaker than strict
   monotone interpolation (Fritsch–Carlson), which never overshoots between points; which one
   auto-clamped means is for the design.
2. **Handles need a length to reach vertical.** Godot's and unweighted Unity's tangents are
   slopes, which cannot go vertical. Blender's 2D handles, and Unity's weighted ones, can. CSS's
   `cubic-bezier` and the crown's rounded ends both need 2D handles, which must then be kept from
   doubling the curve back on itself: Blender shortens them when evaluating, and CSS restricts their
   x.
3. **Penner easing per segment is cheap to sample but costly to show.** Bevy's `EaseFunction`
   provides it, but it adds two menus per segment, and a shader needs an evaluator for each
   function. Cut: presets cover the common shapes as cubics.
4. **Inserting is on the curve everywhere.** Blender's widget adds a point on a click on the
   curve, Unity on a double-click; neither adds one off the curve. A double-click rather than a
   click suits an editor where a drag on empty space selects by rectangle.
5. **The end points stay.** Blender's widget cannot delete its first and last points.
6. **Extrapolation has a third mode.** Blender extends linearly as well as flat, and Unity
   ping-pongs as well as loops. Linear extension is cheap; ping-pong can be cut.
7. **Bounds bind the points, not the curve.** Godot says so outright. Only shape-preserving
   handles keep the curve itself within the points' bounds.
8. **Baking is common.** Blender and Godot both sample curves from a precomputed table. A table
   is another GPU encoding, beside sending Hermite segments: any segment kind, one evaluator,
   at the cost of resolution.
9. **Several curves need a shared y.** Blender's Normalize scales each curve to −1 to 1, as the
   requirements' several-curves option suggests.
10. **Small editors zoom.** Blender's widget, the closest to a popup, has zoom and pan, stopping at
    its clipping rectangle.
11. **Copying a whole curve** between widgets is a cheap convenience Blender has.
12. **Embedded, one curve is edited at a time.** Blender's graph editor lets a selection and a
    drag span curves; its curve-mapping widget, the embedded one, shows several curves (C, R, G,
    B) and edits only the one chosen above the plot.

## What to cut from Blender

Modifiers (noise, generator, envelope, stepped) and drivers; circle and lasso selection; the 2D
cursor; Penner easing per segment (finding 3); handle equalizing and flattening operators;
filters on the channel list.

## Changes made to the requirements

- **CE3**: three segment kinds (constant, linear, cubic), and a mode per cubic handle:
  auto-clamped, aligned or free. Unity's Broken is free on one side and another mode on the
  other.
- **CE5**: linear extension, beside holding and repeating.
- **CE33**: copy and paste a whole curve between editors.
