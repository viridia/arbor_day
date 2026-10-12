# arbor_day ledger

What has landed, what is next, the decisions that stand, and what is deferred and why. What the
tool must do is in [requirements.md](requirements.md). Chunks are `C<n>`, decisions `D<n>`,
deferrals `X<n>`. An id is assigned once and never reused or renumbered; Next lists chunks in the
order they are meant to land, which their ids need not follow. Panoply's ledger
(`../panoply/docs/migration.md`) is cited as "Panoply's".

## Landed

| Commit    | Change                                                                                  |
| --------- | --------------------------------------------------------------------------------------- |
| `0947b01` | Repo started: `AGENTS.md`, `CLAUDE.md`, this ledger; requirements moved from Panoply's `docs/arbor-day.md`. |
| `0947b01` | C1 Minimal shell (D5): a feathers split pane, a `bevy_immediate` panel with one slider scaling a torus, the 3D camera's viewport following the view pane; `plans/smoke.py`. Bevy at `c9c21c2b`; clippy clean; the plan passes (5 steps). |
| `d1ae223` | C2 Settings (D6): window position, size and fullscreen (after Panoply's `window_settings.rs`), and the panel's width, in `src/settings.rs`; saved on any exit, including the driver's. `plans/smoke.py` checks the panel starts at its default width; the run saved to its own `settings/`, and the developer's file was untouched. |
| `eafef5a` | C4 Turntable camera (D7), `src/turntable.rs`: spin by horizontal drag or scroll, tilt by vertical drag, zoom by vertical scroll toward the height under the pointer, a Frame button that also resets the tilt; the home elevation and field of view are the game's. Speeds tuned by hand. The test torus stands on edge so a spin shows. `plans/smoke.py` checks the camera starts framed from the front and clicks Frame (9 steps). Spin and zoom are checked by hand: the driver cannot drag or scroll. |
| `0d1aee5` | C3 Keys (D8), `src/keys.rs`: one observer on the window; F frames, + and − zoom about the focus (`=` and `_` too, for the unshifted and shifted keys), Ctrl+Q quits through the saving exit path, except on macOS, where the OS takes Cmd+Q before the app sees it. `plans/smoke.py` presses − twice and F, checking framing is let go and taken back (15 steps). By hand: typing F into a number input does not frame; clicking the view gives letter keys back. |
| `65d6413` | C5 Property editor, booleans: `src/properties.rs` walks a reflected struct's fields, a row per field named after it; `bool` gets a feathers checkbox bound through reflection, every other field a label with its name and type. A `Placeholder` resource (two `bool`s, `i32`, `f32`, an enum, a nested struct) stands in for the species; `show_ground` hides the ground. The panel edits a copy and writes back with `set_if_neq`. `plans/smoke.py` checks the labels and toggles the ground (22 steps). |
| `65d6413` | C9 Property editor, numbers (D9): `f32`, `f64`, `i32`, `i64` fields get a feathers number input beside a feathers `label`; attributes set limits, slider or spinbox look, precision, step, units and wrap; `Caption` renames a row. The test object's scale is a reflected field edited the same way, replacing C1's slider. `Placeholder` is now a component on a named entity, so plans can read it; it gained an angle (degrees, wrapping). `plans/smoke.py` types 42 into `count` (0 to 10) and reads 10 (28 steps). Worked around a duplicated first text (X9). |
| `a19ec61` | C11 The panel scrolls, as feathers' list view does: a frame with a `ScrollbarGutter` around a node with `Overflow::scroll_y()` and `ScrollArea`, and a `FeathersScrollbar` targeting it. The gutter is reserved only while the scrollbar shows, so the panel keeps its own right margin, and the gutter is exactly the scrollbar's 10 px (6 wide, 4 from the edge), the margin being the gap before it. It first did not scroll: the shell's root and the split pane had `height: 100%` but the default minimum height, a flex item's content height, so tall content stretched every node past the window (measured: 2750 px in a 1440 px window); `min_height: 0` on both fixed it. `plans/smoke.py` checks the scrollbar exists (29 steps); scrolling checked with a temporary tall node, since the placeholder fits the default window. |
| `df0b83c` | C12 Fuzzy numbers (D10), `src/fuzzy.rs`: `Fuzzy { center, spread, shape }`, `shape` a `Distribution`, uniform or bell. Its row: the center as a slider bounded by the field's `ValueRange`, the spread as a 56 px spinbox from 0 to half that range, and a plain tool button (`ButtonVariant::Plain`) whose icon shows the shape and toggles it, the icons the author's (`artwork/distributions.svg`, exported to `assets/icons/`, shown 18 px tall). Number inputs share `NumberLook` (the attributes) and `number_input` (the X9 workaround). A row's gap between widgets is 4 px, as between rows. `plans/fuzzy.py` toggles both shapes (10 steps). |
| `93b3dbb` | C13 Captions above fields: a row is a column, the caption in feathers' `label_small` (its field-caption label) 2 px above the inputs, half the 4 px between rows. A number input spans the row; a fuzzy row's center, spread and shape button share a line beneath, so the center gets about 310 px of a 320 px panel rather than 53. Checkboxes keep their caption beside them. Both plans pass unchanged (29 and 10 steps). |
| (pending) | C14 Spread strip, `src/spread_strip.rs`: a 2 px track across the full width of a fuzzy center input, a child the panel adds to the input's root (`ZIndex(1)`, `Pickable::IGNORE`), so the range maps as the fill bar's does and the strip centres on the fill edge. Lifted 2 px off the bottom edge to clear the rounded corners: a first cut insetting it 4 px each end instead sat visibly off the fill edge when the spread was small and the panel wide. A segment per piece of center ± spread within the range, clamped, or for a wrapping range brought round from the other end (two segments across the seam). Its opacity follows `Distribution::density`: flat for uniform; for bell, the mean of three uniforms, in stops every twelfth of the spread. Nodes are reinserted only when a segment changes; a spread or shape edit shows a frame late, since the strip is built inside the center input before the spread input. The colour is a fixed white at 80% (not a theme token). `Placeholder` gained `axis`, a wrapping fuzzy angle across the seam. A segment is at least 2 px wide, so a tiny spread still shows; with no spread the strip is a solid 2 px tick centred on the value, rather than vanishing. `plans/fuzzy.py` checks one segment for `length` and two for `axis`, then types a spread of 0 and checks the tick (24 steps); unit tests for the segments and the density. |

| `0e9fdef` | C6 Growth-model survey, `docs/growth-model.md`, with a screenshot of seedling's trees in Panoply; the choice is D11. Requirements: outlines become a per-species switch, shown in the preview on the bark shell and the leaf texture; leaf textures are flat colors. |
| (pending) | C15 A generated trunk (D12): `Species` (`src/species.rs`), a resource holding a `Trunk` group (height, radius, taper, sides, bend); `src/generator.rs` builds a tube of rings along an arc, a ring per 15° of turn, normals shared and leaning with the taper, closed by a fan at the top; `src/plant.rs` rebuilds the `Plant` entity's mesh in place when the species changes, so its bounds and the turntable's framing follow, and records `PlantStats`. The panel edits the trunk and shows the triangle count. The trunk replaces the test torus and `TestShape`. Unit tests for ring count, shared vertices, the base, the height and the winding; `plans/smoke.py` sets 4 sides and checks the count follows (36 steps). |
| (pending) | C16 Envelope, with enums (D13). `Species` gained `Envelope { profile, height, crown_base, radius }`, the crown base a fraction of the height; `Profile` is cone, ellipsoid, hemisphere, column or flame (Weber and Penn's, widest 70% up). `generator::envelope_outline` draws it as 8 meridians of 16 steps closed across both ends and a ring every second step, skipping what has no width; `src/plant.rs` rebuilds it on an `Envelope` entity, a line-list mesh, unlit, a `TurntableSubject` so Frame fits the crown, and records `EnvelopeStats`. Any field whose type is an enum of unit variants gets a feathers select, a row per variant named after it, which needed a capability set of the panel's own (`CapsPanel`). A hand-written Envelope subpane above Trunk. The panel now outgrows the window, so clicks scroll their node into view: the driver bumped to `ad194b2`, which does (its ledger). `plans/envelope.py` picks Cone and Column and checks the segment counts, then picks the placeholder's `mode` below the fold (17 steps); unit tests for the profiles' ends and the outline's counts. `smoke.py` (36 steps) and `fuzzy.py` (24) pass. |

## Next

File handling brings Save and Open keys (D8).

The property editor is built against a placeholder struct of a few field types, a widget kind per
chunk; the species types come later (C10). Its shape so far: the panel walks the reflected value
in its immediate-mode `construct`, rows keyed by field path (`GetPath` syntax, which C7's undo
records also use); a table from `TypeId` to widget for types with their own row (`Fuzzy`,
later colors); field attributes (`#[reflect(@…)]`, read with `get_attribute`) for limits, step,
precision, units and wrap. Every number is a feathers number input: with a `SoftLimit` it looks
and drags like a slider and still takes typing, so the slider is not used. Groups are feathers
subpanes; seedling's look is not a target. `bevy_inspector`'s `details_panel` is a reference for
walking reflected fields (`field_entries`, `FieldEdit`) but not a base: it reads no attributes,
has no per-type hook, and is retained-mode.

C5 did booleans, C9 numbers and C12 fuzzy numbers (`src/properties.rs`, attributes per D9);
C13 put captions above fields, C14 drew the spread strip. From here the generator sets the
order (`docs/growth-model.md`), each step bringing the widget kind it needs:

- **C17 Nested structs as subpanes.** The panel walks `Species`, a subpane per group, replacing
  the hand-written Envelope and Trunk subpanes.
- **C18 One level of limbs**: phyllotaxis, elevation from world up, reach to the envelope, which
  needs the envelope's radius at a height (`profile_width` scaled between crown base and top).
  The trunk keeps its own height until the Stems group, when it becomes the envelope's height
  times the leader extent.
- **C19 Limb levels as a list**: the `Vec` widget, with add and remove.

Colors when a group needs one. Notes:

  - The number input scrubs with Shift for slow and has no fast modifier (X4).
  - Digits: the number input never rounds what it shows ("don't lie to the user"), so a value
    the app computes is rounded where it is written, to the field's `Precision` in the units
    shown, as feathers' color input rounds its channels to a tenth before inserting them
    (`scaled_number_input_value`). Defaults are written in the units shown
    (`15.0_f32.to_radians()`, not `0.3`). Applies when the first such writer lands (Reset,
    Randomize, values derived from the generator).
- **C8 Working document in settings.** A settings group holding the working species and, once
  set, the file path, so a restart returns to unsaved edits and Save writes to the right file.
  Seedling did the same (`localStorage` key `sapling-doc`, written on unload: the parameter JSON,
  `name`, `filePath`); here the name comes from the file name. Serialized by reflection, as
  `bevy_settings` does, so the TOML follows the reflected types. Can start with the placeholder.
- **C7 Undo and redo** (requirements, Authoring app), after C5, since what an undo record holds
  follows the reflection design. Take the `bevy_undo_redo` crate from the author's Bevy PR
  #23373 (never merged; local branch `bevy_undo` in `../bevy`) as a module, `src/undo.rs`, with
  its API unchanged and its origin noted: an `UndoAction` trait (`apply` returns the inverse;
  `coalesce`, `label`, `as_any`) and an `UndoStack` resource of undo and redo stacks per
  `ContextId`. Move it to its own repo when Panoply's editor needs it too. An action is likely a
  reflection path and the field's old value, regenerating from the species; no record holds an
  `Entity`, so the review's objection (a despawn undone respawns under a new id, breaking stored
  references) does not arise here. Open:
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
- **C10 Species data model**, its groups the layers of the growth model (D11; sketched in
  `docs/growth-model.md`, Consequences), and seedling's is wrong where the requirements say (roll compounding through levels, card facing tied to the
  tip, growth modes that disable half the parameters, ranges as low and high). Reflectable types
  with `Default` for initial values and attributes for the editor; a
  `Fuzzy { center, spread, shape }` type for varied parameters, `shape` uniform or bell. The bell
  is the mean of three uniform samples: cheap, bell-shaped, and bounded, never past
  center ± spread, so spread means the farthest a value goes in both shapes (a true gaussian's
  tails would break the requirement that every variation is bounded). Every varied parameter
  draws three numbers whatever its shape, so toggling one shape does not shift the rest of its
  random stream. A wrapping parameter's spread can cross the seam (the spread strip draws it in
  two segments), so the generator samples such angles modulo a turn. Serialized by reflection only, no serde derives: TOML in settings
  (C8), JSON as the glTF extra, one representation that cannot drift. Seedling's groups for
  reference: `trunk`; `branch[]`, one per level; and the leaf as `leafGroup` (arrangement),
  `leafShape` and `leafColor`.

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
  to the subject's heights, so zooming in on the crown frames the crown. While `keep_framed` is
  set the camera keeps the subject's bounding sphere in view, through resizes and changes to the
  subject; spinning and tilting keep it, zooming clears it, Frame sets it again. Frame leaves the
  spin alone. The view is not saved: every start frames the subject.
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
  and foliage placement, every direction measured against world up, and cards oriented apart
  from the branch tangent. The design and the survey behind it are in `docs/growth-model.md`.

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

## Notes

- Bevy source at the locked commit is in Cargo's git checkout,
  `~/.cargo/git/checkouts/bevy-*/c9c21c2`; the local `../bevy` is at a newer commit.
- The first `Cargo.lock` was seeded from `bevy_remote_driver`'s, so Bevy resolved to the
  family's commit rather than today's main. Seed a new repo in the family the same way.
