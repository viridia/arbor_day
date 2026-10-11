# arbor_day ledger

What has landed, what is next, the decisions that stand, and what is deferred and why. What the
tool must do is in [requirements.md](requirements.md). Decisions here are `D<n>`, deferrals
`X<n>`. Panoply's ledger (`../panoply/docs/migration.md`) is cited as "Panoply's".

## Landed

| Commit    | Change                                                                                  |
| --------- | --------------------------------------------------------------------------------------- |
| (pending) | Repo started: `AGENTS.md`, `CLAUDE.md`, this ledger; requirements moved from Panoply's `docs/arbor-day.md`. |
| (pending) | Minimal shell (D5): a feathers split pane, a `bevy_immediate` panel with one slider scaling a torus, the 3D camera's viewport following the view pane; `plans/smoke.py`. Bevy at `c9c21c2b`; clippy clean; the plan passes (5 steps). |

## Next

1. **Shell, continued**, a chunk each, order to be decided:
   - Settings (`bevy_settings`): window, panel width, view, open file. Plans must start from
     defaults; the driver's own Next item 4 (`run.py` sets `BEVY_SETTINGS_DIR` per run) gives
     that. `Pane::size` is a flex weight, so keeping the panel's pixel width across window
     resizes means setting the weights from pixel widths.
   - Keys (`bevy_action_map`): save, open, quit, view toggles. Unmodified letter keys would fire
     while typing in a number input; that crate's D23 answers this with a context, active while
     `InputFocus` is on an `EditableText`, that claims character keys.
   - Orbit camera: Bevy's pan-orbit camera (`pan_orbit_camera` feature, the upstreamed
     `bevy_editor_cam`), which works through picking, so drags over the panel stay there.
2. **Parameter editing by reflection.** The generator configuration is reflectable Rust structs;
   field attributes hint which widget edits each field (slider, integer input, center and
   spread, ...); the panel walks the reflected struct. Propose the design before code.
   Groups are feathers subpanes, not rows butted together, and seedling's look is not a target.
   Field kinds seedling had, which the widget hints should cover: integer (seed), enum (growth
   pattern), float, range, bool, color. Branch levels were a repeating group with add and remove
   buttons, which in Rust is a `Vec` of a level struct. Notes for the center-and-spread row
   (requirements, Authoring app):
   - The feathers slider has no thumb entity: its fill is a four-stop gradient on the slider
     node, and the fill's edge marks the value. The band can be an absolutely positioned child
     spanning center ± spread as percentages of the node (`SliderRange::thumb_position`), with
     `ZIndex(-1)` under the value text and `Pickable::IGNORE`. The headless slider maps the
     pointer across the node's full width, so band, fill and pointer agree.
   - The feathers number input scrubs with Shift for slow and has no fast modifier (X4).
   - The slider takes no typed value; a soft-limited number input does, but its fill is on an
     inner entity the band cannot attach to.
3. **Growth-model survey** (requirements, Research): a short write-up with a recommendation.
   The gate for the generator.

## Decisions

- **D1** Own repository and workspace. Bevy is main, named exactly
  `git = "https://github.com/bevyengine/bevy.git", branch = "main"`; `Cargo.lock` holds the
  commit. Cargo treats a branch and a rev as different sources, so any other form builds a
  second Bevy beside the dependencies'. Bump with the family (Panoply, the `bevy_immediate`
  fork, `bevy_action_map`, `bevy_remote_driver`), one chunk per bump (Panoply's Next item 0).
- **D2** UI: `bevy_immediate` from the fork `viridia/bevy_immediate`, branch `bevy-main`, with
  `bevy_feathers` widgets. `bevy_action_map` from its `main`. `bevy_remote_driver` by git from
  `viridia/bevy_remote_driver`; plans follow its `docs/writing-plans.md`.
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
  window. Panoply's X3 viewport takes this form.

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

## Notes

- Bevy source at the locked commit is in Cargo's git checkout,
  `~/.cargo/git/checkouts/bevy-*/c9c21c2`; the local `../bevy` is at a newer commit.
- The first `Cargo.lock` was seeded from `bevy_remote_driver`'s, so Bevy resolved to the
  family's commit rather than today's main. Seed a new repo in the family the same way.
