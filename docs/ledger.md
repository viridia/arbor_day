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
| `258fcd8` | C23 Curved limbs (D17): bend and wander per metre, and the seed; `plans/growth.py`.     |
| `7f6431b` | C28 Varied level fields (D18): span, divergence, angle and reach are `Fuzzy`.           |
| `98382d1` | C31 Bend in proportion (D17): a share of the angle left, per crown radius; the trunk's `bend` is now `lean`. |
| `e409d08` | C34 Fractions as percentages: the `ratio_percent` unit (`properties::RatioPercent`).  |
| `c040b8d` | C36 Nodes by density (D21): nodes per crown radius of span, and per node, rounded at random (`Rng::round`). |
| `1a68679` | C35 Span as middle and length (D19), in place of start and end.                       |
| `3cf3760` | C29 Pipe-model radii (D22), and limbs under the thinnest meshed radius left unmeshed; `plans/levels.py` checks the cut-off. |
| —         | C24 Stems (D23): count, splay and leader; the lean's heading from the seed; `plans/stems.py`. |

## Next

Three parts: the path to the milestone, in order; quick wins, for between big chunks; and
everything else, unordered. Each item ends with its effort, on `bevy_action_map`'s scale
(`docs/guidelines.md`, G19): E[1] one function, E[2] several, E[3] a new mechanism (a chunk),
E[4] more than a chunk (split before it is written). A new idea goes to Later unless the
milestone needs it.

The property editor gains a widget kind as a species field needs it. Rows are keyed by field
path, in `GetPath` syntax, which C7's undo records also use. Seedling's look is not a target.
`bevy_inspector`'s `details_panel` also walks reflected fields, but reads no attributes, has no
per-type hook, and is retained-mode.

### Milestone: the first species exported to Panoply

1. **C44 Targets, not filters.** A limb exists only where its node lies inside the envelope, and
   whether it does rests on settings made apart from the envelope (leader, splay, lean, span,
   crown base), so pushing any of them far enough empties the plant. Make each a target: a
   stem's length is the leader times the distance to the envelope along its starting
   direction, as reach is for limbs, so a splayed stem stops inside the crown; a level's span is
   measured over the part of the parent inside the crown, so the crown base alone sets where
   limbs begin and "0 to 100%" spans the crown whatever its shape (only stems change, since limbs
   lie wholly inside); and each limb subpane's header shows its level's count ("Limbs 1 · 8"),
   so an emptied level shows while dragging. The default span becomes about 5% to 95% of the
   crown's stretch; compare the default tree with today's. Left after this: reach or density
   near 0, which read as what they do. E[3]
2. **C37 Foliage placement** (`docs/growth-model.md`, Foliage): clusters seeded along the parts
   of the limbs within a shell depth of the envelope, at a spacing set by the author, each a
   plain quad for now, in a leaf mesh of its own with an alpha-masked material. Bare trees set
   no foliage. Facing up for now; C39 brings the facing distribution. Twigs and leaves are
   judged against each other (twigs need leaves to show they are in the right place, leaves
   need twigs to sit on), so one side is a placeholder while the other is tuned: the leaves,
   since the twigs exist. The cards carry a texture taken from a seedling `.glb` (they embed
   theirs; growth model, Seedling's flora) as a placeholder in `assets/`, until C38 paints
   them. E[3]
3. **C38 Leaf texture painting** with `tiny-skia` (D20): seedling's `genLeafTexture.ts` ported,
   its leaf shape and color parameters as species fields with streams of their own
   (requirements, Parameters, determinism, variants). The texture goes on C37's cards, replacing the placeholder. A golden
   test on its pixels compares with a tolerance, since SIMD can move a channel by a level. E[3]
4. **C39 Card shapes and facing**: crossed, cupped and creased cards chosen by weight; a card's
   normal drawn around a blend of world up and the envelope's outward normal, with a spread, its
   rotation about the normal free; crown softness for the shading normal (X13). E[3]
5. **C30 Forks**, Weber and Penn's splits: at a point along a limb it ends, and copies of it
   carry on, turned apart by the fork angle. The copies are the same level as the limb, not its
   children, so they reach the envelope and bear children as it would; turning both by half the
   angle reads as the symmetric Y of a child deflecting its parent. Per level: forks per limb (0
   to 2), fork at (`Fuzzy`, a fraction of the limb), fork angle (`Fuzzy`), and fork roll, where
   the Y's plane lies, from level to upright. Each copy's radius follows the pipe model (D22):
   the limb's over the exponent's root of the copies, over √2 for two at exponent 2. Stems fork
   the same way, so a decurrent tree's leader stopping short (D23) ends in a fork (growth
   model, Stems and leader). After foliage, since a crown's look cannot be judged without
   leaves. E[3]
6. **C10 Species data model**, its groups the layers of the growth model (D11;
   `docs/growth-model.md`, Consequences), correcting seedling where the requirements say.
   Reflectable types with `Default` for initial values and attributes for the editor, after the
   fields settle, so saved files are not broken by every change. Serialized
   by reflection only, no serde derives: TOML in settings (C8) and JSON as the glTF extra, one
   representation that cannot drift. The file records the generator version it was made with
   (X15). A golden test: a species serialized and compared with a JSON file in the repo,
   brought in with `include_str!` rather than quoted inline. The limb levels are written as the
   panel shows them, a list of the shown levels only: the count and the array (D15) move into a
   `LimbLevels { shown, levels }` type with `#[reflect(SerializeWithRegistry,
   DeserializeWithRegistry)]`, which writes the shown levels, each by reflection through the
   registry, and reads one to three, filling the rest from each depth's defaults (moved from
   `Species::default` to `LimbLevels::default`). Type data on the type, not a
   `ReflectSerializerProcessor`, since `bevy_settings` makes its own serializer. The panel shows
   `LimbLevels` flattened, its levels beside the other subpanes, by an attribute. A hidden level
   then lasts a session, not a save or a restart. Rejected: the species as a `Vec` of grown
   levels with hidden ones stashed in the editor, the same in files but a larger change that
   undoes D15. Seedling's groups for reference: `trunk`; `branch[]`, one per level; the leaf as
   `leafGroup` (arrangement), `leafShape` and `leafColor`. E[4], split when started
7. **C40 File handling**: Save, Save As and Open, as keys (D8) and as buttons at the top of the
   panel; the file browser is X2. The seed row moves in with them, in a feathers `group`
   (`containers/group.rs`, no header), so it lines up with the subpanes' rows rather than
   standing at the panel's edge. E[3]
8. **C8 Working document in settings.** A settings group holding the working species and, once
   set, the file path, so a restart returns to unsaved edits and Save writes to the right file,
   as seedling did. The document's name comes from the file name. Serialized by reflection, as
   `bevy_settings` does. E[3]
9. **C41 glTF export** from the app: the bark and leaf meshes, the leaf texture, and the species
   as a JSON extra (C10), in the form Panoply loads (requirements, Output; "What Panoply does with the output"). Bevy writes no glTF,
   so this adds a dependency or a writer of our own; propose it before code. The command-line
   export is X1. Research so far (2026-10-12), the author also asking skein's author:
   - `gltf-json` (the `gltf` crate's JSON types, ~11M downloads), with the GLB framing (header,
     JSON chunk, binary chunk) written here: every byte ours, and `extras` on every object
     (behind a feature), which the species needs. Leading candidate. The leaf texture needs a PNG
     encoder (the `png` crate; see whether Bevy already pulls it in).
   - `mesh-tools` (0.4.0, ~7k downloads): a mesh generator first, with GLB export, vertex
     colors, alpha mask and embedded images, but extras not shown; half of it duplicates ours.
   - `gltforge` (0.0.6, ~400 downloads): a reader, exporting through a Unity plugin, too young.
     Ruled out.

   The export writes `.gltf`, `.bin` and the leaf texture as a separate `.png` (requirements,
   Output); the pipeline (overrun with gltf-transform) packs, mipmaps and converts. GLB written
   here is optional. KTX2 in Bevy's glTF loader (`bevy_gltf/src/loader/mod.rs:1227`): any image
   source goes through `Image::from_buffer` by mime type or extension, so KTX2 loads with
   Bevy's `ktx2` feature, embedded or by path, and a path is loaded as its own asset, shared
   across models. But Bevy does not read `KHR_texture_basisu` (`bevy_gltf/src/lib.rs:122`,
   bevy#19104), which gltf-transform's KTX2 conversion writes; so the pipeline points the image
   straight at the `.ktx2`, which Bevy accepts though the standard does not. E[2]

### Quick wins

- **C42 Computed values are rounded where written**, from the first writer (Reset, Randomize,
  values derived from the generator): the number input never rounds what it shows, so round to
  the field's `Precision` in the units shown, as feathers' color input does
  (`scaled_number_input_value`). Defaults are written in the units shown
  (`15.0_f32.to_radians()`, not `0.3`). E[2]

### Later

- **C32 Log scale** for sizes, with no change upstream: an `@LogScale` attribute. The species
  keeps metres; the panel's binding gives the number input ln(value) and a log range, and
  writes e^value back; a registered unit (`length_log_meters`) formats e^value in metres, to
  three significant figures, and parses metres to the log. The widget has no
  `NumberInputPrecision`, which would round in log units; the binding rounds to the field's
  `Precision` in metres as it writes. Feathers computes a drag from where it started, so the
  rounding should not fight it; check by hand that a drag snaps cleanly. First for the
  envelope's height (0.5 to 30 m, where a herb lives in the first pixels) and radius. E[3]
- **C33 Sections within a level**, so the panel is not a wall of like sliders: arrangement
  (nodes, per node, divergence, span), direction (angle, bend toward, bend), growth (reach,
  wander), bark (radius, sides). A field attribute, `@Group("Direction")`, names its section;
  the property editor collates fields by group, ungrouped fields first, each group under a
  label. The grouping is the panel's alone: the data stays flat, so paths (`limbs[0].angle`),
  undo records (C7), saved files and the generator are untouched, and regrouping for looks
  breaks nothing (X15). Fields are still declared in their groups, so panel and source read in
  the same order. A sub-struct only where the group is a concept of its own: the arrangement
  is phyllotaxis, which foliage will need too, so it may become a `Phyllotaxis` type; a nested
  struct inside a level then shows as a labelled group like the rest, not a full subpane. E[3]
- **C43 A varied color** (`Fuzzy<Color>`, D10) with the leaf color, for its hue and lightness
  shifts. E[3]
- **C25 A HUD over the view**, to free the panel: every control that changes the preview rather
  than the species moves onto the view. Buttons the size and shape of feathers' but translucent:
  Frame, zoom, orbit and tilt, and toggles for the envelope and the ground. Icons from Material
  Symbols, chosen by the author, are in `assets/icons/` (48px white PNGs, under their Material
  names): `center_focus_strong` for Frame (corner brackets: fit the subject to the view),
  `zoom_in` and `zoom_out`, `360` for orbit (`ImageNode::flip_x` for the other way, and turned
  90° for tilt), `language` for the envelope, `landscape` for the ground. Zoom, orbit and tilt
  move smoothly for as long as the button is held, and a click moves a fixed step, which lets
  plans drive the camera (the driver cannot drag, and its click is a press and an immediate
  release). The triangle count in the opposite corner, lower left. Feathers resolves colors per
  surface (`SurfaceLevel`, set on a container by `ThemeContext`), so a HUD is one more surface:
  a `SurfaceLevel::Hud` with translucent fills would make plain feathers buttons in it
  translucent. Recommended: propose that upstream (any editor viewport has such overlays:
  Blender, Godot, Unity, Bevy's editor, Panoply's X3), and meanwhile register HUD tokens of our
  own in `UiTheme` with a thin button reading them, replaced by feathers buttons once the
  surface lands. Propose the layout before code. E[4], split first
- **C27 Remove `Placeholder`** (`src/main.rs`) once every widget kind it stands in for has a
  real customer. Numbers, spinboxes, wrapped angles, selects and subpanes have theirs in the
  species, and `Fuzzy` has its in the limb levels. Left: `plans/fuzzy.py` reads `Placeholder`
  (the driver reads components, not the species resource, D12), so it needs another way to
  read a fuzzy field's state; and the checkbox, whose only use is `show_ground`, which C25
  moves to the HUD; a species bool, or a test of the checkbox row of its own, keeps it
  covered. *After* C25. E[2]
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

  E[4]

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
  beside the struct's other rows, as seedling's were. Saved, only the shown levels are written
  (C10), so a hidden level is kept for the session, not across a save.

- **D16** Random numbers come from our own SplitMix64 (`src/random.rs`), not the `rand` crate:
  `rand` reserves the right to change the values its conversions give between versions, and it
  is shared with Bevy's tree, so a Bevy bump could change every plant. Each subsystem draws from
  a stream of its own, `Rng::stream(seed, name)`, so changing one leaves the others' numbers
  alone (requirements, Parameters, determinism, variants). `Fuzzy::sample` always draws three
  numbers: the bell is their mean, uniform uses the first.

- **D17** A limb is traced in 2 cm steps, turning toward its level's bend (an angle from world
  up, keeping its compass heading), closing a share of the angle left over each crown radius
  (revised in C31 from a fixed turn per metre), and wandering by turns drawn a metre
  apart and blended smoothly, so its shape does not depend on the step. It stops at its reach
  or where it would leave the envelope. Rings go wherever it has turned 15° since the last, and
  each carries a side transported along the limb without twisting, which sets the ring's roll
  and its children's azimuth zero. Each limb's stream forks from its parent's by its index
  (the trunk's is the seed's `branching` stream), so a limb's draws depend on where it sits,
  not on how many came before. The seed is a `Seed` in the species, saved with it; the panel
  gives it a number input without a fill and a Random button. The seed list for variants
  (requirements, a species file plus a seed list) can grow from it.

- **D18** A level's varied fields are drawn where they belong: the span once per parent and the
  divergence once per node, from the parent's layout stream; the angle and reach once per limb,
  from the limb's own. Fixed streams take fork indices from the top (`u64::MAX` down), so they
  never meet a child's index. A drawn value is clamped to its field's `ValueRange`, read from
  the type by reflection (`LimbLevel::limits`), so the limits have one owner; the divergence is
  not clamped, since any turn is a turn.

- **D19** Variance pays where instances are many. A value drawn once per plant is invisible
  within a tree and differs only from seed to seed, so it varies only where trees of one
  species should differ (the envelope's lean and squash, the trunk's lean direction). The
  trunk does not vary otherwise. Kept as `Fuzzy`: angle and reach per limb, divergence per node,
  the most visible variety. Not varied: bend toward and bend (wander already irregularizes),
  wander (itself the randomness), radius (the pipe model derives it, D22), sides (mesh detail). Counts are
  fractions rounded at random (D21). The span is a middle and a length, both `Fuzzy` and drawn
  per parent, its ends middle ± length/2 cut short at the parent's ends, so they cannot cross:
  two fields rather than one `Fuzzy` for the range, which would read as "a value drawn near
  here", not "this stretch".

- **D20** Leaf textures are painted on the CPU with `tiny-skia`, a pure-Rust port of part of
  Skia (requirements, open question 3). It covers seedling's Canvas2D subset almost one to one:
  `PathBuilder` for lines and cubics, `fill_path` and `stroke_path`, `Transform`, `Stroke` for
  width, join and cap, `LinearGradient`, and paint alpha. On the CPU, so the command-line export
  (X1) can use it. Rejected: `vello` (GPU, so not for the command line), `vello_cpu` (likely
  tiny-skia's successor, but its API still moving; revisit when it settles), `raqote` (quieter),
  Cairo (a C library to build and ship), `lyon` (tessellates, does not rasterize).

- **D21** A level's nodes are a density, per crown radius of the parent's span, not per metre,
  as bend is measured against the crown (D17): a species scaled up keeps its form and its
  triangle count. Seedling's `interval` was absolute, as real internodes nearly are. The count
  per parent and the limbs per node are fractions rounded at random (`Rng::round`, always one
  draw), from the parent's layout stream; limbs take stream indices in order, whatever each
  node's count. Lengthening the span below the crown base adds only nodes that are left out.

- **D22** Radii follow the pipe model top-down, from the trunk's radius, which the author sets: a
  stem's radius raised to the bark's exponent (2 to 3) is shared among the limbs leaving it and
  what carries on, each limb taking its level's `radius` share of the stem's radius where it
  leaves. A stem's top is what its limbs leave it, or its base tapered if thinner, so the taper
  is the least a stem narrows and childless limbs taper as before. Rejected: bottom-up from the
  twigs, which makes the trunk's radius follow the limb count, so showing a third level thickened
  the trunk about 2.4 times. Limbs under the bark's thinnest meshed radius are grown, for the
  foliage (C37), but not meshed, nor their own limbs, which are thinner. At 1 cm the defaults
  mesh 8, 62 of 84, and 142 of 538 limbs for one to three levels: 194, 866 and 1,862 triangles.
  Sides stay per level, for Panoply's inverted-hull outline (`src/materials/outline.wesl`,
  extruding along vertex normals): a face's middle moves out by width × cos(180° ÷ sides), so
  the line swells and thins with the view, 50% to 100% at 3 sides, 71% at 4, 87% at 6. Hence 3
  for thin levels, where it saves most and cannot be seen, and 4 or more for the trunk and the
  first level, as the defaults have it.

- **D23** The stems' length is the leader, a share of the envelope's height, so the envelope is
  the plant's one size and a fraction suits a herb as well as a tree. Several stems are spaced
  evenly around the base, each leaving the ground at the splay from up and turning by the lean
  as it rises, away from the middle; each has the trunk's radius and bears the first limb level,
  its stream forked from `branching` by its index. The first stem's compass heading is drawn
  from a `trunk` stream (D16), so neighbouring trees of one species lean different ways. A
  shrub needs a low crown base too: nodes under the crown, or outside its narrow foot, grow
  nothing. The panel group stays "Trunk". Defaults keep the 5 m trunk: an 80% leader of a
  6.25 m envelope.

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
- **X5** A dual-thumb range slider (requirements, Authoring app, a **C** item). C35's middle and
  length keep the span's ends from crossing, which was the case that called for it. *Gate:* the
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
- **X15** Keeping old trees as they were when the generator changes. The game ships exported
  meshes, so a tree changes only when regenerated: edited again, or rebuilt by the command-line
  export (X1) if the asset pipeline runs it. Rather than keeping every past generator (Cargo
  could depend on several versions under different names), three measures:
  (1) golden tests, species files with a hash of their meshes, so no output changes by
  accident; (2) a generator version in the species file, raised only when output changes on
  purpose, which the export can warn about; (3) an additive change defaults to the old output
  when the file lacks its field (no wander, say), so a new species' default and a missing
  field's default are different things. What remains (fixes, reordered draws) is re-exported
  deliberately, the earlier export being the record. *Gate:* the first species used in
  Panoply.
- **X16** Captions inside feathers' number input, upstream, by the author, for seedling's
  density: caption left and value right while idle; while editing, the caption hidden and the
  text right-aligned. The input's `label_text` would make slider widths ragged. *Gate:* the
  change in the Bevy we build against; then rows drop their caption line, halving their height.
- **X17** Offer `ratio_percent` (`properties::RatioPercent`) to feathers, which has length, time
  and angle units but no percentage. Not a `StandardUnitKind`: its scaling shows an `f32` 0.3 as
  30.000002%, so the unit shifts the value's shortest decimal instead; feathers' own units would
  gain from the same. It has settled: C24's leader took it unchanged. *Gate:* the author's
  go-ahead to file the PR.
- **X18** Forms and presets. The tool is for indie developers wanting the common Earth plant
  types (requirements, What it is for), so presets named after those types ("Fir", "Willow",
  "Live oak", "Clump"), each a saved species, are its starting points. Whether forms are needed
  as well is open: a few architectures (excurrent and decurrent, which covers clumps; seedling's
  48 used only these two, a limit of seedling's rather than of intent, growth model, Seedling's
  flora), each with a handful of outcome
  parameters it translates into the engine's ("branches per limb on average" rather than nodes
  per crown radius). A species would keep every form's parameters with a selector for the
  active one, as D15 keeps hidden levels, so switching loses nothing and the editor needs no
  widget for an enum with data; the raw engine stays a developer view. *Gate:* C44, foliage (C37
  to C39) and C30 landed, since the generator's look cannot be judged without leaves, and
  presets for the reference set tried: forms only if tuning presets is still too
  fiddly.
- **X19** Pendulous levels, for willows: a level whose limbs may hang below the envelope, the
  envelope guiding rather than clipping them (growth model, How the shapes are covered).
  *Gate:* a weeping preset (X18).
- **X20** Foliage radiating from a stem's tip or the ground, for palms, rosettes and tufts
  (agave, grass, fern), with no limbs. *Gate:* C37 (foliage placement).

## Notes

- Bevy source at the locked commit is in Cargo's git checkout,
  `~/.cargo/git/checkouts/bevy-*/c9c21c2`; the local `../bevy` is at a newer commit.
- The driver cannot drag or scroll, so the turntable's gestures (spin, tilt, zoom, the axis
  choice) are checked by hand, on a trackpad.
- The first `Cargo.lock` was seeded from `bevy_remote_driver`'s, so Bevy resolved to the
  family's commit rather than today's main. Seed a new repo in the family the same way.
