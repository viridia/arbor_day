# arbor_day ledger

What has landed, what is next, the decisions that stand, and what is deferred and why. What the
tool must do is in [requirements.md](requirements.md). Chunks are `C<n>`, decisions `D<n>`,
deferrals `X<n>`. An id is assigned once and never reused or renumbered; Next lists chunks in the
order they are meant to land, which their ids need not follow. Panoply's ledger
(`../panoply/docs/migration.md`) is cited as "Panoply's".

## Landed

| Chunk | Change                                                                                  |
| ----- | --------------------------------------------------------------------------------------- |
| —     | Repo started: `AGENTS.md`, `CLAUDE.md`, this ledger; requirements moved from Panoply.  |
| C1    | Minimal shell (D5): split pane, `bevy_immediate` panel, viewport following the view pane; `plans/smoke.py`. |
| C2    | Settings (D6), `src/settings.rs`: window state and panel width.                      |
| C4    | Turntable camera (D7), `src/turntable.rs`.                                           |
| C3    | Keys (D8), `src/keys.rs`: frame, zoom, quit.                                         |
| C5    | Property editor by reflection, `src/properties.rs`: booleans; `Placeholder` stands in for the species. |
| C9    | Numbers (D9): feathers number inputs shaped by field attributes.                     |
| C11   | The panel scrolls, as feathers' list view does.                                     |
| C12   | Fuzzy numbers (D10), `src/fuzzy.rs`: center, spread and a shape toggle; `plans/fuzzy.py`. |
| C13   | Captions above fields, so a fuzzy row's center input gets the panel's width.        |
| C14   | Spread strip, `src/spread_strip.rs`: center ± spread drawn along the center input.  |
| C6    | Growth-model survey, `docs/growth-model.md` (D11).                                   |
| C15   | A trunk generated from the species (D12): `src/species.rs`, `generator.rs`, `plant.rs`. |
| C16   | Crown envelope and enum selects (D13); `plans/envelope.py`.                         |
| C17   | Nested structs as subpanes (`properties::group`).                                   |
| C21   | Turntable feel (D7): one-shot framing, one axis per gesture.                        |
| C20   | Bark color, and feathers' color input for `Color` fields; `plans/bark.py`.          |
| C22   | Literal names in `bsn!` as `#name`.                                                 |
| C18   | One level of limbs (D14), reaching toward the envelope; `plans/limbs.py`.           |
| C19   | Up to three limb levels (D15), each at an angle from its parent; `plans/levels.py`. |
| C26   | Random streams and `Fuzzy::sample` (D16), `src/random.rs`.                          |
| C23   | Curved limbs (D17): bend and wander per metre, and the seed; `plans/growth.py`.     |
| C28   | Varied level fields (D18): span, divergence, angle and reach are `Fuzzy`.           |
| C31   | Bend in proportion (D17): a share of the angle left, per crown radius; the trunk's `bend` is now `lean`. |
| C34   | Fractions as percentages: the `ratio_percent` unit (`properties::RatioPercent`).  |
| C36   | Nodes by density (D21): nodes per crown radius of span, and per node, rounded at random (`Rng::round`). |
| C35   | Span as middle and length (D19), in place of start and end.                       |
| C29   | Pipe-model radii (D22), and limbs under the thinnest meshed radius left unmeshed; `plans/levels.py` checks the cut-off. |
| C24   | Stems (D23): count, splay and leader; the lean's heading from the seed; `plans/stems.py`. |
| C44   | Spans inside the crown (D24); stems keep their length when they lean, and never turn past level; each limb level's header shows its count. |
| C46   | Node bias (D25) in place of the span, and nodes moved from their even places by a constant jitter; `plans/bias.py`. |
| C51   | Children placed by competition and facing, with a skew per limb (D26), in place of the divergence; `plans/facing.py`. |
| C50   | Wander as kinks about the course, for limbs and now the trunk (D17); lighter second and third levels by default; `plans/wander.py`. |
| C37   | Foliage placement (D27): sprig cards along the last level's twigs in the crown's shell, seedling's leaf texture as a placeholder (`assets/leaf-placeholder.png`); `plans/foliage.py`. |
| C61   | Trunk as the source of truth (D23): the trunk's height in metres, and a "Crown" group (top and base measured against it) in place of the envelope's height and the leader. |
| C53   | Four plans in place of twelve: `smoke.py` (with the seed), `widgets.py` (fuzzy rows, selects, color), `levels.py` (with foliage), and `shapes.py`, a gallery of screenshots with no checks. C59 Header counts in parentheses ("Limbs 1 (8)"). |
| C56   | A HUD over the view (D28), `src/hud.rs`: the triangle count, and toggles for the envelope and the ground, which leaves `Placeholder`. |
| C64   | The HUD's toggles saved, as `ViewSettings` (D28). |
| C57   | Camera buttons on the HUD (D28): orbit, zoom, tilt and Frame, moving while held; the panel's "View" group goes. Needs `bevy_remote_driver`'s press and hold. |
| C27   | `Placeholder` removed, and `Wrap` with it (D9). The plant carries the species it was grown from (`GrownFrom`, D12), which plans read with the driver's `expect(…, at="limbs[0].angle")`; `PlantStats` drops the seed. The Bark group moves up under Trunk. X25 (`bevy::annotations`) and C65 (wireframe toggle) recorded. |
| C63   | Branch ends: a level's nodes from a start in place of the bias (D25); a parent ends at its last node and one limb carries it on (D30); leaves on every branch end, one at each tip (D27); every stem drawn, none narrower than the bark's detail threshold, in place of the thinnest meshed (D22). Sparser defaults (D17). |
| C52   | Hinting (D17): rings by visible error, the traced path simplified to the bark's detail threshold (Ramer–Douglas–Peucker); kinks at one spacing on every level, 0.15 crown radii on average, each stretch varied by up to 40%. A limb's radius is a share of the stem past it, so 100% is an even fork (D22); a stem keeps its radius to its first limb. The budget test allows 2,500 bark triangles for two levels and 3,500 for three. |
| C39   | Card folds: each card two by two quads, after seedling, its halves turned about the midrib by a lateral droop (negative cups) and its far half bent across the middle by an axial droop, plain angles per species, defaulting to seedling's most common pair (−16°, 17°). Every quad keeps the flat card's normal, so folds shape the outline but not the shading. The tip's card reaches back along the twig to its fold (D27). Leaves with no density are hidden rather than their mesh removed, since a `Mesh3d` inserted again was never drawn; `plans/levels.py` checks they come back. Facing went to C70, crown softness to C71, crossed cards to C72. Foliage's rows in Clusters and Card groups. `plans/cards.py`. |
| C33   | Control groups: a `ControlGroup` field attribute gathers a struct's rows under a bold heading where its first field is declared, or into a subpane of its own at the panel's top level, so panes never nest boxes; the data stays flat, so paths are unchanged within a struct. A limb level's rows in Arrangement, Direction (with wander) and Shape (reach, radius, sides). A Plant group at the top: the seed and the detail threshold, now `Species::detail`. `Bark` becomes `Wood` ("Bark color", pipe exponent). `plans/groups.py`, screenshots only. A pattern for Panoply's editor (D3). |
| C74   | Leaves at branch ends only (D27): one card at each limb's tip that bears none, and none along the twigs. A "Leaves" checkbox, the first species `bool`, replaces the cluster density; the density, the shell and `depth_in_envelope` go. No limb kinks in its last stretch under a tip's card (D17), so the card lies along the twig and its fold on the tip; a kink there had turned the fold up to 10 cm off the tip, which the cards along twigs had hidden from C39's test. The Foliage requirement is rewritten. `plans/levels.py` toggles Leaves. |
| C73   | Radii by taper (D22): every stem and limb narrows evenly by one taper, now in the Plant group, beside the bark color, in place of the trunk's taper and the pipe exponent; the Wood group goes, and a limb starts at its level's share of the parent's radius where it leaves. C68 (end taper) folded in. C30 scoped (one fork point, 2 to 4 copies), stem forks split off as C75; X32, X33 recorded. |
| C30   | Forks (D31): a limb of a level with a fork chance ends partway along, and 2 to 4 copies of it carry it on, spread around it by the fork angle and roll; a Fork group per level. `plans/shapes.py` screenshots a level Y, an upright Y and a three-way fork. |
| C76   | Bevy main bumped to `d41bb12` with the family (D1), bringing feathers' number-input chevrons, rounding in the units shown, and interior captions. Feathers' `ratio_percent` replaces ours (X17). `Precision` counts in the units shown: angles drag to a tenth of a degree, percentages to whole ones. The color picker's mode and recent colors are saved (X14), with a deferred save on change in `src/settings.rs`, since Cmd+Q skips the exit save (D6). |
| C77   | One line per row (was X34), in place of C13's stacked captions: number inputs carry their caption inside (feathers' `interior_label_text`), and enum selects sit right of theirs, as colors do. The panel is about half as tall. |
| C78   | Golden meshes (X15), `src/golden.rs`: a hash of the bark and leaf meshes of four species (the default, three levels, forks, three stems), bit for bit, failing with the new table to paste when output changes. |
| C79   | One axis type (`docs/patches.md`, The abstract plant): `Stem` and `Limb` become `Axis`, level 0 for stems and 1 to 3 for the limb levels, with an `AxisKind` (stem, lateral, carry-on, fork copy). The plant grows as one list, stems first, each axis naming its scions by index, in place of the `children`, `carried_on`, `forked` and `copy` flags; `Parent` goes. `src/generator/` in place of `generator.rs`: skeleton, placement, trace, fork, foliage, mesh, envelope, and the tests together in `tests.rs`. The envelope's preview lines are its helper (`envelope_helper`), so "outline" means only the ink outline (C66). Golden hashes unchanged. |
| C80   | Skeleton, then meshes: `grow` returns a `Skeleton` of axes and `Card`s, a leaf cluster each (an axis and a distance along it), and `plant` meshes it, the card quads now in `mesh.rs`. The Leaves switch acts on the clusters, so a bare plant has none. Rings stay simplified at trace time, since scions attach to the path as meshed (C83). Golden hashes unchanged. |
| C81   | A radius per ring (D22), written by the taper (`taper_rings`) once an axis is traced, in place of an axis's base and top radii; `tube` draws each ring at its own radius, none narrower than the detail threshold, and leans each ring's normals by the radii either side. So a twig thinner than the threshold keeps its taper until it reaches it, rather than being thickened along its length, and a limb takes its parent's radius from the parent's rings, which fixes a forked limb's limbs, which had started thinner than the limb where they leave it. New bark hashes, leaves unchanged; at game scale, screenshots show no difference. |
| C82   | Stages as functions: each level grows by `place_scions`, `trace_scions`, `carry_on`, the taper and `fork`, each over a `Level` (its parameters and what every stage reads) and the `Skeleton`, in that order, the taper before forks since copies start at the limb's radius at the fork. Placement records what the trace reads on the axis: its start ring at its own base radius, its reach, and its parent and where along it (`docs/patches.md`'s `parent`). Wander stays inside the trace. Axes are now added stage by stage, so the golden test hashed each mesh's triangles as a set, each from its least corner with its winding kept; hashes taken before the change held after it. Its work done, the golden test was then removed (X15). `trace.rs` defines tracing in its module doc. |

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

The foundation first, the skeleton (`docs/patches.md`, The abstract plant), then leaf painting.
C83 and C84 change output, each judged by screenshots. None of them touches the panel, the
species or the plans.

1. **C83 Hinting at meshing**: the skeleton keeps the full trace, simplified only when meshed
   (D17), so scions attach to the true path. E[2]
2. **C84 Wander as a lightning bolt**, in place of D17's kinks, which turn each stretch off the
   course and so walk the tip off it at random: on the default tree at the most wander (45°),
   the trunk's tip lands 0.3 to 0.9 m off its course, its direction varying by seed, and 10 to
   13% lower (4.94 m to about 4.4 m), since the zigzag spends its length sideways. A stage after
   `carry_on` and before the taper and `fork` (C82): the trace follows the course (bend) alone,
   and wander then displaces it by midpoints, between the base and the end of the course, both
   fixed: the middle of each segment moves square to it, a random amount in a random direction,
   up to half its length times the tangent of the wander, then each half likewise, down to
   segments of about one kink spacing, each split point moved a little as `KINK_JITTER` does, so
   kinks fall unevenly. So the tip stays where splay, lean and bend put it, at any wander, and the
   trunk's height is the height its tip reaches (D23, revised); splay and lean still droop a stem
   (D24). The same jaggedness at every scale; a roughness falling off at smaller scales waits for
   a preset that needs smoother limbs (D25). A branch end's straight run under its card is a
   third fixed point. Points that leave the envelope are pulled back inside. `fork` displaces its
   copies, as it traces and tapers them. Limbs no longer drift either, so compare the shapes
   screenshots at the usual wander. Less faithful to how a shoot grows, each internode turning
   from the last, but a tree steers back toward light and up, so a limb zigzags about a line it
   keeps returning to, which the lightning bolt draws: paintings, not photographs (requirements,
   What it is for; D22). D17 revised
   when it lands, with this reason; C62 (stems shorn back onto their course) goes, its drift gone.
   E[3]
3. **C38 Leaf texture painting** with `tiny-skia` (D20): seedling's leaf, as D29 has it, its
   fields in a "Sprig" and a "Leaf" group under Foliage, and the `Interval` type (D10) for its
   variations. The painter takes the outline as an
   argument, a width in texture pixels or none, which C38 fixes at a constant until C66 makes it
   the species'. The texture goes on C37's cards, replacing the placeholder, with mipmaps, since
   the placeholder's 128 px shimmers at game scale. The defaults paint seedling's leaf, so a new
   species previews well with no leaf fields touched, as the placeholder does. If they fall
   short, the placeholder stays as a choice until they do. A golden test on its pixels compares
   with a tolerance, since SIMD can move a channel by a level. The card's axial fold
   (`AXIAL_FOLD` in `src/generator/foliage.rs`, halfway) moves to where the painted sprig's fan starts, as
   seedling's did. E[3]
4. **C75 Stem forks**, forks (D31) on stems, at any point along them. A decurrent tree's
   leader stopping short (D23) already ends in a fork when the first level forks, its top
   being a limb of that level (D30), so what is left is a stem splitting below its crown. A stem's fork may lie below the crown base:
   the trunk's height sets the axis's length, the fork where it splits. A three-way fork low on
   one stem stands in for several splayed stems, so stems and splay may become a fork at the
   ground, its copies and angle (D25), in place of fork fields beside them. Under stacks (X31),
   the Fork operator on the trunk. E[3]
5. **C70 Card facing**: the face target (D27) widened from world up to a blend of up and the
   envelope's outward normal, with a spread. Measure how far cards are from edge-on to the
   game's camera (45° down), a figure in `PlantStats`, to judge facing and folds by numbers as
   well as by eye. Card counts multiply with levels, eight triangles each since C39, so look at
   density against the budget too. E[3]
6. **C66 Outlines**, a switch for the whole plant (requirements, Foliage): in the Plant group
   (C33), the switch, the bark's width as the game reads it, and the leaf stroke's in texture
   pixels (growth model, Consequences). On, the leaf texture is painted with its strokes (D29),
   the preview draws the bark with an inverted hull after Panoply's `src/materials/outline.wesl`,
   and the export writes the `"outline"` material extra (C41). Off, none of these; whether the
   translucent strokes between leaves stay is judged then. E[3]
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
   panel, above the Plant group (C33); the file browser is X2. File commands act on the active document, so that tabs (X26)
   need not change them. E[3]
9. **C8 Working document in settings.** A settings group holding the working species and, once
   set, the file path, so a restart returns to unsaved edits and Save writes to the right file,
   as seedling did. The document's name comes from the file name. Serialized by reflection, as
   `bevy_settings` does. Stored as a list of open documents holding one, and the active one's
   index, so several (X26) need no change of format. E[3]
10. **C41 glTF export** from the app: the bark and leaf meshes, the leaf texture, and the species
    as a JSON extra (C10), written from the generator's output (`generator::plant`), never from
    the preview's entities, whose leaf mesh is only hidden while none grow, in the form Panoply loads (requirements, Output; "What Panoply does with the output"). Bevy writes no glTF,
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

- **C85 One tube along a carry-on chain** (was X27, the crease at a join showing): `plant`
  follows each axis to the one carrying it on (D30) and draws the chain as one tube, sharing
  the ring at each join, so normals and the outline run unbroken; the radii already meet
  (C81). Where the sides differ, the parent's last segment is the adapter: its last ring takes
  the carry-on's sides, and a stitch beside `tube` joins N sides to M, walking both rings by
  angle, N + M triangles, never more than the 2N it replaces. Equal sides join by dropping the
  duplicate ring. Then forks (D31): the first copy continues the chain as a carry-on does,
  with the forked limb's sides, so no adapter, and the shared ring rounds its turn of half the
  fork angle into an elbow; the other copies meet the chain as laterals meet their parents. A
  fan closes another copy's start only if `plans/shapes.py`'s Y and three-way fork show its
  open base outside the chain's tube. E[2]
- **C60 Sprig pitch and roll**, two foliage fields. Pitch: the share of the angle between the
  twig and level that a sprig's axis turns, keeping its heading, from 0% (along the twig, as
  now) to 100% (level), as a leaf turns to the light; droop below level is the fold's (C39). A
  steep twig's sprig is then no longer seen edge-on from the game's camera. Roll spread: each
  card turns about its axis at random by up to this angle, drawn from a foliage stream of the twig's own (D16),
  so a plant reads from the side too, for games with a lower camera than Panoply's. E[2]
- **C55 Angle by place**, seedling's `angleBias` (`MeshGenerator.ts:593`), for crowns bald on
  top: twigs radiate outward, their leaves with them (D27), and the envelope's top stays empty. A
  field per level, "Angle at tip", −90° to 90°: the change in angle from a node at the base of
  the parent's part inside the crown to one at its tip, scaled by the node's place, added to each
  limb's drawn angle before it is clamped, so upper limbs leave steeper. The angle's caption
  becomes "Angle at base". Every level, not only the trunk's as in seedling, so sprays narrow
  toward a limb's tip. A new field under D25, since nothing yet varies with place along the
  parent. Default 0, so species are unchanged; the first level's default chosen by eye or by the
  share of the envelope's top (outward normal within 45° of up) under cards. Under stacks (X31)
  an operator, "Angle bias", rather than a field. E[2]
- **C42 Computed values are rounded where written**, from the first writer (Reset, Randomize,
  values derived from the generator): the number input never rounds what it shows, so round to
  the field's `Precision` in the units shown, as a drag does: through the unit's
  `canonical_to_display` and `display_to_canonical` (feathers' `NumberInputPrecision::round` is
  private).
  Defaults are written in the units shown
  (`15.0_f32.to_radians()`, not `0.3`). E[2]
- **C47 A sense of scale**: metre rings on the ground disc (`src/main.rs`, `Circle::new(4.0)`),
  since Frame fits any plant to the view and hides how big it is. Seedling never zoomed, so its
  plants showed their size. E[1]
- **C67 The leaf texture on the HUD** (D28), for tuning the leaf (requirements, Authoring app):
  the painted texture as an inset over a checkerboard, so its transparency shows, behind a
  toggle saved in `ViewSettings`. After C38. E[2]

### Later

- **C32 Log scale** for sizes, with no change upstream: an `@LogScale` attribute. The species
  keeps metres; the panel's binding gives the number input ln(value) and a log range, and
  writes e^value back; a registered unit (`length_log_meters`) formats e^value in metres, to
  three significant figures, and parses metres to the log. The widget has no
  `NumberInputPrecision`, which would round in log units; the binding rounds to the field's
  `Precision` in metres as it writes. Feathers computes a drag from where it started, so the
  rounding should not fight it; check by hand that a drag snaps cleanly. First for the
  trunk's height (0.1 to 30 m, where a herb lives in the first pixels) and the crown's radius.
  E[3]
- **C69 A color per limb level**, so the Limbs 1 to 3 subpanes are told apart at a glance: a
  thin stripe on each level's header or left edge, its color set on the node, since feathers'
  contextual theming cannot shade a subtree. *Gate:* C33's groups and C77's denser rows still
  leaving the levels hard to tell apart. E[1]
- **C71 Crown softness** (was part of C39): the cards' shading normal blended toward the
  envelope's outward normal by a weight (growth model, Foliage), so the crown shades as one
  mass; built as a weight and judged in the preview before it becomes an option or is fixed
  either way. Panoply's side is X13. E[2]
- **C72 Crossed cards** (was part of C39): a share of cards with a second card through the
  first along its axis, turned 90° about it, so a crown reads from any side (requirements,
  Foliage). Cupped cards are C39's negative lateral droop. *Gate:* the author wanting them
  after C70's facing. E[2]
- **C43 A varied color** (`Fuzzy<Color>`, D10) with the leaf color, for its hue and lightness
  shifts. E[3]
- **C48 Arching stems**, for fountain shrubs: a lean past level, so a stem rises and then droops
  toward the ground, stopping above it. D24 stops the lean at level until a preset needs more
  (X18). Under stacks (X31), the Weep operator on the trunk. E[2]
- **C65 Wireframe toggle on the HUD** (D28), to see where the triangles go: rings along the bark
  (D17's hinting), sides per level (D22), and each leaf card's quad, for card density and
  facing (C39). Bevy's `WireframePlugin` (`bevy_pbr`), drawing lines over the shaded mesh rather
  than in its place; it needs the `POLYGON_MODE_LINE` and `IMMEDIATES` GPU features, requested
  through `RenderPlugin`'s `WgpuSettings`, so first confirm Metal has both. A `Wireframe`
  component on the bark and leaf entities rather than the global `WireframeConfig`, so plans
  check it as they check `Visibility`. A third toggle in the upper left, saved in `ViewSettings`.
  The icon, from Material Symbols, is the author's to choose and add to `assets/icons/`. E[2]
- **C58 Trunk below ground**, so a tree placed on a slope still meets the ground on its low side:
  the stems start below the origin by their base radius times the tangent of the steepest slope
  a tree stands on, a constant rather than a species field. At 45° the depth is the base radius.
  *Gate:* trees on slopes in Panoply showing a gap.
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
  - A context per document, so several open documents (X26) each keep their own history.

  E[4]
- **C54 Radiating foliage** (was X20, its gate C37 passed): cards radiating from a stem's tip or
  the ground, for palms, rosettes and tufts (agave, grass, fern), with no limbs (growth model,
  How the shapes are covered). A kind of the plant's foliage (`docs/patches.md`, Stacks). E[3]

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
  (`StructInfo::field_at(i).custom_attributes()`), not the value. Added: `Caption` names a row,
  and every species field has one rather than showing its Rust name;
  `NumberInputUnits` is itself the units attribute (`angle_degrees` shows radians as degrees);
  `Spinbox`. A `Wrap` for an angle of a full turn went with the placeholder (C27), since every
  species angle clamps; feathers' `NumberInputWrap` remains should one be needed. A `bool` is a
  checkbox row, first used by Foliage's Leaves (C74). Every number is a feathers number input; the slider is not used. Its
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
  Where too few values are drawn for a shape to show, such as the leaves of one sprig, a varied
  parameter is an `Interval` instead: center ± spread, drawn uniformly with one draw, its row a
  `Fuzzy`'s with an empty space where the shape toggle would be, so the two rows' inputs line
  up (C38). Not in Bevy's prelude, so `bevy_math`'s
  `curve::Interval` does not clash.

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
  resource if dragging stutters. Plans read the result (`PlantStats` on the plant) and the species
  it was grown from (`GrownFrom`, a copy as a component), since the driver reads components, not
  resources; a resource is already a component in this Bevy, so `Species` cannot derive both.

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
  crown radius (revised in C31 from a fixed turn per metre). The limb kinks every 0.15 crown
  radii on average, on every level and on stems alike, each stretch longer or shorter by up to
  40%, drawn from the wander's stream, so kinks fall unevenly but never run together: each
  stretch points off the course by a random angle up to the wander, about a random axis, and
  holds that offset as the course turns.
  Offsets are from the course, not from the stretch before, so a limb zigzags about a steady
  line, as sympodial growth does. A zigzag reads by its period, so a first-level limb kinks
  four to six times and a twig once or not at all. Revised in C52 from 1.5 internodes of the
  level a limb bears, which tied the wander to a density: with sparser levels it vanished.
  Revised in C50 from turns drawn a metre apart, blended smoothly, whose turns added up, so at
  high wander limbs hooked and coiled. Kinks fall on steps, so the shape does not depend on the
  step. It stops at its reach or where it would leave the envelope. No kink falls in a limb's
  last stretch, as long as a tip's card reaches back from its fold (D27), so a branch end runs
  straight under its card; a limb that leaves the envelope early is traced again to where it
  stopped, so this holds wherever it ends (C74). Every limb, since which bear none is known only
  once their children grow; a parent's end is cut at its last node anyway (D30).

  Rings are hinted, as font hinting keeps only what a pixel shows (C52): the traced path is
  simplified as Ramer, Douglas and Peucker do, keeping a point only where it lies further than
  the bark's detail threshold (D22) from the chord between the points kept either side. The eye
  reads a branch by where it goes and by its corners more than by gentle curvature, so kinks
  that move the line by a pixel stay, and bends and twigs' sub-pixel zigzags straighten. Each
  kept ring faces halfway between the chords meeting there, so the tube does not pinch; the
  start ring stays as given, so a limb carrying its parent on still meets it (D30), and the tip
  heads as the trace did. A kept point less than a step before the tip goes, since a tube
  turning between two rings that close folds over itself where it is drawn thicker than the
  gap. Revised in C52 from a ring at every 15° of turn and every kink of 5°, the same however
  the tree is seen. The defaults draw 448, 1,252 and 3,106 bark triangles for one to three
  levels (second and third at 3.5 and 4 nodes per crown radius, revised in C63 from 7 and 10,
  the author judging the leaves too dense). Most trees are meant to have two levels; three are
  the costly exception, their twig count the cost (X24). An author tunes branching toward a
  leaf density, and makes a crown denser with larger leaves rather than more branches, so costs
  are judged at the densities that give it. Each ring carries a side transported along the
  limb without twisting, which sets the ring's roll and its children's azimuth zero. Each limb's stream
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
  wander (itself the randomness), radius (a share of the parent's, D22), sides (mesh detail),
  start (each node's jitter varies the places, D25). Counts are fractions rounded at random
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

- **D22** Radii by taper, top-down from the trunk's radius, which the author sets: every stem and
  limb narrows evenly by the plant's one taper, from its base to its tip as grown, and a limb's
  base radius is its level's `radius` share of the parent's radius where it leaves. A parent cut
  at its last node (D30) keeps the radius it has there, and the limb carrying it on starts at
  it, so a carried-on chain narrows step by step. So each parameter changes one visible thing:
  a level's radius thickens that level alone (requirements). One taper for all wood, as
  seedling had, which served well. Revised in C73 from the pipe model, a stem's radius raised
  to an exponent (2 to 3) shared among the limbs leaving it and what carried on: raising a
  level's radius thinned the trunk past each node and every later limb, a stem kept its base
  radius to its first limb, and the taper was only the least a stem narrowed. Lost until the
  pipe model returns as a stack operator (X31): the trunk narrowing more where many limbs leave
  it. A taper per level is X32. Rejected:
  bottom-up from the twigs, which makes the trunk's radius follow the limb count, so showing a
  third level thickened the trunk about 2.4 times.
  Every stem is drawn, none narrower than the bark's detail threshold (2 cm by default, about a
  pixel at the game's scale), as font hinting keeps a stem a pixel wide: the aim is
  verisimilitude, not realism. Only the drawing thickens; the radii stay true. Revised in C63
  from a cutoff, below which limbs were grown for the foliage but not meshed:
  their leaves floated off visible tips, and visible twigs with thinner children went bare. The
  threshold is also the rings' tolerance (D17), one setting for what the game can show.
  Sides stay per level, for Panoply's inverted-hull outline (`src/materials/outline.wesl`,
  extruding along vertex normals): a face's middle moves out by width × cos(180° ÷ sides), so
  the line swells and thins with the view, 50% to 100% at 3 sides, 71% at 4, 87% at 6. Hence 3
  for thin levels, where it saves most and cannot be seen, and 4 or more for the trunk and the
  first level, as the defaults have it.

- **D23** The trunk's height, each stem's length along its axis in metres, is the plant's one
  size, and the crown is measured against it: its top a multiple, its base a share, so a ratio
  suits a herb as well as a tree, and moving the crown never moves the trunk. Revised in C61
  from the stems' length as a share of the envelope's height, where raising the crown dragged
  the trunk with it. Splay and lean bend a stem without lengthening it (D24). Several stems are spaced
  evenly around the base, each leaving the ground at the splay from up and turning by the lean
  as it rises, away from the middle; each has the trunk's radius and bears the first limb level,
  its stream forked from `branching` by its index. The first stem's compass heading is drawn
  from a `trunk` stream (D16), so neighbouring trees of one species lean different ways. The
  panel group stays "Trunk", ahead of the "Crown" group measured against it.

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
  level has many already. So the span's two fields become one start, with a constant jitter
  (C46, C63); a facing and a skew replace the divergence, competition placing the children
  (C51, D26); wander becomes kinks at a fixed spacing, smooth curves being bend's (C50, C52),
  rather than gaining a smoothness and a spacing. A level keeps 12 fields. The start is a share
  of the parent's part inside the crown below which no nodes grow; nodes spread evenly from
  there to the tip, jitter moving each up to 30% of the gap. No end: the limb carrying the
  parent on (D30) gives the tip its length. Revised in C63 from a bias, which raised even
  places to the power 4^−bias: it kept both ends, so at 100% the lowest node still sat halfway.
  A predictable range over a guessed curve, until a model is worth exposing (X18, X22). Left
  out until a preset needs them: a jitter per level (a fir's even whorls), and a sharply bare
  stretch, such as a bare leader tip. Sides stay a field, being plain to read.

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
  One cluster grows at the tip of every branch end, the limbs that bear none, whatever their
  level, so no visible end is bare; a "Leaves" switch turns them all off for a bare plant. Card
  size is a share of the crown radius, so a scaled species keeps its form (D21). Stems bear none.
  Denser foliage comes from another limb level. Revised in C74 from more clusters at a density
  along each twig within a shell of the envelope: any density put cards along twigs, none made
  the plant bare, and those near a tip ran past the tip's card (X30). Revised in C63 from the
  last grown level only, which left an earlier level's childless limbs bare. Seedling's
  per-level leaf flag waits until a preset needs to turn leaves off a level. Rejected: a free
  turn about the normal, which read as scattered leaves; every limb bearing clusters, which put
  leaves on boughs and stacked counts. The tip's card reaches back along the twig until its
  axial fold lies at the tip, as seedling's did, so the sprig's stem overlaps the twig's end; on
  a twig shorter than that, it starts at the twig's base (C39). The twig runs straight under it
  (D17), so the card lies along the twig.

- **D28** The HUD (`src/hud.rs`) holds controls that change the preview rather than the species,
  over the view: a grid filling the view pane with the panel's spacing (8 px padding and gaps),
  ignored by picking, so the view's drags and scrolls pass between its buttons. Toggles in the
  upper left; the camera's buttons in the upper right, an L with orbit along the top, zoom and
  tilt down the side and Frame in the corner, after Google Maps' controls turned upside down;
  the bottom row for status, the triangle count at its left. A camera button moves the camera
  smoothly from the moment it is pressed until it is released (60° a second of orbit, 30° of
  tilt, a factor of 2 in distance), and Frame acts on a click. A step on the press, then smooth
  movement after a delay, was tried and felt wrong. Plans hold a button with the driver's
  `click(…, hold=frames)`. Its buttons are
  our own, a `bevy_ui_widgets` `Button` the size of feathers' tool button, with fixed
  translucent grays rather than theme tokens: the theme is for a light and dark switch, which a
  HUD over the scene does not follow, so forking feathers' button and registering tokens would
  buy nothing. A toggle's state is a field of `ViewSettings` (D6), so it is saved, as the panel
  width is and the camera is not (D7); the target's `Visibility` follows it, and plans check
  that. The icon dims while the target is hidden. `Button` stops presses and
  clicks but not drags, so the grid stops `PointerDrag` bubbling up from a button, or a drag
  that starts on one would turn the camera. Icons are Material Symbols, chosen by the author,
  in `assets/icons/` (48 px white PNGs, under their Material names). The ground toggle outlives
  the placeholder it came from: a trunk sunk below the ground (C58) shows only with the ground
  hidden.

- **D29** The leaf texture is seedling's (`genLeafShape.ts`, `genLeafTexture.ts`, and
  `createLeafStamps` in `MeshGenerator.ts`), the look the author tuned, with five changes.
  Kept: a leaf is a cubic Bézier half-outline, mirrored, split into segments; each segment's end
  is pushed outward (serration) and toward the tip (rake), with jitter, and a straight line back
  to the curve makes the tooth; serration also sharpens the tip. A sprig is a fan (count, angle,
  a size taper per step) and pairs down the stem at the fan's angle. Each leaf is filled with
  an inner and an outer color, in a lateral or axial gradient in its own frame, shifted in hue
  and lightness at random per leaf. Every leaf is stroked black first, then each is filled and
  stroked translucent dark, so the black shows only around the cluster and the leaves within it
  are parted by faint lines. The texture is square, fitted to the sprig. Changed:
  - Colors are painted as picked. Seedling converted them to linear and painted those values as
    sRGB (three.js r136's `getStyle` does not convert back), so leaves came out far darker than
    their swatches: poplar-green's outer `rgb(80,201,80)` painted near `rgb(20,149,20)`. Colors
    were tuned by eye on the preview, so seedling's stored values are not targets.
  - Lengths are fractions of the leaf's length, which is not a field. The sprig is scaled to fit
    the texture, so seedling's length mattered only against constants in absolute units (tooth
    depth 40 × serration, jitter ±10, outline 2, spacing), and changing it changed the teeth and
    the outline's weight. The outline's width is in texture pixels.
  - A painted stem, which seedling coded but left off: a midrib in the bark color, outlined, from
    the texture's bottom edge (the card's base on the twig, D27) through the pairs to the fan.
    Its width is a constant.
  - The angle and spacing variations are `Interval`s (D10). The color shift per leaf is in Oklch
    rather than HSL, so a step of lightness looks alike on every hue.
  - Shape, arrangement and color draw from streams of their own (D16); seedling's one stream
    reshuffled the leaf on any edit.

  The fan's angle still sets the pairs', until a preset needs them apart. The droops shape the
  card, not the texture (C39). Seedling's 47 models used every field, jitter least (3).

- **D30** A stem or limb bearing limbs ends at its last node, and one more limb of its
  children's level carries it on from there: along it (angle 0, no skew), otherwise grown as
  its siblings are (bend, wander, reach to the envelope, its own limbs). So the trunk turns
  into a branch rather than ending in a post, as in decurrent trees and an excurrent leader's
  last shoot, and a limb's tip is a shoot like any other, as in sympodial growth; a fork of
  one, which C30 generalizes. It draws from a stream of its own (`CARRY_ON_STREAM`, D18), so
  its siblings' count leaves it alone. The two are separate tubes meeting cleanly: the parent's
  last ring is the new limb's first, at the parent's top radius (D22), and the parent is left
  open there. Where their sides differ the silhouette shows a shoulder, and the normals crease
  (C85). If the new limb would be no longer than it is thick, the parent keeps its full length.

- **D31** Forks, after Weber and Penn's splits: a limb forks with its level's chance at one
  point, a share of its length, and ends there, capped, at its radius there (D22); copies of
  it carry it on, the same level as it, not its children, so they reach the envelope and bear
  limbs as it would. They are spread evenly around its direction, each turned half the fork
  angle from it, the first toward its flank turned up by the roll, so two make a Y from level
  (0°) to upright (90°), and three or four a ring, as a shrub's low split. Each copy starts at
  the limb's radius at the fork and draws its own reach, wander and limbs from a stream of the
  fork's (`FORK_STREAM`, D18). A forked limb bears limbs up to the fork and neither a limb
  carrying it on (D30) nor a leaf there; a copy's nodes start from its base, the level's start
  being a limb's bare base, and copies do not fork again. The limb carrying a parent on is a
  limb of its level, so it forks too: a trunk's top forks when its first level does. A copy's
  limbs compete among themselves alone (D26; X33). Five fields in a Fork group, chance, at,
  copies (2 to 4), angle and roll, all inert at the default chance of 0.

## Deferred

- **X1** The generator as its own library crate, beside the app: no Bevy dependency beyond
  `bevy_math` (its `libm` feature gives the same float results on every platform), with the
  export command as a binary of that crate, so the asset pipeline and the determinism tests
  build without the renderer. Until then the generator is `src/generator/`, on Bevy's math
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
- **X10** A feathers number input's drag clamps to the `SoftLimit` before applying
  `NumberInputWrap`, so a wrapping input with the slider look never wraps when dragged (typing
  does); skip the soft clamp when wrapping. Filed upstream as bevy#26125. The other gaps filed
  with it (chevrons, captions, rounding) are fixed (C76). *Gate:* a wrapping field, which no
  species has (D9).
- **X11** The spread strip's colour from the feathers theme rather than a fixed white at 80%. A
  gradient's stops take colours, not tokens, so it means reading `UiTheme` in the panel and
  redrawing on a theme change. *Gate:* a second theme, or the white reading poorly over a fill.
- **X8** Pinch to zoom on a trackpad. C4 zooms on vertical scroll only; a pinch arrives as
  `PinchGesture`, not as a pointer event, so it needs its own path to the view. *Gate:* scroll
  zoom proves awkward on a trackpad.
- **X12** Space colonization (`docs/growth-model.md`) choosing the directions of the last limb
  level only, competing for points in the envelope's shell. Under stacks (X31), a mode of
  Scions or an operator excluding it. *Gate:* crowns show holes or read as too regular.
- **X13** Panoply's leaf material keeps the exported normal on back faces: `cull_mode: None`
  with `double_sided: false`, set where flora loads, since Bevy flips a double-sided material's
  back-face normal (`prepare_world_normal`) and crown shading normals would then point inward.
  An item for Panoply's ledger. *Gate:* crown softness above 0 judged worth keeping, in the
  preview and in the game.
- **X15** Keeping old trees as they were when the generator changes. The game ships exported
  meshes, so a tree changes only when regenerated: edited again, or rebuilt by the command-line
  export (X1) if the asset pipeline runs it. Rather than keeping every past generator (Cargo
  could depend on several versions under different names), three measures:
  (1) golden tests, species files with a hash of their meshes, so no output changes by
  accident, once C10 can save species files; C78's hash of four species built in code, in git
  history, served the skeleton's refactors (C79 to C82), each mesh's triangles hashed as a set;
  (2) a generator version in the species file, raised only when output changes on
  purpose, which the export can warn about; (3) an additive change defaults to the old output
  when the file lacks its field (no wander, say), so a new species' default and a missing
  field's default are different things. What remains (fixes, reordered draws) is re-exported
  deliberately, the earlier export being the record. *Gate:* the first species used in
  Panoply.
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
  envelope guiding rather than clipping them (growth model, How the shapes are covered). Under
  stacks (X31), the Weep operator. *Gate:* a weeping preset (X18).
- **X21** Find out why `bevy_immediate`'s `.text()` on the subpane headers, called every frame
  with an unchanged string, stopped clicks on the bark's color swatch from opening its picker
  (`plans/bark.py` failed three runs in three), and report it to the fork. The headers use
  `on_hash_change_text_fn`, which writes only when the text changes, and the plan passes. Until
  then, a panel binds changing text that way. *Gate:* Panoply's editor taking the panel
  patterns (D3).
- **X22** Forms as patches (`docs/patches.md`), after Reaktor's front panels and Blender's
  geometry nodes: a file of operators over an abstract plant (axes and the attachments placing
  them, so the trunk and limbs share every operator on composition), each input a formula over
  the author's named controls, evaluated per plant or per element, never per vertex; a Rust
  library realizes the result. Today's species becomes the default patch. After stacks (X31),
  the generator split into operators and realization, in Rust, with no language. E[4],
  split as the doc's order of work. *Gate:* the milestone's first species exported, and C10's
  file leaving room for a species to name its patch.
- **X23** Kinks at the nodes, the limb deflecting away from each child it gives off, as
  sympodial growth zigzags (oak, hawthorn). Nodes are laid out after a limb is traced, so this
  means laying them out first. Under stacks (X31), an option of Wander. *Gate:* D17's kinks,
  evenly spaced but not placed at the nodes, read as too random.

- **X24** Cards that include their stem and stand in for the finest twigs, one card per twig
  of the last level in place of its tube, rather than leaves lined up with meshed twigs (D27),
  saving the triangles those twigs cost now that every stem is drawn (D22). Seen edge-on the twig vanishes, so it needs crossed cards (twice the
  triangles) or facing kept away from edge-on. *Gate:* X28, to compare the two at game scale.

- **X25** The property editor's attributes from `bevy::annotations` (bevy#26164, merged as `8550394`): its
  `ValueRange` (an `f64` range, read with `ValueRange::from_attributes`), `Step` and
  `DisplayName` in place of ours (`ValueRange<T>`, `Step<T>`, `Caption`), so a species reads
  the same in any Bevy inspector; `LimbLevel::limits` (D18) reads through `from_attributes`.
  `Tooltip` for the species' doc comments. Ours stay where it has none: `Precision` and
  `Spinbox` (its follow-ups), `ShownCount`, and units, since its `Unit` is a label while ours
  names a feathers units format that also scales and parses. *Gate:* C77 committed; then the
  family bumped to `8550394` in the same chunk.
- **X26** Several open documents, as tabs above the panel (a feathers tab list), for an artist
  working on variants of one species. Each document an entity holding its species, file path
  and name, the species no longer one resource (D12). A new document is "Untitled" until its
  first save names it after the file. There is one plant, rebuilt from the active document's
  species on a switch, as on any edit (D12): hiding inactive plants instead would keep their
  meshes and textures in memory, and every system that assumes one plant (framing, the HUD's
  toggles, plans selecting `Plant`) would need to skip them. If painting the leaf texture (C38)
  makes switching stutter, cache the texture per document. Until then, nothing may assume
  there is only one document: C8 stores a list, C40 acts on the active document, C7 keeps a
  history per document. Plans that write the species past the panel (the driver's
  `set_resource`) move to `set` on the active document's entity. Open: closing a tab with unsaved changes, whether a new document starts
  from defaults or a copy of the active one, and which
  need tabs serve that X28's sheets would not (editing variants one at a time, against comparing
  them side by side). *Gate:* C8 and C40 landed, and an author working on variants.

- **X28** Contact sheets (was C45), so designs are explored in sweeps rather than one
  screenshot at a time, by the author or by an agent. A second binary (`src/bin/sheet.rs`; the
  modules move behind a `src/lib.rs` so both binaries share them) reads a species file and a
  sweep (seeds, or one field's values by reflection path), grows each plant with the generator,
  lays them out in a grid at game scale (150 to 300 px a plant, the size they are in Panoply),
  saves one PNG and exits; no panel, no driver. Species files are read by reflection, the part
  of C10's serialization this needs, pulled forward. Costs: an image is about width × height ÷
  750 tokens, so a 3 × 3 sheet near 900 px square is about 1,000, nine designs for the price of
  one screenshot. Uses: interactive selection (a sheet of variants, the author picks, the next
  sheet varies the pick), presets (X18), and comparing card models (X24). The sheets would
  replace `plans/shapes.py`'s gallery. Unattended exploration runs in a subagent, keeping the
  images out of the main conversation: sweeping parameters needs only writing sweep files and
  running the one command, so a narrow allowlist covers it; trying generator changes (a form,
  say) runs in a git worktree that is thrown away afterwards, the findings written up and built
  properly. Deferred as a large, disruptive change (the crate split) whose payoff is unproven.
  E[3] *Gate:* tuning in the editor proving too slow to compare designs at game scale, or
  presets (X18) begun.

- **X29** `plans/levels.py` failed once of two runs (2026-10-14) at the second level's density
  typed as 1.5: the plant kept the default's 70 limbs, as if the text never applied. *Gate:* a
  second failure; then find whether the click lands before the row is ready, or the typing is
  lost.

- **X30** Cards along a twig, for plants whose leaves crowd their shoots: spur-bearing trees
  (apple, larch, ginkgo) and conifers' needles (growth model, How the shapes are covered: "clusters
  along the limbs"). "Cards along the twig", default 0 with a small maximum, never past the tip's
  card, with no shell. Removed in C74, where they came from the shell (C37). Interior leaves
  that stand apart from any stem would want a texture of their own, without the painted stem
  (requirements, Foliage: more than one leaf texture region), and a second texture's fields
  would crowd the panel, so they wait for stacks (X31). *Gate:* a preset
  (X18) that another limb level cannot make dense enough.

- **X31** Operator stacks (`docs/patches.md`, Stacks), the stage between the fixed panel and
  formulas (X22): each level, the trunk included, a fixed frame and optional operators in a
  fixed order (bend, wander, fork, scions and more; the doc lists them), added from a
  menu on the level's header and removed by ✕. Scions add the next level, replacing the levels' − and +
  (D15). Fewer rows, since a missing operator shows none. Each operator a `Toggle<T>` field,
  its settings and whether it is on, so ✕ keeps its values for the session (D15). E[4], split: the operator group and add menu
  with bend and wander; scions; angle bias (C55). The default species should grow as before, so
  bring back C78's golden meshes (git history) for the first part. *Gate:* the skeleton (C79 to
  C84) landed, and settled before C10's file format.

- **X32** A taper per limb level, in place of the plant's one (D22); under stacks (X31), the
  Taper operator. *Gate:* a preset (X18) whose
  twigs taper unlike its limbs.

- **X33** A fork's copies (C30) competing as one parent for their children's places (D26), so
  each copy's first child goes to its outer flank, away from the crotch; alternation still puts
  later ones inside, as in real amphitonic forks. E[2] *Gate:* C30's screenshot of a level-roll
  Y with sides facing shows the crotch crowded.

- **X35** Report upstream: a feathers number input left without a change (Enter, or focus lost)
  keeps its editing text, so an angle shows `20d` rather than `20°`, beside its interior caption
  (C77). Its text is rewritten only when `NumericValue` changes; leaving edit mode should
  rewrite it in the display form. *Gate:* a fix in the Bevy we build against.

- **X36** The trunk's lean split in two, revising D23: a tilt where the stem leaves the ground,
  in the trunk's frame beside the splay, and the curve after it from Bend (`docs/patches.md`,
  Stacks), which needs the heading the tilt gives. Bend toward up then gives a trunk leaning at
  its base and recovering, as on slopes; past the tilt, today's bowing; past level, Weep (C48).
  The heading may be fixed rather than drawn from the seed, since Panoply turns placed
  instances anyway. Lean's use so far, steering back a stem the wander sent off course, goes
  with C84, whose wander leaves the tip on its course. *Gate:* C84 landed, and an author wanting
  a leaning or recovering trunk.

- **X37** Variety among the tips' leaves: several sprigs painted as regions of the one leaf
  texture (requirements, Foliage), each tip's card drawing one from its foliage stream (D16).
  *Gate:* C38 landed, and the crown's tips reading as one sprig repeated.

## Notes

- Bevy source at the locked commit is in Cargo's git checkout,
  `~/.cargo/git/checkouts/bevy-*/d41bb12`; the local `../bevy` is at the same commit.
- `plans/levels.py` once failed at its foliage density edit (the edit did not apply) and passed
  with a screenshot before Enter; if it fails there again, look at that edit first.
- `plans/smoke.py` once failed at its first orbit hold (`orbit_left`, azimuth still 0) and
  passed on the next run; if it fails there again, look at the driver's press and hold.
- The driver cannot drag or scroll, so the turntable's gestures (spin, tilt, zoom, the axis
  choice) are checked by hand, on a trackpad.
- The first `Cargo.lock` was seeded from `bevy_remote_driver`'s, so Bevy resolved to the
  family's commit rather than today's main. Seed a new repo in the family the same way.
