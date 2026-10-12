# arbor_day ledger

What has landed, what is next, the decisions that stand, and what is deferred and why. What the
tool must do is in [requirements.md](requirements.md). Chunks are `C<n>`, decisions `D<n>`,
deferrals `X<n>`. An id is assigned once and never reused or renumbered; Next lists chunks in the
order they are meant to land, which their ids need not follow. Panoply's ledger
(`../panoply/docs/migration.md`) is cited as "Panoply's".

## Landed

| Commit    | Change                                                                                  |
| --------- | --------------------------------------------------------------------------------------- |
| `0947b01` | Repo started: `AGENTS.md`, `CLAUDE.md`, this ledger; requirements moved from Panoply.  |
| `0947b01` | C1 Minimal shell (D5): split pane, `bevy_immediate` panel, viewport following the view pane; `plans/smoke.py`. |
| `d1ae223` | C2 Settings (D6), `src/settings.rs`: window state and panel width.                      |
| `eafef5a` | C4 Turntable camera (D7), `src/turntable.rs`.                                           |
| `0d1aee5` | C3 Keys (D8), `src/keys.rs`: frame, zoom, quit.                                         |
| `65d6413` | C5 Property editor by reflection, `src/properties.rs`: booleans; `Placeholder` stands in for the species. |
| `65d6413` | C9 Numbers (D9): feathers number inputs shaped by field attributes.                     |
| `a19ec61` | C11 The panel scrolls, as feathers' list view does.                                     |
| `df0b83c` | C12 Fuzzy numbers (D10), `src/fuzzy.rs`: center, spread and a shape toggle; `plans/fuzzy.py`. |
| `93b3dbb` | C13 Captions above fields, so a fuzzy row's center input gets the panel's width.        |
| `8d39dc3` | C14 Spread strip, `src/spread_strip.rs`: center ± spread drawn along the center input.  |
| `0e9fdef` | C6 Growth-model survey, `docs/growth-model.md` (D11).                                   |
| `3dcfc10` | C15 A trunk generated from the species (D12): `src/species.rs`, `generator.rs`, `plant.rs`. |
| `0950771` | C16 Crown envelope and enum selects (D13); `plans/envelope.py`.                         |
| `806fed9` | C17 Nested structs as subpanes (`properties::group`).                                   |
| `b757d16` | C21 Turntable feel (D7): one-shot framing, one axis per gesture.                        |
| `e05893b` | C20 Bark color, and feathers' color input for `Color` fields; `plans/bark.py`.          |
| `a9399e7` | C22 Literal names in `bsn!` as `#name`.                                                 |
| `0a3924b` | C18 One level of limbs (D14), reaching toward the envelope; `plans/limbs.py`.           |
| `1e7bd38` | C19 Up to three limb levels (D15), each at an angle from its parent; `plans/levels.py`. |
| `5621ce7` | C26 Random streams and `Fuzzy::sample` (D16), `src/random.rs`.                          |
|           | C23 Curved limbs (D17): bend and wander per metre, and the seed; `plans/growth.py`.     |

## Next

File handling brings Save and Open keys (D8), and Save and Open buttons at the top of the panel.
The seed row moves in with them, in a feathers `group` (`containers/group.rs`, no header), so it
lines up with the subpanes' rows rather than standing at the panel's edge.

The property editor gains a widget kind per chunk, built against `Placeholder` until a species
field needs it (C27). Rows are keyed by field path, in `GetPath` syntax, which C7's undo records also use.
Seedling's look is not a target. `bevy_inspector`'s `details_panel` also walks reflected fields,
but reads no attributes, has no per-type hook, and is retained-mode. The generator sets the order
(`docs/growth-model.md`), each step bringing the widget kind it needs:

- **C28 Varied level fields**: bounded jitter within per-level limits; the level's fields that
  seedling varied become `Fuzzy` (span, reach, angle, divergence), drawn from each limb's
  stream (D16, D17). Where each value is used, the generator takes a wrapping angle modulo a
  turn and clamps the rest to the field's limits, since the limits are editor attributes it
  does not read. Then drop the `dead_code` expectation on `Fuzzy::sample`.
- **C29 Pipe-model radii**, and limbs below a radius left to the foliage rather than meshed.
  Until then limbs taper as the trunk does, and three default levels are heavy (wandering limbs
  add rings, 2 × sides triangles each); revisit the third level's defaults then.
- **C24 Stems**: the trunk's height becomes the envelope's height times the leader extent; until
  then the trunk keeps its own height.
- **C25 A HUD over the view**, to free the panel: every control that changes the preview rather
  than the species moves onto the view. Buttons the size and shape of feathers' but translucent:
  Frame, zoom, orbit and tilt, and
  toggles for the envelope and the ground. Icons from Material Symbols, chosen by the author,
  are in `assets/icons/` (48px white PNGs, under their Material names): `center_focus_strong`
  for Frame (corner brackets: fit the subject to the view), `zoom_in` and `zoom_out`, `360` for orbit (`ImageNode::flip_x` for the other way, and
  turned 90° for tilt), `language` for the envelope, `landscape` for the ground.
  Zoom, orbit and tilt move smoothly for as long as the button is held, and a click moves a
  fixed step, which lets plans drive the camera (the driver cannot drag, and its click is a
  press and an immediate release). The triangle count in the opposite corner, lower left.
  Feathers resolves colors per surface (`SurfaceLevel`, set on a container by `ThemeContext`), so
  a HUD is one more surface: a `SurfaceLevel::Hud` with translucent fills would make plain
  feathers buttons in it translucent. Recommended: propose that upstream (any editor viewport
  has such overlays: Blender, Godot, Unity, Bevy's editor, Panoply's X3), and meanwhile register
  HUD tokens of our own in `UiTheme` with a thin button reading them, replaced by feathers
  buttons once the surface lands. Propose the layout before code.
- **C27 Remove `Placeholder`** (`src/main.rs`) once every widget kind it stands in for has a
  real customer. Numbers, spinboxes, wrapped angles, selects and subpanes have theirs in the
  species. Left: `Fuzzy` (C28; move `plans/fuzzy.py` onto a species field), and the checkbox,
  whose only use is `show_ground`, which C25 moves to the HUD; a species bool, or a test of
  the checkbox row of its own, keeps it covered. *After* C28 and C25.
- **Computed values are rounded where written**, from the first writer (Reset, Randomize, values
  derived from the generator): the number input never rounds what it shows, so round to the
  field's `Precision` in the units shown, as feathers' color input does
  (`scaled_number_input_value`). Defaults are written in the units shown
  (`15.0_f32.to_radians()`, not `0.3`).
- **A varied color** (`Fuzzy<Color>`, D10) with the leaf color, for its hue and lightness shifts.
- **C8 Working document in settings.** A settings group holding the working species and, once
  set, the file path, so a restart returns to unsaved edits and Save writes to the right file,
  as seedling did. The document's name comes from the file name. Serialized by reflection, as
  `bevy_settings` does. Can start with the placeholder.
- **C7 Undo and redo** (requirements, Authoring app). Take the `bevy_undo_redo` crate from the
  author's Bevy PR #23373 (never merged; branch `bevy_undo` in `../bevy`) as `src/undo.rs`, its
  API unchanged and its origin noted: an `UndoAction` trait (`apply` returns the inverse;
  `coalesce`, `label`, `as_any`) and an `UndoStack` resource of stacks per `ContextId`. Move it
  to its own repo when Panoply's editor needs it. An action is likely a reflection path and the
  field's old value, regenerating from the species; no record holds an `Entity`, so the PR
  review's objection (an undone despawn respawns under a new id) does not apply. Open:
  - When coalescing stops: merging by path alone joins two separate drags of one slider. Close a
    step on the widgets' `is_final`, or add the grouping the review found missing.
  - Cmd+Z in a focused text field: the text input passes it on, so the window would undo a
    document edit mid-typing. The PR's per-field contexts are for this case.
  - Keys: Cmd+Z and Cmd+Shift+Z (Ctrl+Z and Ctrl+Y elsewhere), in `src/keys.rs` (D8).
- **Leaf texture painting**, a chunk of its own, since it adds a dependency: a 2D rasterizer on
  the CPU (requirements, open question 3; the command-line mode needs the CPU path). Seedling's
  `genLeafTexture.ts` uses a small part of Canvas2D: paths of lines and cubic Béziers, stroke and
  fill, affine transforms, line width, join and cap, global alpha, solid colors and linear
  gradients, `clearRect` and `strokeRect`. The candidate must cover that set; propose it before
  code.
- **C10 Species data model**, its groups the layers of the growth model (D11;
  `docs/growth-model.md`, Consequences), correcting seedling where the requirements say.
  Reflectable types with `Default` for initial values and attributes for the editor, after the
  fields settle (C28, C29, C24), so saved files are not broken by every change. Serialized by
  reflection only, no serde derives: TOML in settings (C8) and JSON as the glTF
  extra, one representation that cannot drift. A golden test: a species serialized and compared
  with a JSON file in the repo, brought in with `include_str!` rather than quoted inline.
  Seedling's groups for reference: `trunk`;
  `branch[]`, one per level; the leaf as `leafGroup` (arrangement), `leafShape` and `leafColor`.

## Decisions

- **D1** Own repository and workspace. Bevy is main, named exactly
  `git = "https://github.com/bevyengine/bevy.git", branch = "main"`; `Cargo.lock` holds the
  commit. Cargo treats a branch and a rev as different sources, so any other form builds a
  second Bevy beside the dependencies'. Bump with the family (Panoply, the `bevy_immediate`
  fork, `bevy_action_map`, `bevy_remote_driver`), one chunk per bump (Panoply's Next item 0).
- **D2** UI: `bevy_immediate` from the fork `viridia/bevy_immediate`, branch `bevy-main`, with
  `bevy_feathers` widgets. `bevy_remote_driver` by git from `viridia/bevy_remote_driver`; plans
  follow its `docs/writing-plans.md`. Keys do not use `bevy_action_map` (D8).
- **D3** The shell comes first, with a stub in place of the generator, before the growth-model
  survey. It is the `bevy_immediate` spike for Panoply's editor (Panoply's X3), so its patterns
  (panel layout, viewport, settings, plans) are meant to be reused there.
- **D4** The requirements live here, in `docs/requirements.md`, moved from Panoply on
  2026-10-11.
- **D5** The 3D view is a camera `Viewport` that follows a UI node (`ViewportSlot`), not a
  `ViewportNode` render target. `follow_viewport_slot` runs in `PostUpdate` after UI layout and
  sets the viewport from the node's `UiGlobalTransform` and `ComputedNode::size`, both physical
  pixels, so no scale factor is applied. The UI needs its own camera: UI lays out within its
  camera's viewport, so on the 3D camera it would shrink into the view pane. A `Camera2d` with
  `IsDefaultUiCamera`, `order: 1` and `ClearColorConfig::None` draws the UI over the whole
  window. Panoply's X3 viewport takes this form. The view's node is pickable: the turntable's
  drag and scroll observers sit on it (D7), so the panel's own drags and scrolls never reach the
  camera.
- **D6** App state is saved with `bevy_settings` as `org.viridia.arbor_day`; under the driver,
  `BEVY_SETTINGS_DIR` gives each run a fresh directory, so the app has no driver-specific name.
  The panel keeps a fixed width in pixels: `Pane::size` is a flex weight, so the two panes get
  weights in pixels summing to the space they share, reset when the window resizes, and a
  finished drag's first proposed size is the panel's new width. Settings save on any `AppExit`
  (`SaveSettingsSync` in `Last`), since the driver ends a plan with one and a deferred save may
  not have run. On macOS, Cmd+Q quits the app before any of it runs, so there only the deferred
  save (half a second after each change) protects settings.
- **D7** The 3D view uses a turntable camera of our own (`src/turntable.rs`), not Bevy's
  pan-orbit camera, which rotates and pans freely: every control is one more thing to reset, so
  there is no roll or pan, and Frame resets the rest. It circles a vertical axis through the
  subject with the game's field of view (40°, Panoply's `DEFAULT_FOV`), and frames from the
  game's elevation (45°, Panoply's `elevation: PI * 0.25`), so plants are seen as players will
  see them. Spin: horizontal drag or scroll. Tilt: vertical drag (dragging down looks from above),
  5° to 85°, because the hand expects it; Frame returns it to the game's elevation, which is what
  makes the extra control acceptable. Zoom: vertical scroll, moving the focus toward the height
  under the pointer (where the pointer's ray crosses the vertical plane through the axis), clamped
  to the subject's heights, so zooming in on the crown frames the crown. Frame fits the subject's
  bounding sphere once (at start and on Frame), waiting until every subject has bounds; the camera
  does not follow the subject as it changes, so an edit shows at its true size (following it made
  a taller tree look smaller). While `keep_framed` is set the distance follows the viewport so
  the sphere last framed still fits; spinning and tilting keep it, zooming clears it. Frame
  leaves the spin alone. The view is not saved: every start frames the subject.
  A drag or scroll moves along one axis at a time: each axis's movement is summed, decayed with a
  0.15 s time constant, and each event applies along the larger sum. It starts at once and
  changes axis when the other persists. A lock chosen per gesture was tried first and felt stuck:
  scroll events carry no gesture phase, and momentum scrolling kept the gesture from ending.
  The module is shaped like `bevy_camera_controller`'s (a component, a plugin, input from picking
  events) so it can be offered upstream (X7). Framing reads the subject's world transform, so it
  runs after transform propagation (before frustum update) and sets the camera's
  `GlobalTransform` itself; run before propagation, a scale change reached the camera a frame
  late and the view jittered. The camera must therefore have no parent.
- **D9** Property editor attributes follow the author's `bevy_reactor_prop_inspect`
  (`../bevy_reactor_3/crates/bevy_reactor_prop_inspect/src/attribute.rs`), the design borrowed,
  not the code (it uses older feathers names): `ValueRange<T>`, `Precision(i32)`, `Step<T>`,
  written `#[reflect(@ValueRange::<f32>(0.0..=1.0), @Precision(2))]`, read from the type
  (`StructInfo::field_at(i).custom_attributes()`), not the value. Added: `Caption` renames a row;
  `NumberInputUnits` is itself the units attribute (`angle_degrees` shows radians as degrees);
  `Wrap` for an angle of a full turn (seedling's `axis`; its `angle`, away from the parent branch,
  clamps), which also makes the input a spinbox, since feathers clamps a drag to the soft limit
  before wrapping (X10); `Spinbox`. Every number is a feathers number input; the slider is not used. Its
  `ValueRange` is always the `HardLimit`: no value here normally sits in a range while
  occasionally leaving it (HDR illuminance would), so a soft limit is never a narrower range. It is
  the same range, and its presence only chooses the look: a fill bar dragged like a slider (the
  default, as in reactor), or with `Spinbox` none, showing the step chevrons on hover. Row
  captions use feathers' `label`, which sets its own font: plain `Text` in a row node gets the
  default font, since the panel's font does not reach it. Multiline text waits for text fields.
- **D8** Keys are handled by an observer of `FocusedInput<KeyboardInput>` on the primary window,
  not `bevy_action_map`: the keys are fixed, with no rebinding or saved bindings. Focused input
  reaches the window either way, directly when nothing has focus or bubbling up from the focused
  widget. A feathers text input stops what it consumes (plain and shifted characters, Space,
  editing keys, its own Cmd shortcuts) and passes the rest on (Escape after blurring, Enter,
  Tab, other Cmd shortcuts), so letter keys never fire while typing and Cmd+S works while a
  field has focus. Checked in `bevy_ui_widgets`' `text_input.rs` and `bevy_input_focus`.
- **D10** A varied parameter is a `Fuzzy`, after the fuzzy number: a value with a peak, bounded
  support either side, and a shape. Rejected: `Varied` (vague), "window" and "error bars" (the
  right idea, but read otherwise), `Tolerance` (one of its fields would share the name),
  `FuzzyNumber` (rules out a later `Fuzzy<Color>`). A fuzzy number's membership is not a
  probability, but nobody reading `length: Fuzzy` takes it for anything but a bounded variation.

- **D11** The growth model is parametric per level, with a crown envelope that sets limb reach
  and foliage placement, limbs bending toward directions measured against world up, and cards
  oriented apart from the branch tangent. A limb leaves its parent at an angle from the parent
  (revised in C19 from an angle from world up). The design and the survey behind it are in
  `docs/growth-model.md`.

- **D12** The species is one resource, and any change to it rebuilds the whole plant: most
  parameters change much of the tree, so finer change detection would save little. The panel
  edits a copy and writes it back with `set_if_neq`, so the rebuild runs only on a real edit.
  The leaf texture is the likely exception, being far costlier to make; it can become a second
  resource if dragging stutters. Plans read the result (`PlantStats` on the plant), since the
  driver reads components, not resources.

- **D13** An enum field is a feathers select, `bevy_immediate` having no binding for one. Each
  option row carries `OptionIndex`; an observer on the select records the picked index in a
  `PickedVariant` component, which the panel reads on its next pass and applies through reflection
  (`DynamicEnum`). A variant changed elsewhere is shown by triggering `SetSelected`. The panel may
  read only components its capability set requests, so it has its own, `CapsPanel` in
  `src/properties.rs`: `CapsUiFeathers`' list plus `CapabilityEnumSelect`. Keep the list in step
  with `CapsUiFeathers` when the fork changes it. A select rather than radio buttons: one row
  however many variants.

- **D14** Phyllotaxis is two numbers per level, limbs per node and the divergence between nodes,
  in place of a rule enum: they cover spiral, alternate, opposite and whorled
  (`docs/growth-model.md`, Limbs), and each changes one visible thing.

- **D15** Limb levels are a fixed array of three and a count of those shown, 1 to 3, not a list
  that grows and shrinks: a hidden level keeps its parameters, so hiding one and showing it again
  loses nothing, with no `enabled` flag per level. A fourth level would be twigs too thin to
  mesh. A level's parameters mean something only at its depth, so there is no reordering, and
  − and + on the last shown level's header hide it or show the next. The property editor does
  this for any array marked `ShownCount`, naming the count field; each shown item is a subpane
  beside the struct's other rows, as seedling's were.

- **D16** Random numbers come from our own SplitMix64 (`src/random.rs`), not the `rand` crate:
  `rand` reserves the right to change the values its conversions give between versions, and it
  is shared with Bevy's tree, so a Bevy bump could change every plant. Each subsystem draws from
  a stream of its own, `Rng::stream(seed, name)`, so changing one leaves the others' numbers
  alone (requirements, Parameters, determinism, variants). `Fuzzy::sample` always draws three
  numbers: the bell is their mean, uniform uses the first.

- **D17** A limb is traced in 2 cm steps, turning toward its level's bend (an angle from world
  up, keeping its compass heading) at a rate per metre, and wandering by turns drawn a metre
  apart and blended smoothly, so its shape does not depend on the step. It stops at its reach
  or where it would leave the envelope. Rings go wherever it has turned 15° since the last, and
  each carries a side transported along the limb without twisting, which sets the ring's roll
  and its children's azimuth zero. Each limb's stream forks from its parent's by its index
  (the trunk's is the seed's `branching` stream), so a limb's draws depend on where it sits,
  not on how many came before. The seed is a `Seed` in the species, saved with it; the panel
  gives it a number input without a fill and a Random button. The seed list for variants
  (requirements, a species file plus a seed list) can grow from it.

## Deferred

- **X1** The generator as its own library crate, beside the app: no Bevy dependency beyond
  `bevy_math` (its `libm` feature gives the same float results on every platform), with the
  export command as a binary of that crate, so the asset pipeline and the determinism tests
  build without the renderer. Until then the generator is `src/generator.rs`, on Bevy's math
  types with no ECS. The species types need `bevy_reflect` too, and their editor attributes
  (`ValueRange` and the rest, now in `src/properties.rs`) would move with them. *Gate:* the
  command-line export.
- **X2** A file browser, for Open and Save As: the `rfd` crate (native file dialogs), the
  author's choice. *Gate:* file handling, after C8 (the working document and its path in
  settings).
- **X3** Publishing, or reserving the name on crates.io. A crate with git dependencies cannot be
  published. *Gate:* a Bevy release with what the app uses (0.21), or the author choosing to
  reserve the name with a placeholder.
- **X4** A fast modifier for scrubbing feathers number inputs, upstream (Shift for slow is all
  there is). *Gate:* spread inputs prove slow to drag.
- **X5** A dual-thumb range slider (requirements, Authoring app, a **C** item). *Gate:* the
  center-and-spread row proves clumsy.
- **X6** A view gizmo, and a home elevation and field of view set per project, for games whose
  camera differs from Panoply's. C4 has tilt by dragging, but its home view is Panoply's.
  *Gate:* another game using the tool.
- **X7** Propose a turntable camera for `bevy_camera_controller`, with C4 as the prototype. Bevy
  has free, pan and pan-orbit controllers; the issue tracker has nothing on a turntable (searched
  2026-10-11). Pan-orbit can turn off pan and fix the up direction, but cannot lock pitch, and it
  orbits about the point under the pointer rather than a fixed vertical axis through the subject.
  *Gate:* C4 landed and used long enough to settle its controls.
- **X9** Report upstream: a feathers number input given its value twice before its text applies
  shows the text twice ("11" for 1, "0°0°" for 0°). Its observers queue select-all and insert on
  `EditableText` after comparing against `value()`, which ignores queued edits, so the second
  write is queued too, and its select-all does not cover the first insert. It happens on the
  spawn frame, where the scene's default `NumericValue` writes "0" and `bevy_immediate`'s
  `number_input` always inserts the value again. `src/properties.rs` works around it: the value,
  limits and units are inserted before the scene, so the first write is right, and the binding
  starts the frame after (`will_be_spawned`). Filed upstream as bevy#26123.
  *Gate:* a fix in the Bevy we build against; then drop the workaround.
- **X10** Feathers number input gaps, upstream. (1) Its step chevrons show on hover but nothing
  handles a click (no observer on `NumberInputDecrement` or `NumberInputIncrement`, at our commit
  and on newer main), so a press falls through to the scrubber and drags. (2) A drag clamps to
  the `SoftLimit` before applying `NumberInputWrap`, so a wrapping input with the slider look
  never wraps when dragged (typing does); skip the soft clamp when wrapping. (3) A caption of its
  own: `label_text` is the short sigil label (X, Y, Z), not a field caption. (4) A drag rounds
  to `NumberInputPrecision` in the canonical units, not the units shown: an angle with precision
  2 snaps to 0.01 rad, so a drag lands on values like 17.761973°; round through the
  `UnitsFormat` instead. Without a `NumberInputPrecision` a drag does not round at all (tilt
  drags to `86.90488°`), and a precision cannot help an angle, since `Precision(0)` snaps to whole
  radians. Filed upstream as bevy#26124 (chevrons), bevy#26125 (wrap under a soft limit),
  bevy#26126 (field captions) and bevy#26130 (rounding). *Gate:* fixes in the Bevy we build against; then wrapping inputs
  may keep the slider look, rows may use the input's own caption, and angle drags round in
  degrees.
- **X11** The spread strip's colour from the feathers theme rather than a fixed white at 80%. A
  gradient's stops take colours, not tokens, so it means reading `UiTheme` in the panel and
  redrawing on a theme change. *Gate:* a second theme, or the white reading poorly over a fill.
- **X8** Pinch to zoom on a trackpad. C4 zooms on vertical scroll only; a pinch arrives as
  `PinchGesture`, not as a pointer event, so it needs its own path to the view. *Gate:* scroll
  zoom proves awkward on a trackpad.
- **X12** Space colonization (`docs/growth-model.md`) choosing the directions of the last limb
  level only, competing for points in the envelope's shell. *Gate:* crowns show holes or read
  as too regular.
- **X13** Panoply's leaf material keeps the exported normal on back faces: `cull_mode: None`
  with `double_sided: false`, set where flora loads, since Bevy flips a double-sided material's
  back-face normal (`prepare_world_normal`) and crown shading normals would then point inward.
  An item for Panoply's ledger. *Gate:* crown softness above 0 judged worth keeping, in the
  preview and in the game.
- **X14** Feathers' color input should save its `ColorInputSettings` (the picker's mode and recent
  colors) with the app's settings, but does not: a bug upstream, being fixed by someone else.
  Nothing to do here but confirm it once fixed. *Gate:* the fix in the Bevy we build against.

## Notes

- Bevy source at the locked commit is in Cargo's git checkout,
  `~/.cargo/git/checkouts/bevy-*/c9c21c2`; the local `../bevy` is at a newer commit.
- The driver cannot drag or scroll, so the turntable's gestures (spin, tilt, zoom, the axis
  choice) are checked by hand, on a trackpad.
- The first `Cargo.lock` was seeded from `bevy_remote_driver`'s, so Bevy resolved to the
  family's commit rather than today's main. Seed a new repo in the family the same way.
