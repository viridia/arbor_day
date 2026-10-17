# How it works

arbor_day grows a low-poly plant from a species, a set of parameters an author edits in a panel
beside a 3D preview, and meshes it as bark tubes and leaf cards.

This document describes what is built. It does not argue for any of it: why each part is the way
it is lives in [decisions.md](decisions.md), and what the tool must do in
[requirements.md](requirements.md).

**What this document admits.** Mechanism, in the present tense. The test is whether a sentence would
still be true after the reason for it is forgotten. What has been built is described here and
nowhere else, so a second description elsewhere is a regression; a function's own details stay in
its doc comment.

**Numbering.** Sections are `## <n>.` and subsections `### <n>.<m>`, cited as `TD<n>` and
`TD<n>.<m>`. A section is renumbered only in a sweep that rewrites every reference to it.

---

## 1. The app

### 1.1 Shell and viewport

The window is a split pane: the panel on one side, the view on the other. The 3D camera draws into
a `Viewport` that follows a UI node (`ViewportSlot`): `follow_viewport_slot` runs in `PostUpdate`
after UI layout and sets the viewport from the node's `UiGlobalTransform` and `ComputedNode::size`,
both in physical pixels, so no scale factor is applied. The UI has a camera of its own, a
`Camera2d` with `IsDefaultUiCamera`, `order: 1` and `ClearColorConfig::None`, drawing over the
whole window. The view's node is pickable, and the turntable's drag and scroll observers sit on it
(TD1.3), so the panel's own drags and scrolls never reach the camera.

### 1.2 Settings

App state is saved with `bevy_settings` as `org.viridia.arbor_day`; under the driver,
`BEVY_SETTINGS_DIR` gives each run a fresh directory. The panel keeps a fixed width in pixels:
`Pane::size` is a flex weight, so the two panes get weights in pixels summing to the space they
share, reset when the window resizes, and a finished drag's first proposed size is the panel's new
width. Settings save on any `AppExit` (`SaveSettingsSync` in `Last`), and half a second after each
change (`src/settings.rs`), since on macOS Cmd+Q quits before the exit save runs. The color picker's
mode and recent colors are saved too.

### 1.3 Turntable

The camera (`src/turntable.rs`) circles a vertical axis through the subject with a 40° field of
view, framing from 45° above the horizon. Spin: horizontal drag or scroll. Tilt: vertical drag,
dragging down looking from above, 5° to 85°. Zoom: vertical scroll, moving the focus toward the
height under the pointer (where the pointer's ray crosses the vertical plane through the axis),
clamped to the subject's heights. There is no roll or pan.

Frame fits the subject's bounding sphere once, at start and on Frame, waiting until every subject
has bounds, and returns the tilt to 45°; it leaves the spin alone. The camera does not follow the
subject as it changes. While `keep_framed` is set the distance follows the viewport so the sphere
last framed still fits; spinning and tilting keep it, zooming clears it. The view is not saved.

A drag or scroll moves along one axis at a time: each axis's movement is summed, decayed with a
0.15 s time constant, and each event applies along the larger sum, so it starts at once and changes
axis when the other persists. Framing reads the subject's world transform, so it runs after
transform propagation (before frustum update) and sets the camera's `GlobalTransform` itself; the
camera has no parent.

### 1.4 Keys

Keys are handled by an observer of `FocusedInput<KeyboardInput>` on the primary window
(`src/keys.rs`): Frame, zoom and quit. Focused input reaches the window directly when nothing has
focus, or bubbling up from the focused widget. A feathers text input stops what it consumes (plain
and shifted characters, Space, editing keys, its own Cmd shortcuts) and passes the rest on (Escape
after blurring, Enter, Tab, other Cmd shortcuts), so letter keys never fire while typing and Cmd
shortcuts still reach the window.

### 1.5 The panel

The panel (`src/properties.rs`) walks the species by reflection, a row per field, keyed by its
`GetPath`. Field attributes, read from the type (`StructInfo::field_at(i).custom_attributes()`),
shape each row: `Caption` names it; `ValueRange<T>`, `Precision(i32)` (in the units shown) and
`Step<T>` bound and step a number; `NumberInputUnits` is the units (`angle_degrees` shows radians
as degrees, `ratio_percent` fractions as percentages); `Spinbox` drops the fill bar for chevrons on
hover. Every number is a feathers number input, its caption inside it, and its `ValueRange` the hard
limit. A `bool` is a checkbox row, a `Color` a feathers color input, and an enum a feathers select
right of its caption. Row captions use feathers' `label`.

A select's option rows carry `OptionIndex`; an observer records the picked index in a
`PickedVariant` component, which the panel reads on its next pass and applies through reflection
(`DynamicEnum`). A variant changed elsewhere is shown by triggering `SetSelected`. The panel's
capability set, `CapsPanel`, is `CapsUiFeathers`' list plus `CapabilityEnumSelect`.

A `Fuzzy` row (TD2.2) is a center input, a spread input and a shape toggle, with a strip drawn
along the center input showing center ± spread (`src/spread_strip.rs`). Nested structs are
subpanes; a `ControlGroup` attribute gathers a struct's rows under a bold heading where its first
field is declared, or into a subpane of its own at the panel's top level, so panes never nest
boxes. An array marked `ShownCount` shows its first items, the count naming a field; − and + on the
last shown item's header hide it or show the next, and a hidden item keeps its values. Each limb
level's header shows its count, "Limbs 1 (8)".

### 1.6 The HUD

The HUD (`src/hud.rs`) is a grid over the view pane with the panel's spacing (8 px padding and
gaps), ignored by picking, so the view's drags and scrolls pass between its buttons. Toggles for the
envelope and the ground sit in the upper left; their state is a field of `ViewSettings`, saved, and
each target's `Visibility` follows it, its icon dimming while the target is hidden. The camera's
buttons sit in the upper right, an L: orbit along the top, zoom and tilt down the side, Frame in the
corner. A camera button moves the camera from the moment it is pressed until it is released (60° a
second of orbit, 30° of tilt, a factor of 2 in distance); Frame acts on a click. The bottom row
holds the triangle count. The buttons are `bevy_ui_widgets` `Button`s the size of feathers' tool
button, in fixed translucent grays; the grid stops `PointerDrag` bubbling up from a button, which
`Button` does not. Icons are Material Symbols, 48 px white PNGs in `assets/icons/`.

## 2. The species

### 2.1 One resource

The species is one resource, `Species` (`src/species.rs`). The panel edits a copy and writes it
back with `set_if_neq`, and any change rebuilds the whole plant (`src/plant.rs`). The plant carries
the species it was grown from as a component (`GrownFrom`) and figures about the result
(`PlantStats`), which plans read.

Limb levels are a fixed array of three and a count of those shown, 1 to 3 (`limb_levels`); only
the shown levels grow.

### 2.2 Varied values

A `Fuzzy` (`src/fuzzy.rs`) is a center, a spread either side and a shape, bell or uniform.
`Fuzzy::sample` always draws three numbers: the bell is their mean, uniform uses the first.

Random numbers come from SplitMix64 (`src/random.rs`). A seed opens named streams,
`Rng::stream(seed, name)`, one per subsystem (`trunk` for the lean's heading, `branching` for the
stems), and a stream forks by index: an axis's scions fork from its stream by their index from 0,
and fixed streams take indices from `u64::MAX` down (wander, the axis's own values, layout,
carry-on, fork), so they never meet a scion's. A count given as a fraction is rounded at random,
`Rng::round`, always one draw.

A level's varied fields are drawn where they belong: each node's jitter and limb count once per
node, from the parent's layout stream; a limb's angle, reach and skew once per limb, from its own.
A drawn value is clamped to its field's `ValueRange`, read from the type (`LimbLevel::limits`).

## 3. The generator

`src/generator/`: plain functions on Bevy's math types, with no ECS. `grow` turns the species into
a skeleton; `plant` meshes it.

### 3.1 The skeleton

A `Skeleton` is a list of axes and a list of cards (`skeleton.rs`). An `Axis` is one stem of any
order, the trunk or a limb: its level (0 for a stem, 1 to 3 for the limb levels), its kind (stem,
lateral, carry-on, fork copy), its parent and the distance along it where it leaves, its rings,
its reach, its stream and its scions, by index. An axis bearing no scions bears leaves. A `Ring` is
a point on an axis, one at every trace step until the axis is hinted (TD3.4): center, tangent, side (carried along without twisting, setting the
ring's roll and its children's azimuth 0), distance along the axis and radius. A `Card` is a leaf
card: an axis, the distance along it of the card's base, and the card's direction.

`grow` makes the stems (TD3.2) and hints them (TD3.4), then grows each level in stages over a
`Level`, which holds the level's parameters and what every stage reads: place its laterals (TD3.3),
trace them (TD3.4), carry each parent bearing them on (TD3.6), wander (TD3.4), taper (TD3.5), fork
(TD3.7), and hint. Then it places the leaf clusters (TD3.8).

### 3.2 Stems

`stems` makes `trunk.stems` of them from the origin, spaced evenly around, each the trunk's height
long along its axis. Each leaves the ground at the splay from up and turns by the lean as it rises,
away from the middle, but never past level; the first's compass heading is drawn from the `trunk`
stream. Its course is an arc in one vertical plane, so its side never twists, and it wanders
about the course as limbs do (TD3.4), its tip on the course. Each has the trunk's radius and bears the first level. The crown is
measured against the trunk's height: its top a multiple, its base a share.

### 3.3 Placing laterals

`place_scions` places a level's laterals on each axis of the level before, along the part of it
inside the crown. Nodes are a density per crown radius of that part, the count rounded at random;
they spread evenly from the level's start (a share of the part; a fork's copy starts from its base)
to the tip, each moved from its even place by up to 30% of the gap. A node holds `per_node` limbs,
rounded at random.

A node's limbs are turned about the parent by its facing, measured from the parent's upper side,
the one nearest world up:
- around: spaced evenly, the group turned to the least cost of 72 turns;
- sides: on alternate flanks, the first on the flank of lower cost;
- up or down: all on the upper side or the underside.

A turn's cost is how crowded its limbs are by those already placed on the parent, less how far
they point out from the plant's axis. Crowding is how near each azimuth is to each placed one's,
raised to the 4th power, and faded to 0.7 a mean gap further along the parent; pointing outward,
the more so the further the node is from the axis, up to a crown radius, counts 1 against it. Each
limb then turns by its own random amount up to the level's skew, and leaves at its drawn angle from
the parent, its reach a drawn share of the distance to the envelope. Its base radius is the level's
radius share of the parent's radius there.

### 3.4 Tracing

`trace.rs` defines tracing. A limb's course turns toward an angle from world up (bend toward),
keeping its compass heading, closing a share of the angle left over each crown radius (bend). A
limb stops at its reach, or before a step that would leave the envelope. A traced limb no longer
than it is thick at its base is left out.

The level then wanders (`wander.rs`), after carrying on and before the taper; a fork's copies
wander as they are added, and stems as they are made. The course is displaced between two fixed
points, the end of a straight run at its base, a kink spacing (0.15 crown radii) give or take 40%,
and the start of the straight run under a tip's card (TD3.8; a stem's end): a segment longer than
1.5 spacings is split within 20% of its middle, and the split point moves square to the segment,
in a random direction, by a random share of the shorter half's length, or a spacing if less, times
the tangent of the wander, halved until neither half points further off the course than the
wander; then each half likewise. An axis too short for both runs does not wander. Offsets between split points are
interpolated along the course, each split point gets a ring, so each kink is a corner, and a
point displaced outside the envelope is pulled back toward the course until inside. Draws come
from the axis's wander stream.

A traced axis keeps a ring at every step through its level's stages. The last stage, hinting
(`hint.rs`), simplifies them by Ramer, Douglas and Peucker to the bark's detail threshold, keeping
a point only where it lies further than that from the chord between the points kept either side; a
point less than a trace step before the tip goes. Each kept ring faces halfway between the chords
meeting there; the start ring stays as given, and the tip heads as the trace did. The next level is
placed on the rings kept, so its laterals start inside the bark as meshed.

### 3.5 Taper

Every axis narrows evenly by the plant's one taper, from its base radius at its start to its tip
as grown, written on its rings (`taper_rings`). A lateral's base radius is set where it is placed
(TD3.3); an axis carrying another on starts at that one's radius there. Cutting an axis (TD3.6,
TD3.7) keeps its radii, the cut ring carrying the radius at the cut.

### 3.6 Carrying on

An axis bearing laterals, unless it forked, ends at its last lateral's node, and one more limb of
the level carries it on from there (`carry_on.rs`): along it, at its radius there, from a stream of
its own, otherwise grown as its siblings are. The parent is cut there and left open, its last ring
the carrier's first. If the carrier comes out no longer than it is thick, the parent keeps its full
length.

### 3.7 Forks

A level's axes fork with the level's chance (`fork.rs`), at a drawn share of their length, and end
there, capped. Copies carry the axis on, 2 to 4, of the same level: spread evenly around its
direction, each turned half the drawn fork angle from it, the first toward its flank turned up by
the roll. Each copy starts at the axis's radius at the fork and draws its reach, wander and limbs
from a stream of the fork's; it is traced and tapered as it is added. A copy no longer than it is
thick is left out; with none the axis does not fork. A forked axis bears neither a carrier nor
leaves at its tip, and copies do not fork again. A copy's limbs compete among themselves alone.

### 3.8 Foliage

`clusters` places cards at the tip of each limb bearing no scions, unless leaves are off. Their
size is a share of the crown radius. Stems bear none. A tip bears one card, unless its twig rises
45° or more above level at its tip, when the species' upright tips say: none, one, or a rosette
of two or three (`cards_at`).

One card lies along the twig, reaching back until its axial fold lies at the tip, or starting at
the twig's base on a twig shorter than that. A rosette's cards start at the tip, spaced evenly
about world up from the heading the twig points (straight up, the opposite of its upper side).
Each turns from the twig toward level in its own heading by three quarters of the angle between
them, so on an upright twig each rises 22.5°, and in a pair on a 60° one the card ahead rises 15°
and the one behind, turned over the top, 30°.

### 3.9 Meshing

`plant` meshes the skeleton (`mesh.rs`). Each axis is a tube through its rings, each ring turned to
its side, with its level's sides (the trunk's for stems), drawn at its ring's radius but none
narrower than half the detail threshold; each ring's normals lean forward by the slope between the
radii either side. A stem is open at its base; a tube is closed at its tip by a fan unless an axis
carries it on.

A card is two by two quads, its base where it was placed, its axis along its direction, its face
toward that axis's upper side. Each half turns down about the midrib by the lateral droop (up, cupping, if negative),
and the far half bends down across the middle by the axial droop. Every quad keeps the flat card's
normal. The texture's bottom is the card's base.
