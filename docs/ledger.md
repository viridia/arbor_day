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
| `5eca2d9` | C24 Stems (D23): count, splay and leader; the lean's heading from the seed; `plans/stems.py`. |
| `b3a1947` | C44 Spans inside the crown (D24); stems keep their length when they lean, and never turn past level; each limb level's header shows its count. |
| `e64daa1` | C46 Node bias (D25) in place of the span, and nodes moved from their even places by a constant jitter; `plans/bias.py`. |
| `705dc9a` | C51 Children placed by competition and facing, with a skew per limb (D26), in place of the divergence; `plans/facing.py`. |
| `9123c76` | C50 Wander as kinks about the course, for limbs and now the trunk (D17); lighter second and third levels by default; `plans/wander.py`. |
| (pending) | C37 Foliage placement (D27): sprig cards along the last level's twigs in the crown's shell, seedling's leaf texture as a placeholder (`assets/leaf-placeholder.png`); `plans/foliage.py`. |

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

1. **C61 Trunk as the source of truth**, since raising the crown moves the trunk with it today.
   The trunk's `height` in metres, the stem's length along its axis, scales the plant. The crown's
   top is a multiple of it (100% to 200%: near 100% for an excurrent tree, more for a decurrent
   one whose leader stops short), and its base a share of it, so raising the top leaves the base
   alone. The crown's radius stays in metres. Revises D23 (the envelope as the plant's one size)
   and D24. Ahead of C45, so the sheets sweep settled fields. E[3]
2. **C63 Branch ends**: where a stem or limb ends and what grows there. Three changes, judged
   together:
   - **Start in place of bias.** A share of the parent's part inside the crown, below which no
     nodes grow; nodes spread evenly from there to the tip, with the constant jitter. Bias
     (D25) could not push every limb near the top: 4^−bias keeps both ends, so at 100% the
     lowest node still sits halfway. A predictable range over a guessed curve, until a model is
     worth exposing (X18, X22). No end: the next rule gives the tip its length.
   - **A branch carries on as its next level.** A stem or limb with children ends at its last
     node, and one limb of the next level carries on from there along it: at angle 0, no
     skew, grown like the level's other limbs (bend, wander, reach to the envelope, children),
     its radius what the pipe model leaves (D22). So the trunk turns into a branch rather than
     ending in a post, as in decurrent trees and in an excurrent leader's last shoot, and a
     limb's tip is a shoot like any other, as in sympodial growth; the pitchfork's middle tine
     is a carrying-on limb. A fork of one, which C30 generalizes. Adds a limb for each limb with
     children (about 10 on the default trunk, 64 on the default first level).
   - **Leaves on every branch end.** Clusters grow on each limb's stretch past its last child,
     the whole limb on the last level, in the shell as now (D27 revised from the last level
     only). Stems bear none. Seedling's per-level leaf flag waits until a preset needs to turn
     this off.

   Ahead of C45, as C61 is. E[3]
3. **C45 Contact sheets**, so designs are explored in sweeps rather than one screenshot at a
   time, by the author or by an agent. A second binary (`src/bin/sheet.rs`; the modules move
   behind a `src/lib.rs` so both binaries share them) reads a species file and a sweep (seeds,
   or one field's values by reflection path), grows each plant with the generator, lays them out
   in a grid at game scale (150 to 300 px a plant, the size they are in Panoply), saves one PNG
   and exits; no panel, no driver. Species files are read by reflection, the part of C10's
   serialization this needs, pulled forward. Costs: an image is about width × height ÷ 750
   tokens, so a 3 × 3 sheet near 900 px square is about 1,000, nine designs for the price of
   one screenshot. Uses: interactive selection (a sheet of variants, the author picks, the next
   sheet varies the pick), presets (X18), tuning twigs against C37's placeholder leaves, and
   comparing card models (X24).
   Unattended exploration runs in a subagent, keeping the images out of the main conversation:
   sweeping parameters needs only writing sweep files and running the one command, so a narrow
   allowlist covers it; trying generator changes (a form, say) runs in a git worktree that is
   thrown away afterwards, the findings written up and built properly. E[3]
4. **C38 Leaf texture painting** with `tiny-skia` (D20): seedling's `genLeafTexture.ts` ported,
   its leaf shape and color parameters, and the sprig's arrangement (`leafGroup`'s fan and
   pairs; growth model, Consequences), as species fields with streams of their own
   (requirements, Parameters, determinism, variants). The texture goes on C37's cards, replacing
   the placeholder, with mipmaps, since the placeholder's 128 px shimmers at game scale. A golden
   test on its pixels compares with a tolerance, since SIMD can move a channel by a level. E[3]
5. **C39 Card shapes and facing**: crossed, cupped and creased cards chosen by weight (seedling's
   `lateralDroop` folds a card about its midrib, `axialDroop` bends its far half down); the face
   target (D27) widened from world up to a blend of up and the envelope's outward normal, with a
   spread; crown softness for the shading normal (X13), built as a weight and judged in the
   preview before it becomes an option or is fixed either way. Measure how far cards are from
   edge-on to the game's camera (45° down), to judge facing and folding by numbers as well as by
   eye. Card counts multiply with levels (the default with three: 525 cards, 1,050 triangles),
   so look at density against the budget too. E[3]
6. **C30 Forks**, Weber and Penn's splits: at a point along a limb it ends, and copies of it
   carry on, turned apart by the fork angle. The copies are the same level as the limb, not its
   children, so they reach the envelope and bear children as it would; turning both by half the
   angle reads as the symmetric Y of a child deflecting its parent. Per level: forks per limb (0
   to 2), fork at (`Fuzzy`, a fraction of the limb), fork angle (`Fuzzy`), and fork roll, where
   the Y's plane lies, from level to upright. Each copy's radius follows the pipe model (D22):
   the limb's over the exponent's root of the copies, over √2 for two at exponent 2. Stems fork
   the same way, so a decurrent tree's leader stopping short (D23) ends in a fork (growth
   model, Stems and leader). A stem's fork may lie below the crown base: the leader sets the
   axis's length, the fork where it splits. After foliage, since a crown's
   look cannot be judged without leaves. E[3]
7. **C10 Species data model**, its groups the layers of the growth model (D11;
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
8. **C40 File handling**: Save, Save As and Open, as keys (D8) and as buttons at the top of the
   panel; the file browser is X2. The seed row moves in with them, in a feathers `group`
   (`containers/group.rs`, no header), so it lines up with the subpanes' rows rather than
   standing at the panel's edge. E[3]
9. **C8 Working document in settings.** A settings group holding the working species and, once
   set, the file path, so a restart returns to unsaved edits and Save writes to the right file,
   as seedling did. The document's name comes from the file name. Serialized by reflection, as
   `bevy_settings` does. E[3]
10. **C41 glTF export** from the app: the bark and leaf meshes, the leaf texture, and the species
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

- **C60 Sprig pitch and roll**, two foliage fields. Pitch: the share of the angle between the
  twig and level that a sprig's axis turns, keeping its heading, from 0% (along the twig, as
  now) to 100% (level), as a leaf turns to the light; droop below level is the fold's (C39). A
  steep twig's sprig is then no longer seen edge-on from the game's camera. Roll spread: each
  card turns about its axis at random by up to this angle, drawn from the twig's foliage stream,
  so a plant reads from the side too, for games with a lower camera than Panoply's. E[2]
- **C62 Stems end on their course**: the trunk's kinks (D17) displace its tip like a random
  walk, since each stretch shifts the rest of the stem sideways, so a stem drifts even at no
  lean, and the lean cannot cancel it. Trace a stem as now, then shear it back by the tip's
  miss in proportion to the distance along it, turning each ring's tangent to match: the zigzag
  stays and the tip lands where splay and lean put it, so the lean is the only tilt. Stems only;
  limbs keep their drift. E[2]
- **C59 Header counts in parentheses**: "Limbs 1 (8)", not "Limbs 1 · 8", which reads as items 1
  to 8 (`properties.rs`, the header's format and `HeaderNotes`' doc comment). E[1]
- **C55 Angle by place**, seedling's `angleBias` (`MeshGenerator.ts:593`), for crowns bald on
  top: twigs radiate outward, their leaves with them (D27), and the top shell stays empty. A
  field per level, "Angle at tip", −90° to 90°: the change in angle from a node at the base of
  the parent's part inside the crown to one at its tip, scaled by the node's place, added to each
  limb's drawn angle before it is clamped, so upper limbs leave steeper. The angle's caption
  becomes "Angle at base". Every level, not only the trunk's as in seedling, so sprays narrow
  toward a limb's tip. A new field under D25, since nothing yet varies with place along the
  parent. Default 0, so species are unchanged; the first level's default chosen by eye or by the
  share of the envelope's top (outward normal within 45° of up) under cards. E[2]
- **C53 Fewer plans**, since each one launches the app: twelve become four. `smoke.py` takes
  `growth.py`'s seed checks; `widgets.py` joins `fuzzy.py`, `envelope.py` and `bark.py`;
  `levels.py` takes `foliage.py`'s checks; `shapes.py` joins the screenshot galleries (`bias.py`,
  `wander.py`, `facing.py`, `stems.py`, `limbs.py`, the foliage shots), whose checks the
  generator's unit tests already make, until C45's sheets replace it. `levels.py` failed once
  in two runs at its foliage density edit (the edit did not apply) and passed with a screenshot
  before Enter; watch for it in the merged plan. E[2]
- **C42 Computed values are rounded where written**, from the first writer (Reset, Randomize,
  values derived from the generator): the number input never rounds what it shows, so round to
  the field's `Precision` in the units shown, as feathers' color input does
  (`scaled_number_input_value`). Defaults are written in the units shown
  (`15.0_f32.to_radians()`, not `0.3`). E[2]
- **C47 A sense of scale**: metre rings on the ground disc (`src/main.rs`, `Circle::new(4.0)`),
  since Frame fits any plant to the view and hides how big it is. Seedling never zoomed, so its
  plants showed their size. E[1]

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
  (nodes, per node, facing, skew, bias), direction (angle, bend toward, bend), growth (reach,
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
- **C48 Arching stems**, for fountain shrubs: a lean past level, so a stem rises and then droops
  toward the ground, stopping above it. D24 stops the lean at level until a preset needs more
  (X18). E[2]
- **C52 Rings by visible error** (D17), an optimization, in place of placing them by turn (15°
  of curvature, 5° at a kink). A limb or stem is traced in full first, then simplified as
  Ramer, Douglas and Peucker do: from its two ends, the traced point furthest from the chord
  between them is kept if it lies further than half the tube's radius there, and each half is
  simplified the same way. A path that stays within the tube's thickness cannot be seen, so
  thick limbs keep their kinks and twigs straighten into a segment or two, where the triangles
  are. A kept point's ring faces halfway between its two chords, so the tube does not pinch;
  children attach along the kept rings, as now. One rule for curvature and kinks, so the kink
  ring's threshold goes. The tolerance is a constant, tuned against the triangle budget and
  screenshots; tied to Panoply's camera instead, the generator would depend on the game. E[2]
- **C56 A HUD over the view** (C25 split, its first part), to free the panel: controls that
  change the preview rather than the species move onto the view. The triangle count in the lower
  left; toggles for the envelope and the ground in the upper right. The ground toggle stays: a
  trunk sunk below the ground (C58) shows only with the ground hidden; or the toggle switches
  between the ground and an infinite grid. Buttons the size and shape of feathers' but
  translucent. Feathers' `SurfaceLevel` and `ButtonVariant` are exhaustive enums, so neither can
  gain a HUD case here; we need one widget, a tool button, so fork feathers' button as one
  reading HUD tokens of our own registered in `UiTheme`. The author's widget library (BYOA,
  stalled) would make this moot. Icons from Material Symbols, chosen by the author, are in
  `assets/icons/` (48px white PNGs, under their Material names): `language` for the envelope,
  `landscape` for the ground. E[3]
- **C57 Camera buttons on the HUD** (C25 split, its second part): Frame, zoom, orbit and tilt,
  with the tool button from C56. Proposed layout, after Google Maps, the author to confirm: an L
  in the lower right, orbit along the bottom (spinning is horizontal), zoom and tilt up the right
  side (vertical), Frame in the corner where they meet. Icons: `center_focus_strong` for Frame
  (corner brackets: fit the subject to the view), `zoom_in` and `zoom_out`, `360` for orbit
  (`ImageNode::flip_x` for the other way, and turned 90° for tilt). Zoom, orbit and tilt move
  smoothly for as long as a button is held, and a click moves a fixed step, which lets plans
  drive the camera (the driver cannot drag, and its click is a press and an immediate release).
  *After* C56. E[3]
- **C58 Trunk below ground**, so a tree placed on a slope still meets the ground on its low side:
  the stems start below the origin by their base radius times the tangent of the steepest slope
  a tree stands on, a constant rather than a species field. At 45° the depth is the base radius.
  *Gate:* trees on slopes in Panoply showing a gap.
- **C27 Remove `Placeholder`** (`src/main.rs`). Every widget kind it stands in for has a real
  customer in the species, but for the checkbox (`show_ground`, which C56 moves to the HUD) and
  `Wrap`, which nothing in the species uses: delete `Wrap`, and keep the checkbox row unused
  until a species field needs one. Plans read `Placeholder` because the driver reads components,
  not the species resource (D12), so the plant entity carries a copy of the `Species` as a
  component, inserted on each rebuild, and plans check species fields in its place
  (`limbs[0].angle.shape`). *After* C56. E[2]
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
- **C54 Radiating foliage** (was X20, its gate C37 passed): cards radiating from a stem's tip or
  the ground, for palms, rosettes and tufts (agave, grass, fern), with no limbs (growth model,
  How the shapes are covered). E[3]

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
  lying along their twigs (D27). A limb leaves its parent at an angle from the parent
  (revised in C19 from an angle from world up; cards revised in C37 from oriented apart from the
  branch tangent). The design and the survey behind it are in
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

- **D14** Phyllotaxis is limbs per node and a facing per level (D26), in place of a rule enum:
  competition gives the spiral, crossing pairs and alternating whorls (`docs/growth-model.md`,
  Limbs), and each field changes one visible thing. Revised in C51 from limbs per node and a
  divergence between nodes.

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

- **D17** A limb's course is traced in 2 cm steps, turning toward its level's bend (an angle
  from world up, keeping its compass heading), closing a share of the angle left over each
  crown radius (revised in C31 from a fixed turn per metre). The limb kinks every 1.5
  internodes of the level it bears (an internode is a crown radius over that level's density,
  shown or not, so showing a level leaves its parents alone; the last level's own): each
  stretch points off the course by a random angle up to the wander, about a random axis, and
  holds that offset as the course turns. Offsets are from the course, not from the stretch
  before, so a limb zigzags about a steady line, as sympodial growth does, and twigs kink finer
  than limbs. Stems kink the same way about their arc, by the trunk's own wander, every 1.5
  internodes of the first level. Revised in C50 from turns drawn a metre apart, blended
  smoothly, whose turns added up, so at high wander limbs hooked and coiled. Kinks fall on
  steps, so the shape does not depend on the step. It stops at its reach or where it would
  leave the envelope. Rings go wherever it has turned 15° since the last, or at a kink turning
  it 5° at once, so a modest wander shows; a kink's ring is turned halfway between the
  stretches meeting there so the tube does not pinch. Kinks roughly triple a tree's triangles,
  so the defaults' second and third levels are sparser (7 and 10 nodes per crown radius), which
  keeps three levels under 3,000. Each ring carries a side transported along the limb without
  twisting, which sets the ring's roll and its children's azimuth zero. Each limb's stream
  forks from its parent's by its index
  (the trunk's is the seed's `branching` stream), so a limb's draws depend on where it sits,
  not on how many came before. The seed is a `Seed` in the species, saved with it; the panel
  gives it a number input without a fill and a Random button. The seed list for variants
  (requirements, a species file plus a seed list) can grow from it.

- **D18** A level's varied fields are drawn where they belong: each node's jitter and limb count
  once per node, from the parent's layout stream; the angle, reach and skew once per limb, from
  the limb's own. Fixed streams take fork indices from the top (`u64::MAX` down), so they never
  meet a child's index. A drawn value is clamped to its field's `ValueRange`, read from the
  type by reflection (`LimbLevel::limits`), so the limits have one owner; the skew is not
  clamped, since any turn is a turn.

- **D19** Variance pays where instances are many. A value drawn once per plant is invisible
  within a tree and differs only from seed to seed, so it varies only where trees of one
  species should differ (the envelope's lean and squash, the trunk's lean direction). The
  trunk does not vary otherwise. Kept as `Fuzzy`: angle and reach per limb, the most visible
  variety; the skew is a turn either way of up to its value, per limb (D26). Not varied: bend toward and bend (wander already irregularizes),
  wander (itself the randomness), radius (the pipe model derives it, D22), sides (mesh detail),
  bias (each node's jitter varies the places, D25). Counts are fractions rounded at random
  (D21).

- **D20** Leaf textures are painted on the CPU with `tiny-skia`, a pure-Rust port of part of
  Skia (requirements, open question 3). It covers seedling's Canvas2D subset almost one to one:
  `PathBuilder` for lines and cubics, `fill_path` and `stroke_path`, `Transform`, `Stroke` for
  width, join and cap, `LinearGradient`, and paint alpha. On the CPU, so the command-line export
  (X1) can use it. Rejected: `vello` (GPU, so not for the command line), `vello_cpu` (likely
  tiny-skia's successor, but its API still moving; revisit when it settles), `raqote` (quieter),
  Cairo (a C library to build and ship), `lyon` (tessellates, does not rasterize).

- **D21** A level's nodes are a density, per crown radius of the parent, not per metre,
  as bend is measured against the crown (D17): a species scaled up keeps its form and its
  triangle count. Seedling's `interval` was absolute, as real internodes nearly are. The count
  per parent and the limbs per node are fractions rounded at random (`Rng::round`, always one
  draw), from the parent's layout stream; limbs take stream indices in order, whatever each
  node's count. The parent is measured over its part inside the crown (D24).

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

- **D23** The leader is each stem's length along its axis as a fraction of the envelope's
  height, so the envelope is the plant's one size and a fraction suits a herb as well as a
  tree. Splay and lean bend a stem without lengthening it (D24). Several stems are spaced
  evenly around the base, each leaving the ground at the splay from up and turning by the lean
  as it rises, away from the middle; each has the trunk's radius and bears the first limb level,
  its stream forked from `branching` by its index. The first stem's compass heading is drawn
  from a `trunk` stream (D16), so neighbouring trees of one species lean different ways. The
  panel group stays "Trunk".

- **D24** The crown is the author's: each setting changes one thing, and the crown is never
  resized to fit the stems. A stem keeps its length whatever its splay and lean, so leaning
  droops it, and the lean stops at level so a stem never sinks toward the ground (C48). Nodes
  lie along the part of their parent inside the crown, so the crown base alone sets where a
  stem's limbs begin; limbs lie wholly inside, so theirs is the whole limb. A stem that
  misses the crown bears no limbs, which its level's header count shows. Rejected: fitting the
  crown to the stems, so no setting could leave them outside it (C44's first form). The
  stems' length was solved to reach a height in the crown and the crown widened by their
  spread, so lean lengthened the stems and widened the crown, and raising the crown base
  lengthened splayed stems to several metres.

- **D25** A new behaviour changes what a field means, or is derived, before it adds a field: a
  level has many already. So the span's two fields become one bias, with a constant jitter
  (C46); a facing and a skew replace the divergence, competition placing the children (C51,
  D26); wander becomes kinks spaced by the internode, smooth curves being bend's (C50), rather
  than gaining a smoothness and a spacing. A level keeps 12 fields. The bias pushes even
  places toward the tip or the base by raising them to the power 4^−bias, keeping both ends and
  the order; jitter moves a node up to 30% of the gap first. Left out until a preset needs
  them: a jitter per level (a fir's even whorls), and a sharply bare stretch, such as a bare
  leader tip. Sides stay a field, being plain to read.

- **D26** Children are placed where they compete least, not by a fixed divergence: a simple
  form of space colonization's competition for light, kept to siblings on one parent. It is
  how botany explains phyllotaxis (Hofmeister's rule, that a new bud forms furthest from the
  last ones; Douady and Couder, that an inhibition fading with distance yields the golden
  angle, crossing pairs and alternating whorls), so the spiral the divergence set by hand
  comes out of the rule, and the same rule serves lateral limbs, which a divergence served
  poorly. A level's facing is one of the patterns botany names on sloping limbs (Hallé,
  Oldeman and Tomlinson), measured from the parent's upper side, the one nearest world up; no
  tree fans vertically:
  - around: competition turns each node's limbs, spaced evenly, to the least crowded place,
    and toward the outside, the more so the further the node is from the plant's axis, so
    splayed stems' children face outward, toward light;
  - sides (amphitony): limbs on alternate flanks, competition picking the less crowded flank
    for the first, so one limb a node alternates and a pair takes both;
  - up and down (epitony and hypotony): every limb on the upper side or the underside, with no
    competition.

  Each limb then turns by its own random amount up to the level's skew, so a pair is not held
  opposite and the pattern loosens. The constants (crowding fades to 0.7 a mean gap further
  along the parent, nearness is raised to the 4th power, and outward counts 1 against
  crowding's 1) were tuned in simulation: gentler crowding falls to 180° turns, sharper turns
  erratic. Rejected: children facing from the flank with a divergence of 180° (C49, never
  built), which flattened lateral limbs only when the author set it and left the trunk's
  spiral fixed; facing as a weighted preference beside competition, which up and down could
  not apply to a pair, one limb of which always faced each way; room to the envelope for
  light, which near the crown's edge turns children inward; and full space colonization
  (X12), with limbs on different parents competing, which stays deferred.

- **D27** A card is a sprig of leaves on a painted stem, lined up with a real twig, after
  seedling (`docs/growth-model.md`, Foliage): its base on the twig, its axis along it, its face
  as near world up as the axis allows. Seen edge-on, a card then loses leaves but never a twig.
  Clusters grow on the last grown level only, where twigs are, at a density per crown radius of
  twig within the shell; shell and card size are shares of the crown radius, so a scaled species
  keeps its form (D21). A cluster's place and count come from a stream of the twig's own
  (`FOLIAGE_STREAM`). Rejected: a free turn about the normal, which read as scattered leaves;
  every level bearing clusters, which put leaves on boughs and stacked counts. Seedling put one
  card at each leaf-bearing tip, scaled by the branch; spacing along the shell is ours.

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
- **X5** A dual-thumb range slider (requirements, Authoring app, a **C** item). The span, the
  case that called for it, is gone (C46). *Gate:* the
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
- **X21** Find out why `bevy_immediate`'s `.text()` on the subpane headers, called every frame
  with an unchanged string, stopped clicks on the bark's color swatch from opening its picker
  (`plans/bark.py` failed three runs in three), and report it to the fork. The headers use
  `on_hash_change_text_fn`, which writes only when the text changes, and the plan passes. Until
  then, a panel binds changing text that way. *Gate:* Panoply's editor taking the panel
  patterns (D3).
- **X22** Forms as patches, after Native Instruments' Reaktor, where an instrument is a graph of
  low-level modules (oscillators, amplifiers) and some of them are front-panel controls wired to
  their inputs. Here the engine's parameters would be the low-level inputs, and a form (X18)
  would be a saved graph: a few panel controls ("branches per limb", "droop") feeding
  operators (scale, map a range, split one value across levels) that set the engine's fields.
  Forms would then be data that authors build and share, not code. Two depths. Above the
  engine, the graph only computes the engine's fields, and its operators act on scalars (as
  macro knobs do). As the engine, the graph would replace it: generators for a limb, for
  children placed along a parent, and for leaves, as in Xfrog and SpeedTree. Its signal would be
  a curve over the parent's length (0 to 1) rather than a scalar, as Reaktor's is a function of
  time. Twig density would then be a distribution whose density is a curve, shaped by
  primitives such as a window (the part inside the crown), a push toward one end (C46's bias)
  and noise (its jitter). The hard part is that change: the engine would evaluate curves rather than read fields. Likely
  more than this tool needs. E[5] *Gate:* forms written in code (X18) prove too many or too
  rigid.
- **X23** Kinks at the nodes, the limb deflecting away from each child it gives off, as
  sympodial growth zigzags (oak, hawthorn). Nodes are laid out after a limb is traced, so this
  means laying them out first. *Gate:* C50's kinks, spaced by the internode but not placed at
  the nodes, read as too random.

- **X24** Cards that include their stem and stand in for the finest twigs, those under the
  meshed radius (D22), one card per unmeshed twig along it, rather than leaves lined up with
  meshed twigs (D27). Seen edge-on the twig vanishes, so it needs crossed cards (twice the
  triangles) or facing kept away from edge-on. *Gate:* C45, to compare the two at game scale.

## Notes

- Bevy source at the locked commit is in Cargo's git checkout,
  `~/.cargo/git/checkouts/bevy-*/c9c21c2`; the local `../bevy` is at a newer commit.
- The driver cannot drag or scroll, so the turntable's gestures (spin, tilt, zoom, the axis
  choice) are checked by hand, on a trackpad.
- The first `Cargo.lock` was seeded from `bevy_remote_driver`'s, so Bevy resolved to the
  family's commit rather than today's main. Seed a new repo in the family the same way.
