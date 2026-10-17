# arbor_day ledger

What has landed, what is next, and what is deferred and why. What the tool must do is in
[requirements.md](requirements.md), how it works in [design.md](design.md) (`TD<n>`), why in
[decisions.md](decisions.md) (`D<n>`), and the judgements made repeatedly in
[guidelines.md](guidelines.md) (`G<n>`). Chunks are `C<n>`, deferrals `X<n>`. An id is assigned once and never reused or renumbered; Next lists chunks in the
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
| C86   | Decisions out of the ledger, after `../ldd_template`: `docs/decisions.md` holds each decision as it now stands, what it rules out and what reversing it costs, with no revision history; `docs/design.md` holds the mechanism, numbered `TD<n>.<m>`. D4, D13, D14 and D18 are withdrawn, struck and pointing where their content went. Ledger 13,056 words to 7,816; Decisions 5,431 to 3,628 and design 2,598. |
| C83   | Hinting as each level's last stage (TD3.4), `src/generator/hint.rs`, in place of at trace time: an axis keeps a ring at every trace step through its level's stages, so C84's displacement works on the full path, and the next level is placed on the hinted rings, so scions start inside the bark as meshed. |
| C84   | Wander as a lightning bolt (D17, TD3.4), `src/generator/wander.rs`, in place of kinks: the trace follows the course alone, and a stage after carrying on displaces it by midpoints between fixed ends, past a straight run at the base, each stretch within the wander of its course and no move larger than at two spacings, so tips stay where the course puts them and a stem's height is reached at any wander (D23). `frame_at` takes the chord after a ring a point sits on. |
| C87   | `docs/guidelines.md` after `../ldd_template`'s: G1 (change a field's meaning before adding a field) and G2 (vary a value where its instances are many), from D25 and D19, now struck, and G3, the effort scale, after `../ldd_template`'s G7. AGENTS.md says which doc each change updates, and to read the guidelines before adding to the species. |
| C60   | Tip rosettes (D27, TD3.8), in place of sprig pitch and roll: a tip 45° or more above level bears what "Upright tips" says, a Foliage select: none, one card, or a rosette of two or three (the default) splayed toward level, so the crown's upright tips no longer stand on end. `PlantStats` counts leafy tips beside cards. Roll is X40; C94 recorded. |
| C85   | One tube along a carry-on chain (TD3.9, D30): a stem or limb drawn with the limbs carrying it on, sharing each join's ring, and a stitch of N + M triangles where the sides change. Forks split off as C95. |
| C91   | The stacks file, on paper (`docs/patches.md`, The file): operators that are off left out and read from a template with all off; versions as slots, stream keys by kind; slots, not a list; C10 lands flat after C96 groups fields into operators. |
| C96   | Operator-shaped levels (`docs/patches.md`, The file): a level's fields as its frame (Arrangement, and Shape with the angle) and its operators' structs, always on: `Bend` (toward, amount), `Wander` (angle), `Fork` (chance, at, copies, angle, roll); the trunk's wander a `Wander`; `Foliage` becomes `Ornaments`, the Leaves switch beside its one slot, `TipLeaves`. A struct nested in a subpane shows as a heading over its rows, in a node named after the field (TD1.5). Fuzzy draws are clamped by the owning struct's limits (`species::limits`). Plans' paths follow (`limbs[0]/fork/chance`, `ornaments.tip_leaves.upright`). Output unchanged. |
| C10a  | `LimbLevels { shown, levels }` (D15) in place of `limb_levels` and the `limbs` array, each depth's defaults in its `Default`; the generator reads `limbs.grown()`. A `Flatten` attribute shows a struct's rows in place of its own (TD1.5), so the levels stay beside the other subpanes. Paths: `limbs.levels[0].angle` in reflection, `levels[0]/…` in the panel and header notes. Output unchanged. |

## Next

Three parts: the path to the milestone, in order; quick wins, for between big chunks; and
everything else, unordered. Each item ends with its effort (G3). A new idea goes to Later unless the
milestone needs it.

The property editor gains a widget kind as a species field needs it. Rows are keyed by field
path, in `GetPath` syntax, which C7's undo records also use. Seedling's look is not a target.
`bevy_inspector`'s `details_panel` also walks reflected fields, but reads no attributes, has no
per-type hook, and is retained-mode.

### Milestone: the first species exported to Panoply

The foundation, the skeleton (`docs/patches.md`, The abstract plant), and tip rosettes have
landed; leaf painting next.

1. **C38 Leaf texture painting** with `tiny-skia` (D20): seedling's leaf, as D29 has it, its
   fields in a "Sprig" and a "Leaf" group in the tip leaves (`TipLeaves`, C96), and the `Interval` type (D10) for its
   variations. The painter takes the outline as an
   argument, a width in texture pixels or none, which C38 fixes at a constant until C66 makes it
   the species'. The texture goes on C37's cards, replacing the placeholder, with mipmaps, since
   the placeholder's 128 px shimmers at game scale. The defaults paint seedling's leaf, so a new
   species previews well with no leaf fields touched, as the placeholder does. If they fall
   short, the placeholder stays as a choice until they do. A golden test on its pixels compares
   with a tolerance, since SIMD can move a channel by a level. The card's axial fold
   (`AXIAL_FOLD` in `src/generator/foliage.rs`, halfway) moves to where the painted sprig's fan starts, as
   seedling's did. E[3]
2. **C70 Card facing**: first measure how far cards are from edge-on to the game's camera (45°
   down), a figure in `PlantStats`, to judge facing and folds by numbers as well as by eye; C60's
   rosettes took the steepest cards off edge-on. Then, if the figure still calls for it, the face
   target (D27) widened from world up to a blend of up and the envelope's outward normal, with a
   spread. Card counts multiply with levels, eight triangles each since C39, so look at density
   against the budget too. E[3]
3. **C66 Outlines**, a switch for the whole plant (requirements, Foliage): in the Plant group
   (C33), the switch, the bark's width as the game reads it, and the leaf stroke's in texture
   pixels (growth model, Consequences). On, the leaf texture is painted with its strokes (D29),
   the preview draws the bark with an inverted hull after Panoply's `src/materials/outline.wesl`,
   and the export writes the `"outline"` material extra (C41). Off, none of these; whether the
   translucent strokes between leaves stay is judged then. E[3]
4. **C10 Species data model**, on C96's levels, its file the one `docs/patches.md` (The file)
   lays out; its groups the layers of the growth model (D11;
   `docs/growth-model.md`, Consequences), correcting seedling where the requirements say.
   Reflectable types with `Default` for initial values and attributes for the editor. Serialized
   by reflection only, no serde derives: TOML in settings (C8) and JSON as the glTF extra, one
   representation that cannot drift. The file records the generator version it was made with
   (X15). A golden test: a species serialized and compared with a JSON file in the repo,
   brought in with `include_str!` rather than quoted inline. The limb levels are written as the
   panel shows them, a list of the shown levels only: `LimbLevels` (C10a) gets
   `#[reflect(SerializeWithRegistry, DeserializeWithRegistry)]`, which writes the shown levels,
   each by reflection through the registry, and reads one to three, filling the rest from each
   depth's defaults. Type data on the type, not a `ReflectSerializerProcessor`, since
   `bevy_settings` makes its own serializer. A hidden level then lasts a session, not a save or a
   restart. Rejected: the species as a `Vec` of grown levels with hidden ones stashed in the
   editor, the same in files but a larger change that undoes D15. Seedling's groups for
   reference: `trunk`; `branch[]`, one per level; the leaf as `leafGroup` (arrangement),
   `leafShape` and `leafColor`. Reading is reflection's, strict: a file from before a schema
   change fails with an error naming the key (D32). Split, C10a landed:
   - **C10b Writing**: the species to JSON by reflection, `generator: 1`, `LimbLevels` writing
     the shown levels; the golden test. E[3]
   - **C10c Reading**: strict, `LimbLevels` filling the levels a file leaves out; a round trip,
     and a file with an unknown key failing with its name. E[2]
5. **C40 File handling**: Save, Save As and Open, as keys (D8) and as buttons at the top of the
   panel, above the Plant group (C33); the file browser is X2. File commands act on the active document, so that tabs (X26)
   need not change them. E[3]
6. **C8 Working document in settings.** A settings group holding the working species and, once
   set, the file path, so a restart returns to unsaved edits and Save writes to the right file,
   as seedling did. The document's name comes from the file name. Serialized by reflection, as
   `bevy_settings` does. A working document from before a schema change is lost (D32), but not
   silently: `bevy_settings` ignores unknown keys only at a resource's top level, and below it
   drops a field that fails to deserialize without a word (`load_properties`), so the species is
   read by C10's strict path and its error logged. Stored as a list of open documents holding
   one, and the active one's index, so several (X26) need no change of format. E[3]
7. **C41 glTF export** from the app: the bark and leaf meshes, the leaf texture, and the species
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

- **C93 Typed values lost in plans** (was X29, its gate passed): find whether a click right after
  Enter lands before the panel has rebuilt, or the typing is lost. Three failures, each once of two
  runs: `plans/levels.py` (2026-10-14) at the second level's density typed as 1.5, the plant
  keeping the default's 70 limbs; `plans/levels.py` (2026-10-16) just after that edit, the second
  level's − click not taking; `plans/cards.py` (2026-10-16), the axial droop typed as 0 keeping
  its default. E[2]
- **C88 Landed rows as citations**: each row a line naming the TD section or decision it landed,
  the design the longer rows restate now being `design.md`'s. E[1]
- **C89 `growth-model.md` against `design.md`**: its Consequences describe the model as built;
  keep the survey and the reasons, citing TD for the mechanism. E[2]
- **C90 `show.py`** from `../ldd_template`, printing a D, TD or X entry by its number, so a session
  reads the entries it needs rather than whole files. Propose it before code. E[2]
- **C95 Forks along the chain** (split from C85): a forked limb (D31) no longer ends capped; its
  first copy continues the chain (TD3.9) as a carry-on does, with the forked limb's sides, so no
  adapter, and the shared ring rounds its turn of half the fork angle into an elbow; the other
  copies meet the chain as laterals meet their parents. A fan closes another copy's start only
  if `plans/shapes.py`'s Y and three-way fork show its open base outside the chain's tube. E[2]
- **C55 Angle by place**, seedling's `angleBias` (`MeshGenerator.ts:593`), for crowns bald on
  top: twigs radiate outward, their leaves with them (D27), and the envelope's top stays empty. A
  field per level, "Angle at tip", −90° to 90°: the change in angle from a node at the base of
  the parent's part inside the crown to one at its tip, scaled by the node's place, added to each
  limb's drawn angle before it is clamped, so upper limbs leave steeper. The angle's caption
  becomes "Angle at base". Every level, not only the trunk's as in seedling, so sprays narrow
  toward a limb's tip. A new field under G1, since nothing yet varies with place along the
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

- **C75 Stem forks**, forks (D31) on stems, at any point along them, as codominant trunks split
  (a vase elm, an old oak). A decurrent tree's leader stopping short (D23) already ends in a fork
  when the first level forks, its top being a limb of that level (D30), so what is left is a stem
  splitting below its crown, perhaps below the crown base: the trunk's height sets the axis's
  length, the fork where it splits. A fork at the ground is several splayed stems, so under
  stacks (X31) it is the trunk's Fork operator, and stems and splay may become one (G1). After
  C95, which draws the fork. *Gate:* stacks landed, or a preset needing a split trunk before
  them. E[3]
- **C94 Sag and rise as two bends**, revising D17's bend toward an angle, by which an upright
  limb, furthest from a level target, bends most, so the leader's carry-on tips over even with
  no lean (found in C60). Sag turns a limb down by its weight, each step's turn scaled by the sine
  of its angle from up, the share acting as a cantilever, so an upright limb does not sag; rise
  turns it up toward the light, an upright one having no angle left. Under stacks (X31), two
  operators, Sag and Rise, in place of Bend; before them, judge whether the flat panel takes both
  or sag alone. Changes every plant's limbs. E[3]
- **C69 A color per part of the plant**: a 4 px stripe down the left side of each top-level
  subpane, a color each for Plant, Trunk, Limbs 1 to 3 and Foliage (the ornaments), running from
  bark brown to light green, root to leaf, so the levels are told apart at a glance. Its color
  is set on the node, since feathers' contextual theming cannot shade a subtree. One dimension of
  color, by part; operators' icons (X31 step 5) take the other, by kind, so the two do not
  clash. E[1]
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
- **C92 Scions by parent size**: larger limbs of a level bear more of the next level than smaller
  ones, which today differ only by length (nodes are a density per crown radius). From the
  parent's radius, already written by the taper (G1): a threshold, parents thinner than it
  bearing none, so only main limbs carry the level; or a weight, density scaled by the parent's
  base radius over the level's thickest, to a power (0 as now), after the pipe model. Under
  stacks (X31), a parameter of the Scions operator. E[2]
- **C54 Radiating foliage** (was X20, its gate C37 passed): cards radiating from a stem's tip or
  the ground, for palms, rosettes and tufts (agave, grass, fern), with no limbs (growth model,
  How the shapes are covered): C60's tip rosettes (TD3.8) with more cards and their own lean,
  from a stem's tip or the ground. Under stacks, an ornament (X42).
  E[3]

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
  level only, competing for points in the envelope's shell. Under stacks (X31), a mode of the
  level's Arrangement (`docs/patches.md`, Stacks). *Gate:* crowns show holes or read as too regular.
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
  deliberately, the earlier export being the record. Under stacks (X31), versioned operators keep
  most changes out of this; what remains is the skeleton, the global parameters and realization.
  Reading a file with a key this generator retired wants a policy per unknown field (skip, warn
  or error), which `bevy_reflect`'s deserializer lacks: its maintainer wants one configurable in
  the deserializer (Discord, 2026-10), with no date, during reflection's rewrite. Use it if it
  has landed by the gate; otherwise the species reads its own fields. *Gate:* the first species
  used in Panoply.
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
  the same in any Bevy inspector; `LimbLevel::limits` (TD2.2) reads through `from_attributes`.
  `Tooltip` for the species' doc comments. Ours stay where it has none: `Precision` and
  `Spinbox` (its follow-ups), `ShownCount`, and units, since its `Unit` is a label while ours
  names a feathers units format that also scales and parses. *Gate:* the follow-up annotations
  PR merged, so the family is bumped and the attributes migrated once, in the same chunk; the
  family's `d41bb12` predates bevy#26164.
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

- **X30** Cards along a twig, for plants whose leaves crowd their shoots: spur-bearing trees
  (apple, larch, ginkgo) and conifers' needles (growth model, How the shapes are covered: "clusters
  along the limbs"). "Cards along the twig", default 0 with a small maximum, never past the tip's
  card, with no shell. Removed in C74, where they came from the shell (C37). Interior leaves
  that stand apart from any stem would want a texture of their own, without the painted stem
  (requirements, Foliage: more than one leaf texture region), and a second texture's fields
  would crowd the panel, so they wait to be an ornament (X42). *Gate:* a preset
  (X18) that another limb level cannot make dense enough.

- **X31** Operator stacks (`docs/patches.md`, Stacks), the stage between the fixed panel and
  formulas (X22): each level, the trunk included, a fixed frame and optional operators in a
  fixed order (bend, or sag and rise after C94, wander, fork, scions and more; the doc lists them), added from a
  menu on the level's header and removed by ✕. Fewer rows, since a missing operator shows none.
  The file, versions and stream keys are `docs/patches.md`'s (The file); versions do not cover
  the skeleton and stage order, the global parameters, or realization, which fall to X15. In
  steps, each small enough to check by eye in the app and `plans/shapes.py`'s gallery, the first
  four leaving every plant as it was:
  1. C96, operator-shaped levels: landed.
  2. `Toggle<T>` and its widget kind (the heading's ✕, the level header's add menu) on Wander
     alone; the generator skips it where off, and the file leaves it out. The risky step: the
     rest reuse the widget. E[3]
  3. Bend and Fork as toggles, and the trunk's Wander, the trunk becoming a level to the panel.
     E[2]
  4. Scions: "Add level" below the add menu's divider, and a ✕ on each level's header, in place
     of D15's − and +. E[2]
  5. Panes, each level a feathers pane and its operators subpanes, only if C33's headings feel
     cramped. A pane's header has room for more than a heading: the operator's name, its
     version when not the latest, an icon of its own, tinted by kind to break up the column,
     and the ✕. The icons are the author's to choose, from any openly licensed set in a style
     matching the HUD's Material Symbols, if each operator has a good one. The icon's tint is
     by operator, the pane's stripe (C69) by part, two dimensions that do not compete.
  6. New operators, each a chunk, judged by eye as they change output: angle bias (C55), sag
     and rise (C94), weep (C48), taper per level (X32), size bias.

  *Gate:* C10 landed, for steps 2 to 4.

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
  in the trunk's frame beside the splay, and the curve after it from the bends (C94), which need
  the heading the tilt gives. Rise then gives a trunk leaning at its base and recovering, as on
  slopes; sag, today's bowing; past level, Weep (C48). The heading may be fixed rather than drawn
  from the seed, since Panoply turns placed instances anyway. Lean's use so far, steering back a
  stem the wander sent off course, went with C84, whose wander leaves the tip on its course.
  *Gate:* C94 landed, and an author wanting a leaning or recovering trunk.

- **X37** Variety among the tips' leaves: several sprigs painted as regions of the one leaf
  texture (requirements, Foliage), each tip's card drawing one from its foliage stream (D16).
  *Gate:* C38 landed, and the crown's tips reading as one sprig repeated.

- **X38** Wander that falls off at smaller scales, so a limb sweeps in a few large kinks and its
  smaller ones are gentler; today's bolt is as jagged at every scale (D17). *Gate:* a preset
  (X18) that needs smoother limbs.

- **X39** Rounded turns, as an old oak's limbs bend (D17): each corner the hinting keeps smoothed
  into an arc with extra rings, in proportion to the tube's radius, so twigs at the detail
  threshold stay cornered. Rings tilted through a tight turn meet on its inside where the arc's
  radius is under the tube's, so the arc must be at least as wide as the tube, never fixed by
  boolean geometry. *Gate:* corners reading as too angular at game scale.

- **X42** Ornaments (`docs/patches.md`, Stacks): a plant-wide stack of what is placed on the
  grown skeleton, one slot per kind: the tip leaves (today's foliage), leaves along twigs
  (X30), radiating cards (C54), fruit. Added and removed as operators are, with the same add
  menu. C96 lays the groundwork. *Gate:* stacks (X31) landed, and a preset wanting a second
  ornament.

- **X41** Log scale for sizes (was C32), a convenience: an `@LogScale` attribute whose unit keeps
  the number input in ln(metres), so a drag moves evenly in the log, with text printed and
  parsed in metres, since feathers drags in display units and won't drag in canonical ones.
  *Gate:* sizing small plants (herbs, under a metre) proving fiddly on the linear slider.

- **X40** Roll spread (was part of C60): each card turned about its axis at random, by up to a
  species angle, drawn from a foliage stream of the twig's own (D16), so a plant reads from the
  side, for games with a lower camera than Panoply's. Lateral droop (C39) already turns a card's
  halves toward the side. *Gate:* a game viewing plants from near level, where lateral droop
  does not keep cards from reading edge-on.

## Notes

- Bevy source at the locked commit is in Cargo's git checkout,
  `~/.cargo/git/checkouts/bevy-*/d41bb12`; the local `../bevy` is at the same commit.
- `plans/smoke.py` once failed at its first orbit hold (`orbit_left`, azimuth still 0) and
  passed on the next run; if it fails there again, look at the driver's press and hold.
- The driver cannot drag or scroll, so the turntable's gestures (spin, tilt, zoom, the axis
  choice) are checked by hand, on a trackpad.
- The first `Cargo.lock` was seeded from `bevy_remote_driver`'s, so Bevy resolved to the
  family's commit rather than today's main. Seed a new repo in the family the same way.
