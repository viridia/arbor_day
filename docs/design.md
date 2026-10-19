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

The view renders as the game does, by one resource, `GameLook` (`src/look.rs`), applied to the 3D
camera, the sun and `GlobalAmbientLight`: Panoply's, linear tonemapping, so painted colors show as
picked, a 1,750 lux sun, and ambient light at 700 in the average of its hemisphere light's sky and
ground colors. Other games' looks are project settings (X6).

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
along the center input showing center ± spread (`src/spread_strip.rs`). A `ControlGroup`
attribute gathers a struct's rows under a bold heading where its first field is declared, after
the `Icon` on that field if it has one, as an operator's heading is, or into
a subpane of its own at the panel's top level; a nested struct is shown the same way, a subpane at
the top level and a heading over its rows in a subpane, so panes never nest boxes. Its rows sit in
a node named after the field, so their paths stay apart from the outer struct's
(`levels[0]/fork/angle`). A struct field marked `Flatten` shows its fields' rows in place of its
own, with no node of its own, since Bevy UI has no `display: contents` and the panel's gaps differ
from a subpane's. An array marked `ShownCount` shows its first items, the count naming a field. The last
shown item's add menu ends, below a divider, with the next item ("Limbs 2", named `add`), and
each item past the fewest shown has a ✕ (`remove`) hiding it and those after it; a hidden item
keeps its values. Each limb
level's header shows its count, "Limbs 1 (8)".

A `Stripe` attribute colors a 4 px stripe down a top-level subpane's left side, the left border of
the subpane's outer node with 4 px of padding inside it, set on the node since feathers' theming
cannot shade a subtree. A `ControlGroup`'s stripe is on its first field, and an array's items take
one color each. A subpane with none keeps the room, clear. The species' run from bark brown to
light green, root to leaf: Plant, Trunk, Limbs 1 to 3, Foliage; Crown, the envelope rather than a
part, is grey.

An operator is a `Toggle<T>` field (`src/toggle.rs`): its settings and an `OperatorState`, off,
disabled or on, as Blender's modifiers are removed, hidden or shown; the generator runs it only
when on. The panel finds a toggle by reflection, through the `IsToggle` attribute on its type. On
or disabled, it is a heading over its settings' rows, after an icon of its kind (an `Icon`
attribute on the settings' type, tinted as the heading is), in a node named after the field, so panel
paths skip `settings` (`levels[0]/wander/angle`) while reflection paths keep it
(`limbs.levels[0].wander.settings.angle`). At the heading's right, an eye (`enabled`, Material
Symbols' `visibility` and `visibility_off`) switches it between on and disabled, and a ✕ (`off`)
turns it off. Off, it has no rows. The header of a level, or of a top-level struct holding
toggles (the trunk), has an add menu (`add_operator`, a boxed plus, a feathers menu) listing its
toggles that are off, an item each (`add_wander`), disabled while none is. Turned off and on
again, an operator keeps its settings for the session.

### 1.6 The HUD

The HUD (`src/hud.rs`) is a grid over the view pane with the panel's spacing (8 px padding and
gaps), ignored by picking, so the view's drags and scrolls pass between its buttons. Toggles for the
envelope, the ground and the wireframe sit in the upper left; their state is a field of
`ViewSettings`, saved, and the icon dims while the target is hidden. The envelope's and the
ground's `Visibility` follows it; the wireframe is Bevy's `Wireframe` with an orange
`WireframeColor`, added to the bark and the leaves while shown and removed while not. Its GPU
features (line polygon mode, immediates) need no request, since Bevy enables all the adapter
has. The camera's
buttons sit in the upper right, an L: orbit along the top, zoom and tilt down the side, Frame in the
corner. A camera button moves the camera from the moment it is pressed until it is released (60° a
second of orbit, 30° of tilt, a factor of 2 in distance); Frame acts on a click. The bottom row
holds the triangle count. The buttons are `bevy_ui_widgets` `Button`s the size of feathers' tool
button, in fixed translucent grays; the grid stops `PointerDrag` bubbling up from a button, which
`Button` does not. Icons are Material Symbols, 48 px white PNGs in `assets/icons/`.

### 1.7 Documents

File commands (`src/document.rs`) act on the one `Document`: its file's path and name, and the
species last saved. New, Open, Save and Save As are keys (⌘N, ⌘O, ⌘S, ⌘⇧S) and items of a file
menu (`menu.png`) at the right of a header row above the Plant group. The row's left, and the
window title, show the document's name, from its file ("Untitled" before the first save), and a dot
while the species differs from the one saved. New resets the species, not the view.

Open and Save As, and Save before the first save, pick a `.gltf` in a native dialog (`rfd`), run
as a task on the IO pool so the app keeps drawing; Save As's starts with the document's name.
Commands are ignored while a dialog is open. A pick writes a `FileRequest` (Open or Save As, with a
path), which plans send through the driver's `message`; the path becomes the document's.

Save (`src/gltf_file.rs`) writes the plant from `generator::plant` as a `.gltf` with its `.bin`
and leaf `.png` beside it, named after it, their URIs escaped: a scene named after the document,
holding the species file (TD2.3) under the extra `arbor_day`; a bark mesh (`COLOR_0`, linear) and a
leaf mesh (`TEXCOORD_0`, masked at `LEAF_CUTOFF`, double-sided), both materials matte and not
metallic, the bark's with the `"outline"` extra when outlined. The PNG is the texture's full-size
level; the pipeline mipmaps. Open reads the species back from the extra.

A menu item shows its keys dimmed at its right end (`src/shortcut.rs`), in the families "Fira
Sans, Key Symbols". Key Symbols (`assets/fonts/KeySymbols.ttf`) is DejaVu Sans subset to the key
symbols Fira Sans lacks, renamed; Parley picks a font per character. ⎋ is in neither, so Escape
shows as "Esc".

## 2. The species

### 2.1 One resource

The species is one resource, `Species` (`src/species.rs`). The panel edits a copy and writes it
back with `set_if_neq`, and any change rebuilds the whole plant (`src/plant.rs`). The plant carries
the species it was grown from as a component (`GrownFrom`) and figures about the result
(`PlantStats`), which plans read.

Limb levels are `LimbLevels`: a fixed array of three and a count of those shown, 1 to 3
(`limbs.shown`), each depth's defaults in its `Default`; only the shown levels grow (`grown`). The
panel shows it flattened, its levels beside the other subpanes.

### 2.2 Varied values

A `Fuzzy` (`src/fuzzy.rs`) is a center, a spread either side and a shape, bell or uniform.
`Fuzzy::sample` always draws three numbers: the bell is their mean, uniform uses the first. An
`Interval` is a center and a spread, always uniform, one draw; `offset` draws how far a value
strays, for values spaced evenly and then varied, as a sprig's leaves are. Its row is a `Fuzzy`'s
with an empty space where the shape button would be.

Random numbers come from SplitMix64 (`src/random.rs`). A seed opens named streams,
`Rng::stream(seed, name)`, one per subsystem (`trunk` for the lean's heading, `branching` for the
stems), and a stream forks by index: an axis's scions fork from its stream by their index from 0,
and fixed streams take indices from `u64::MAX` down (wander, the axis's own values, layout,
fork), so they never meet a scion's. A count given as a fraction is rounded at random,
`Rng::round`, always one draw.

A level's varied fields are drawn where they belong: each node's jitter and limb count once per
node, from the parent's layout stream; a limb's angle, reach and skew once per limb, from its own.
A drawn value is clamped to its field's `ValueRange`, read from the owning type
(`species::limits`).

### 2.3 The species file

A species file (`src/species_file.rs`) is JSON: `generator`, the version of the generator it was
made with (X15), then each of the species' fields in declared order, each written by reflection
through the type registry, with no serde derives. The version is the file's, not a species field,
so the panel never shows it. `LimbLevels` writes itself as a list of the shown levels, by
`SerializeWithRegistry` type data. serde_json's `Value` would sort the keys, so the file is
streamed rather than built as a `Value`. `src/default_species.json` is the default species as
written; a test compares the two. Custom serialization is type data on the type, not a
`ReflectSerializerProcessor`, since `bevy_settings` makes its own serializer (C8).

Reading takes `generator` off and refuses a file without one or from a newer generator, then
deserializes the rest strictly: a key the species lacks fails, naming it (D32). Reflection returns
only the fields the file holds, applied onto a template, the default species with every operator
off (`all_off`), so a field left out keeps its default and an operator left out is off.
`LimbLevels` reads one to three levels, each onto its depth's defaults with every operator off, the
levels after them their depths' defaults as they are.

An operator's `Toggle` writes its settings bare when on, and when disabled `"enabled": false`
before them; a struct holding operators leaves out those that are off (`serialize_on`), as
`LimbLevel`'s and `Trunk`'s `SerializeWithRegistry`. Read, an operator is on unless marked, its
settings read whole: a field missing from an operator fails, since its fields are fixed for its
version. Its settings go through a `serde_json::Value`, whatever the format, so `"enabled"` can be
taken off before reflection reads them.

BRP serializes through the same registry, so a plan reading the species (`GrownFrom`) sees the
file's shape, `at="limbs[0].angle"`, while a path it writes (`set`) is a reflection path,
`limbs.levels[0].angle`.

## 3. The generator

`src/generator/`: plain functions on Bevy's math types, with no ECS. `grow` turns the species into
a skeleton; `plant` meshes it.

### 3.1 The skeleton

A `Skeleton` is a list of axes and a list of cards (`skeleton.rs`). An `Axis` is one stem of any
order, the trunk or a limb: its level (0 for a stem, 1 to 3 for the limb levels), its kind (stem,
lateral, fork copy), its parent and the distance along it where it leaves, its rings,
its reach, its stream and its scions, by index. A stem or limb that did not fork bears leaves at its tip. A `Ring` is
a point on an axis, one at every trace step until the axis is hinted (TD3.4): center, tangent, side (carried along without twisting, setting the
ring's roll and its children's azimuth 0), distance along the axis and radius. A `Card` is a leaf
card: an axis, the distance along it of the card's base, and the card's direction.

`grow` makes the stems (TD3.2) and hints them (TD3.4), then grows each level in stages over a
`Level`, which holds the level's parameters and what every stage reads: place its laterals (TD3.3),
trace them (TD3.4), wander (TD3.4), taper (TD3.5), fork (TD3.7), and hint. Then it places the leaf clusters (TD3.8).

### 3.2 Stems

`stems` makes the trunk from the origin, the trunk's height long along its axis. It leaves the
ground upright and, with Lean on, turns by its angle as it rises, but never past level, toward a compass heading
drawn from the `trunk` stream. Its course is an arc in one vertical plane, so its side never
twists, and it wanders about the course as limbs do (TD3.4), its tip on the course. With the
trunk's Fork on, it forks as a limb does (TD3.7), but each copy is a stem: an arc from the fork,
leaving at its own angle from up (none below level) toward its own heading and turning by the
lean's share left past the fork, away from the middle, the rest of the height long, so every tip
is the height from the ground along the stems. Forked at its base, its copies are stems rising
from the ground, a clump. Every stem bears the first level. The crown is measured against the
trunk's height: its top a multiple, its base a share.

Where stems meet the ground (`ground.rs`): with Flare on, each stem rising from the ground is
wider by the flare's width share of its base radius at the ground, the widening falling off as
the square of the way up to three base radii, so it meets the taper without a crease. It is added
after the stems are hinted and before the first level is placed, so limbs low on the trunk take
its radius; rings are added along its curve where it lies further than the detail threshold from
the rings either side. Last of all, each such stem reaches straight down below the ground by its
ground radius, the tangent of the steepest slope a plant stands on (45°), so a plant on a slope
meets the ground on its low side; that ring's distance along the axis is below 0.

### 3.3 Placing laterals

`place_scions` places a level's laterals on each axis of the level before, along the part of it
inside the crown. Nodes are a density per crown radius of that part, the count rounded at random;
they spread evenly from the level's start (a share of the part; a fork's copy starts from its base)
to its end (a share too; below the start, every node is at the start), each moved from its even place by up to 30% of the gap. A node holds `per_node` limbs,
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
the parent, its reach a drawn share of the distance to the envelope. With Bias on, both change
with the node's index about the middle node's, from −1 at the first to 1 at the last: the reach
is scaled by 1 less the size times it, and the angle less the bias angle times it is added, then
clamped to the level's limits, so the level's mean holds and the start and end do not change
their meaning. Its base radius is the level's radius share of the
parent's radius there.

### 3.4 Tracing

`trace.rs` defines tracing. A limb's course turns in its vertical plane, keeping its compass
heading. With Sag on, it turns down by the sag's angle per crown radius times the sine of its
angle from up, solved exactly per step (the tangent of the half angle grows by e to the turn), so
neither an upright nor a hanging course sags. With Rise on, it then closes the rise's share of
its angle from up over each crown radius. With neither, the course is straight. A
limb stops at its reach, or before a step that would leave the envelope. A traced limb no longer
than it is thick at its base is left out.

The level then wanders (`wander.rs`), after tracing and before the taper; a fork's copies
wander as they are added, and stems as they are made. The course is displaced between two fixed
points, the end of a straight run at its base, a kink spacing (0.15 crown radii) give or take 40%,
and the tip: a segment longer than 1.5 spacings is split within 20% of its middle, but never
inside the straight run under a tip's card (TD3.8), where the split falls at the run's start, so
the last segment runs straight to the tip at whatever angle; and the split point moves square to the segment,
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
(TD3.3); a fork's copy starts at its axis's radius at the fork. Cutting an axis at a fork (TD3.7)
keeps its radii, the cut ring carrying the radius at the cut.

### 3.6 Branch ends

An axis bearing laterals runs on past its last node to its own tip, as traced, its sag, sides and
wander its own level's throughout. Its nodes stop at its level's end (TD3.3), so the tip runs on bare past them. Unless it
forked, its tip bears leaves as a twig's does (TD3.8). Only a fork cuts an axis (TD3.7).

### 3.7 Forks

With Fork on, a level's axes fork with its chance (`fork.rs`), at a drawn share of their length, and end
there. Copies carry the axis on, 2 to 5, of the same level: spread evenly around its
direction, each turned half the drawn fork angle from it, the first toward its flank turned up by
the drawn roll (`copy_directions`). Each copy starts at the axis's radius at the fork, rolled to the axis's side there so the first carries on its tube untwisted, reaches what the axis had left to grow past it, so a short limb forks short, and draws its wander and limbs
from a stream of the fork's; it is traced and tapered as it is added. A copy no longer than it is
thick is left out; with none the axis does not fork. A forked axis bears no leaves at its tip, and copies do not fork again. A copy's limbs compete among themselves alone.
A fork nearer the axis's base than its radius (`at_base`) leaves no stub: the copies take the
axis's place, the first in its slot and kind, the rest as laterals beside it, all from the axis's
parent where it left. Stems fork the same way (TD3.2).

### 3.8 Foliage

`clusters` places cards at the tip of each stem or limb that did not fork, while the tip leaves are on, an
ornament slot (`docs/patches.md`, Stacks) whose panel and file are an operator's (TD1.5, TD2.3). Their
size is a share of the crown radius. A tip bears one card, unless its twig rises
45° or more above level at its tip, when the species' upright tips say: none, one, or a rosette
of two or three (`cards_at`).

One card lies along the twig, reaching back until its axial fold, where the painted sprig's fan
starts (`axial_fold`, TD3.10), lies at the tip, or starting at
the twig's base on a twig shorter than that. A rosette's cards start at the tip, spaced evenly
about world up from the heading the twig points (straight up, the opposite of its upper side).
Each turns from the twig toward level in its own heading by three quarters of the angle between
them, so on an upright twig each rises 22.5°, and in a pair on a 60° one the card ahead rises 15°
and the one behind, turned over the top, 30°.

### 3.9 Meshing

`plant` meshes the skeleton (`mesh.rs`). Each stem or limb, with its fork's first copy and so on
(`chain`), is one tube through their rings, each ring turned to its side, with its axis's level's
sides (the trunk's for stems), drawn at its ring's radius but none narrower than half the detail
threshold; each ring's normals lean forward by the slope between the radii either side, distances
running on along the chain. A join's ring is drawn once, facing halfway between the segments
meeting there, so a fork's turn is an elbow. A fork's other copies
are tubes of their own, open at their base inside the chain's, as laterals are inside their
parents'. A segment between rings of equal sides is a band of
quads; between N and M, a stitch of N + M triangles walking both rings by
angle. A tube is open at its base and closed at its tip by a fan.

A card is two by two quads, its base where it was placed, its axis along its direction, its face
toward that axis's upper side. Each half turns down about the midrib by the lateral droop (up, cupping, if negative),
and the far half bends down across the axial fold by the axial droop. Every quad keeps the flat
card's normal. The texture's bottom is the card's base.

On an outlined species the app (`src/outline.rs`) draws the bark again, sharing its mesh, with
Panoply's `OutlineMaterial`: an inverted hull, each vertex pushed out along its normal by the bark
outline's width in the mesh's own units, front faces culled, unlit, black, casting no shadow. Off,
the hull's entity is hidden.

### 3.10 The leaf texture

`leaf.rs` paints the tip leaves' sprig (D29) with `tiny-skia` (D20), 256 pixels square, in leaf
lengths, the middle leaf's, until fitted. A leaf's right half is a cubic from the base, (0, 0), to
the tip, (0, 1), its inner control points set by the widths and tapers, cut into segments by
blossoming; each segment but the last has its end pushed out by the serration, toward the tip by
the rake in segment lengths, and at random by the jitter (stream `leaf shape`), its control points
moved in proportion to how far along it they lie, and the tip is raised by the serration. The left
half is its mirror. The sprig (stream `sprig`) is the middle leaf, the fan either side of it, leaf
*i* of *n* turned *i*/*n* of the angle's center and scaled by the outer size to the power *i*/*n*,
and the pairs below, a spacing apart, at the angle and the outer size; each leaf's angle and each
pair leaf's place stray by their spreads.

`layout` fits the sprig to the texture: symmetric about the stem, so it lies on the card's
midrib, square, padded by half the outline and a pixel, the tip up. Where the fan starts, as a
share of the height from the bottom edge, is the card's axial fold. `paint` strokes the stem, from
below the bottom edge to the fan, and every leaf black, the outline's width wider; then the stem in
the bark color; then each leaf in turn stroked translucent black and filled with a linear gradient
in its own frame, lateral (outer, inner at the midrib, outer, across 1.2 times the half's widest
control point) or axial (inner at the base, outer at the tip), both colors shifted in Oklch
(stream `leaf color`) by up to 144° of hue and 0.4 of lightness at a variation of 1. The strokes,
black and translucent, are the leaf outline's width, and painted only on an outlined species. A
golden test compares the default species' texture with
`src/generator/leaf-golden.png`, each channel within 2, and on a difference writes what it painted
to `target/leaf-golden.png`.

The app (`plant.rs`) gives the leaves' material a new image on each build, with its mipmaps: each
level a 2 × 2 box filter of the one before, colors weighted by alpha so transparent pixels add none
of theirs. A new image rather than the old one replaced, since Bevy reuses the GPU texture of an
image replaced at the same size and writes only its first level (X48).
