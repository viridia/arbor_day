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

## Next

File handling brings Save and Open keys (D8).

The property editor is built against a placeholder struct of a few field types, a widget kind per
chunk; the species types come later (C10). Its shape so far: the panel walks the reflected value
in its immediate-mode `construct`, rows keyed by field path (`GetPath` syntax, which C7's undo
records also use); a table from `TypeId` to widget for types with their own row (`Varied`,
later colors); field attributes (`#[reflect(@…)]`, read with `get_attribute`) for limits, step,
precision, units and wrap. Every number is a feathers number input: with a `SoftLimit` it looks
and drags like a slider and still takes typing, so the slider is not used. Groups are feathers
subpanes; seedling's look is not a target. `bevy_inspector`'s `details_panel` is a reference for
walking reflected fields (`field_entries`, `FieldEdit`) but not a base: it reads no attributes,
has no per-type hook, and is retained-mode.

- **C5 Property editor: booleans.** A placeholder resource of a few field types (a `bool` or two,
  an `i32`, an `f32`, an enum, a nested struct). The panel walks it by reflection: `bool` fields
  get a feathers checkbox, every other field a static label with its name and type. An edited
  checkbox writes through reflection. Then a chunk each, ids assigned when planned: numbers (limits,
  step, precision, units, angles that wrap); center and spread, as two number inputs; enums;
  nested structs as subpanes; `Vec`s with add and remove; colors. The C1 scale slider becomes a
  number input with the numbers chunk. Notes:
  - Angles: `NumberInputUnits` (`angle_degrees`) shows radians as degrees; an angle that is a
    full turn gets a `HardLimit` of one turn and `NumberInputWrap::Wrap` (seedling's `axis`); one
    that is not (seedling's `angle`, away from the parent branch) clamps. A wrapping center's
    spread can cross the seam, so the generator samples such angles modulo a turn.
  - The spread input's `SoftLimit` fill shows the spread against its maximum. A band on the
    center's input would need the number input to expose its inner fill entity (upstream).
  - Distribution shape: a plain (borderless) icon button beside the spread toggles uniform and
    bell, its icon showing the current shape; the author draws both icons (PNG, since feathers
    icons are images).
  - The number input scrubs with Shift for slow and has no fast modifier (X4).
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
- **C6 Growth-model survey** (requirements, Research): a short write-up with a recommendation.
  The gate for the generator.
- **C10 Species data model**, after C6, since the model follows the growth model, and seedling's
  is wrong where the requirements say (roll compounding through levels, card facing tied to the
  tip, growth modes that disable half the parameters, ranges as low and high). Reflectable types
  with `Default` for initial values and attributes for the editor; a
  `Varied { center, spread, shape }` type for varied parameters, `shape` uniform or bell. The bell
  is the mean of three uniform samples: cheap, bell-shaped, and bounded, never past
  center ± spread, so spread means the farthest a value goes in both shapes (a true gaussian's
  tails would break the requirement that every variation is bounded). Every varied parameter
  draws three numbers whatever its shape, so toggling one shape does not shift the rest of its
  random stream. Serialized by reflection only, no serde derives: TOML in settings
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
  events) so it can be offered upstream (X7).
- **D8** Keys are handled by an observer of `FocusedInput<KeyboardInput>` on the primary window,
  not `bevy_action_map`: the keys are fixed, with no rebinding or saved bindings. Focused input
  reaches the window either way, directly when nothing has focus or bubbling up from the focused
  widget. A feathers text input stops what it consumes (plain and shifted characters, Space,
  editing keys, its own Cmd shortcuts) and passes the rest on (Escape after blurring, Enter,
  Tab, other Cmd shortcuts), so letter keys never fire while typing and Cmd+S works while a
  field has focus. Checked in `bevy_ui_widgets`' `text_input.rs` and `bevy_input_focus`.

## Deferred

- **X1** The generator as its own library crate, beside the app: no Bevy dependency beyond
  `bevy_math` (its `libm` feature gives the same float results on every platform), with the
  export command as a binary of that crate, so the asset pipeline and the determinism tests
  build without the renderer. *Gate:* generator code starts.
- **X2** A file browser, for open and save-as. The crate is the author's choice. *Gate:* files
  beyond one remembered path.
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
- **X8** Pinch to zoom on a trackpad. C4 zooms on vertical scroll only; a pinch arrives as
  `PinchGesture`, not as a pointer event, so it needs its own path to the view. *Gate:* scroll
  zoom proves awkward on a trackpad.

## Notes

- Bevy source at the locked commit is in Cargo's git checkout,
  `~/.cargo/git/checkouts/bevy-*/c9c21c2`; the local `../bevy` is at a newer commit.
- The first `Cargo.lock` was seeded from `bevy_remote_driver`'s, so Bevy resolved to the
  family's commit rather than today's main. Seed a new repo in the family the same way.
