# arbor_day ledger

What has landed, what is next, and what is deferred and why. What the tool must do is in
[requirements.md](requirements.md), how it works in [design.md](design.md) (`TD<n>`), why in
[decisions.md](decisions.md) (`D<n>`), and the judgements made repeatedly in
[guidelines.md](guidelines.md) (`G<n>`). Chunks are `C<n>`, deferrals `X<n>`. A Landed row names what landed and cites where it is described. An id is assigned once and never reused or renumbered; Next lists chunks in the
order they are meant to land, which their ids need not follow. Panoply's ledger
(`../panoply/docs/migration.md`) is cited as "Panoply's".

## Landed

| Chunk | Change                                                                                  |
| ----- | --------------------------------------------------------------------------------------- |
| —     | Repo started; requirements moved from Panoply. |
| C1    | Shell (D5, TD1.1). |
| C2    | Settings (D6, TD1.2). |
| C4    | Turntable (D7, TD1.3). |
| C3    | Keys (D8, TD1.4). |
| C5    | Property editor by reflection (TD1.5). |
| C9    | Number inputs shaped by field attributes (D9, TD1.5). |
| C11   | The panel scrolls. |
| C12   | Fuzzy numbers (D10, TD2.2). |
| C13   | Captions above fields; replaced by C77. |
| C14   | Spread strip (TD1.5). |
| C6    | Growth-model survey, `docs/prior-art.md` (D11). |
| C15   | A trunk from the species (D12, TD2.1). |
| C16   | Crown envelope and enum selects (TD1.5). |
| C17   | Nested structs as subpanes (TD1.5). |
| C21   | Turntable feel: one-shot framing, one axis per gesture (D7, TD1.3). |
| C20   | Bark color, a feathers color input (TD1.5). |
| C22   | Literal names in `bsn!` as `#name`. |
| C18   | One limb level; D14, withdrawn. |
| C19   | Up to three limb levels (D15). |
| C26   | Random streams (D16, TD2.2). |
| C23   | Curved limbs (D17); replaced by C94 and C84. |
| C28   | Varied level fields (G2). |
| C31   | Bend in proportion; replaced by C94. |
| C34   | Fractions as percentages; feathers' since C76. |
| C36   | Nodes by density (D21, TD3.3). |
| C35   | Span as middle and length; replaced by C46. |
| C29   | Pipe-model radii; replaced by C73. |
| C24   | Stems (D23); count and splay replaced by C75. |
| C44   | Spans inside the crown (D24, TD3.3). |
| C46   | Node bias; replaced by C63. |
| C51   | Placement by competition and facing (D26, TD3.3). |
| C50   | Wander as kinks; replaced by C84. |
| C37   | Foliage along twigs in a shell; replaced by C74. |
| C61   | The trunk's height as the plant's size, and the Crown group (D23, TD3.2). |
| C53   | Four plans in place of twelve; header counts (C59). |
| C56   | The HUD (D28, TD1.6). |
| C64   | The HUD's toggles saved (TD1.6). |
| C57   | Camera buttons on the HUD (TD1.6). |
| C27   | `Placeholder` removed; the plant carries `GrownFrom` (D12). |
| C63   | Nodes from a start, leaves at every branch end, every stem drawn (D22, D27). |
| C52   | Hinting (D17, TD3.4); limb radii as shares (D22). |
| C39   | Card folds (TD3.9). |
| C33   | Control groups (TD1.5), a pattern for Panoply's editor (D3). |
| C74   | Leaves at branch ends only (D27, TD3.8). |
| C73   | Radii by one taper (D22, TD3.5). |
| C30   | Forks (D31, TD3.7). |
| C76   | Bevy main at `d41bb12` (D1): feathers' chevrons, rounding and interior captions. |
| C77   | One line per row (TD1.5). |
| C78   | Golden meshes; removed in C82 (X15). |
| C79   | One axis type, and `src/generator/` (TD3.1). |
| C80   | Skeleton, then meshes (TD3.1). |
| C81   | A radius per ring (TD3.5, TD3.9). |
| C82   | Stages as functions (TD3.1). |
| C86   | Decisions and design out of the ledger. |
| C83   | Hinting as each level's last stage (TD3.4). |
| C84   | Wander as a lightning bolt (D17, TD3.4). |
| C87   | Guidelines (G1 to G3). |
| C60   | Tip rosettes (D27, TD3.8). |
| C85   | One tube along a chain (TD3.9). |
| C91   | The stacks file, on paper (`docs/patches.md`). |
| C96   | Operator-shaped levels (TD1.5). |
| C10a  | `LimbLevels` and `Flatten` (D15, TD1.5). |
| C10b  | Writing the species file (TD2.3). |
| C10c  | Reading the species file (D32, TD2.3). |
| C97   | Toggles (TD1.5, TD2.3). |
| C98   | Bend and Fork as toggles (TD1.5). |
| C99   | Levels added and hidden from the menu (TD1.5). |
| C69   | A color per part (TD1.5), a pattern for Panoply's editor (D3). |
| C95   | Forks along the chain (TD3.9). |
| C75   | Stem forks (D31, TD3.2). |
| C94   | Sag and rise (D17, TD3.4). |
| C100  | Lean as a trunk operator (TD3.2). |
| C102  | A bare default (D33). |
| C38   | Leaf texture painting (D20, D29, TD3.10), and `GameLook` (TD1.1). |
| C103  | Ornaments as slots (TD3.8). |
| C66   | Outlines (TD3.9, TD3.10). |
| C105  | Carry-on reach; removed in C117. |
| C106  | Size bias; replaced by C116. |
| C107  | Shortcut text (TD1.7). |
| C40   | File commands (D8, TD1.7). |
| C109  | File dialogs, with `rfd` (TD1.7). |
| C41   | Save and Open as glTF (D34, TD1.7). |
| C110  | Four plan launches in place of eight; the gallery in `plans/gallery/`. |
| C101  | Control-group headings like operators' (TD1.5). |
| C65   | Wireframe toggle (TD1.6). |
| C117  | No carry-on (D30, TD3.6); every unforked tip bears leaves, the trunk's too (TD3.8). |
| C118  | Branch ends judged by eye: C119, C120, X54, and X53's depth in the crown. |
| C88   | Landed rows as citations; the file commands in TD1.7. |
| C119  | End, beside Start (D30, TD3.3), default 85%. |
| C120  | Wander up to the tip: the tip pinned, no kink inside the tip's straight run (D17, TD3.4). |
| C116  | Bias in place of Size bias (TD3.3); a fork's copies reach what their limb had left, its roll `Fuzzy` (TD3.7). |
| C58   | Stems below the ground (TD3.2). |
| C122  | Flare as a trunk operator (TD3.2), from X45. |
| C127  | A trunk fork's copies rolled to the trunk's side, so the first carries on its tube untwisted (TD3.7). |
| C124  | Best-effort open: a file's fields the species cannot take pruned and logged, the document marked edited (D32, TD2.3). |
| C129  | Bevy main at `2614884` (D1); the leaf golden test's failure image in the temp directory (TD3.10). |
| C130  | Aim in place of Bias's angle (TD3.3), from the crown tests (Notes, 2026-10-21). |
| C135  | Phantom nodes below a parent's first (TD3.3). |
| C136  | Node gaps by balance (TD3.3). |
| C137  | Seedling models converted to species files in `reference/` by `scripts/from_seedling.py`; `plans/gallery/reference.py` shoots each. |
| C139  | Operators as the species' layers (G4); guidelines admit expected corrections. |
| C138  | Leaf size by Bias (TD3.3, TD3.8). |
| C140  | A fixed panel header (TD1.7). |
| C141  | A short twig's card reaches back past its root (TD3.8). |
| C142  | The trunk's radius down to 1 cm, the thinnest that reads at Panoply's camera distance, for flower stems and herbs. |
| C143  | Tip leaves' smallest, the floor on Bias's leaf scale (TD3.8). |
| C47   | A sense of scale: a measuring stick and a silhouette (TD1.6). |
| C146  | Golden trees (TD1.7, G6); X15's first measure. |
| C70a  | Card facing in `PlantStats` (TD2.1); a baseline in Notes. |
| C70b  | Rosettes closing as their tip tilts (D27, TD3.8); Triple is Rosette, Double gone, `reference/` converted. |
| C148  | Buried limbs left out (TD3.4). |
| C149  | Wander's kinks steady under the tip's straight run (TD3.4). |
| C150  | The sprig's leaves at a fixed size (D29, TD3.8 to TD3.10, G7); `reference/` converted. |
| C151  | Wander's spacing (D17, TD3.4). |
| C71   | Crown softness (TD3.9); X13 withdrawn. |
| C152  | Cards bend with their twigs (D27, TD3.9); hemlock1 a golden tree. |
| C145  | Bias Size as a curve: `src/curve/` ([curve-editor-design.md](curve-editor-design.md), 1, 3, 5, 6; TD3.3); `reference/` converted by `scripts/size_curve.py`. |
| C155  | The inline curve editor ([curve-editor-design.md](curve-editor-design.md), 2 to 7). |
| C156a | Curve selection by rectangle, the editor's keys, and double-click deleting a point ([curve-editor-design.md](curve-editor-design.md), 4); X65. |
| C156b | The host's selection commands and `curve_self_update` ([curve-editor-design.md](curve-editor-design.md), 2, 2.1), tested in a `World`. |
| C156c | The curve editor's ⋯ menu: Reset, Copy and Paste curve ([curve-editor-design.md](curve-editor-design.md), 5, 6). |
| C158  | `src/lib.rs` holding `curve`, and `examples/curve_editor.rs`, each editor beside its curve as its app stores it ([curve-editor-design.md](curve-editor-design.md), 7). |
| C159a | Handles on the curve editor's plot, dragged; the editor a scene component; a `cubic-bezier` row in the example ([curve-editor-design.md](curve-editor-design.md), 4, 5, 7). |

## Next

Three parts: the path to the milestone, in order; quick wins, for between big chunks; and
everything else, unordered. Each item ends with its effort (G3). A new idea goes to Later unless the
milestone needs it.

The property editor gains a widget kind as a species field needs it. Rows are keyed by field
path, in `GetPath` syntax, which C7's undo records also use. Seedling's look is not a target.
`bevy_inspector`'s `details_panel` also walks reflected fields, but reads no attributes, has no
per-type hook, and is retained-mode.

### Milestone: the first species exported to Panoply

The foundation, the skeleton (`docs/patches.md`, The abstract plant), tip rosettes, the species
file (C10), operator stacks (C96 to C99) and colors by part (C69) have
landed, and forks drawn along the chain (C95) and on stems (C75), sag and rise (C94), ornament
slots (C103), the painted leaf (C38), outlines (C66), carry-on's removal (C117), aim (C130) and
the golden trees (C146). Before the freeze (C113), the changes that would cost the finished trees
a migration or a new look land while the trees are few, the golden trees showing each.

1. **C157 Bias Leaves and Aim as curves**: two more `PlaceCurve` fields, converted as Size was
   (`scripts/size_curve.py` generalized). Decided here: whether reach and angle take curves too.
   E[2]
2. **C125 Steady nodes**: moving Start or End moves each node, which jumps limbs about in two
   ways. A lateral's length changes, so its own node count steps; its nodes draw in order from
   one `layout` stream and its limbs' streams are numbered by a running count, so one node more
   re-deals every node after it, and every level below. Each node from a stream of its own
   (`layout.fork(node)`), and each limb's from its node and slot, so a count change adds or
   drops one node. And a node's azimuth is the cheapest of its candidates (`node_azimuths`),
   which flips as crowding changes with the nodes' spacing; blend the two cheapest when their
   costs are near. Changes every plant's layout (X15's generator version). E[2]
3. **C134 Facing, skew and angle as G5 has them** (the author, 2026-10-21 and 10-22): adjusting
   facing and skew seldom improves a plant, since the model decides what they promise to set.
   - Facing only where the parent has an upper side: an upright trunk has none, so Sides, Up and
     Down measure from an arbitrary side there. The trunk's level is always Around (crowding's
     spiral, `node_azimuths`), its Facing row hidden.
   - Skew becomes Irregularity: how far limbs stray from crowding's pick, the random turn
     `placement.rs` adds after it, captioned by effect, a tidy spiral to scattered. Its field and
     range stay (G1). It may go instead; its default is already 0, so a species that never set it
     changes nothing either way.
   - Angle under Aim: Aim keeps each limb's heading and turns its angle, so an author setting the
     angle with Aim on sees Aim's result. The angle row says so while Aim is on, or the two meet
     in one row; which, decided when built.

   E[2]
4. **C147 Sprigs as a list** (X37's format, ahead of its feature): the tip leaves' sprig becomes
   a list holding one, the panel showing it as now, so several sprigs (X37) later need no
   migration. `reference/` converted. E[1]
5. **C113 File format freeze**, at D32's gate: from here a change that renames, moves or removes
   a species field, or changes its type, needs a migration (X15). Before it, each item below
   either lands, is dropped, or is accepted as a later migration; prune the list as those
   choices are made. E[1], the items aside.
   - **Migrated**, a field's type or meaning changing, `reference/` converted in the same chunk:
     - C157 Bias Leaves and Aim as curves: their numbers become two-point curves, as Size's did
       (C145), with no change of output.
     - C147 Sprigs as a list: the tip leaves' one sprig becomes a list of one.
   - **Output changed**, files reading as before but growing differently, each tree judged close
     enough, retuned, or the change put behind an option:
     - C125 Steady nodes: every plant's layout.
     - C134: a trunk level whose Facing is not Around.

   Additive changes, whose default or off state grows today's plant, need no place here (C132,
   C133, X12's modes with today's as the default), nor does X22, a species of its own kind.

6. **C8 Working document in settings.** A settings group holding the working species and, once
   set, the file path, so a restart returns to unsaved edits and Save writes to the right file,
   as seedling did. The document's name comes from the file name. Serialized by reflection, as
   `bevy_settings` does. A working document from before a schema change is lost (D32), but not
   silently: `bevy_settings` ignores unknown keys only at a resource's top level, and below it
   drops a field that fails to deserialize without a word (`load_properties`), so the species is
   read as a species file is (C124, TD2.3), its pruned fields logged and the document marked
   edited. Stored as a list of open documents holding
   one, and the active one's index, so several (X26) need no change of format. E[3]

### Quick wins

- **C160 A placeholder for no point**: while no single point is selected, the curve editor's
  point inputs are disabled and show 0, which reads as a value (CE25). Swap the input row for a
  placeholder row of the same height, its captions over "—" as the mock has it, by toggling
  `Display::None`, so the inputs keep their entities and configuration. Several points selected
  could show their count. A pinned end's x stays a disabled input, so `restyle_number_input`
  (X67) stays. E[1]
- **C131 Reference trees**: the species in `reference/`, converted from seedling (C137), tuned
  by the author and saved from arbor_day, which fills in their meshes; the first three are tuned, the
  other 44 are as converted. `plans/gallery/reference.py` shoots each from the default view; add a
  shot from above, so a change to the layout (C125, C130, C132, C133) is judged against them.
  E[2]
- **C108 Ask before dropping unsaved changes**: New, and Open once it works, replace an edited
  species without asking (C40). A feathers dialog (Save, Don't Save, Cancel) while the document
  has its dot. After C41, so Save in the dialog saves. E[2]
- **C121 Frame on the trunk's middle** (TD1.3): zooming in cuts off the roots or the crown first,
  since the view centres on the subject's bounding sphere and zoom moves the focus toward the
  height under the pointer. Frame looking at the middle of the trunk's height, or of the whole
  plant's, so both ends leave the view together. Zoom's pull toward the height under the pointer
  is taken where the pointer's ray meets a vertical plane through the axis, which a steep view
  meets at a grazing angle: looking straight down, it swings to the root or the top and clamps
  there, and at the home elevation a pointer just above centre pulls well above the focus. Fade
  the pull with the tilt, by cos(elevation). E[2]
- **C93 Flaky typing in plans** (was X29): plans that type into a number input sometimes keep
  the old value (once a wrong one, 45 typed becoming 90°), or the click right after misses; nine
  failures from 2026-10-14 to 10-22 (the last `levels.py`'s add click, one level left) in `levels.py`, `cards.py`, `operators.py` and `widgets.py`,
  each passing on a rerun. Only plans type; the author drags. To tell the causes apart, a plan
  helper types, presses Enter and `expect`s the field's value before the next step: a failure
  there is the typing lost, a miss after it a click landing before the panel rebuilds. Then fix
  the one it is. E[2]
- **C128 A notice for pruned fields**: a file opened with fields pruned (C124) says so only in
  the log and the dot. A line at the foot of the HUD, opposite the triangle count, fading after
  some seconds: how many fields were reset, the first few paths. Neither feathers nor
  `bevy_ui_widgets` has a toast. E[2]
- **C123 Flare's icon**: Flare (C122) borrows `icons/operators/taper.png`; an icon of its own.
  E[0]
- **C144 Feathers' Key Symbols** (was X51, its gate passed): feathers on Bevy main has the Key
  Symbols font (C107); `src/shortcut.rs` uses feathers' and `assets/fonts/KeySymbols.ttf` goes.
  If the locked Bevy predates it, `cargo update -p bevy` first, a full rebuild. E[1]

### Later

- **C159b In and Out inputs** (C159 was X62): a row each for the selected point's shown handles
  (CE34), sending `MoveHandle`, as the mock's "In and Out inputs" frame draws them (b). Spawned
  unless the host's `@handle_inputs` prop is false, as `@point_inputs` spawns the point's. E[1]
- **C159c A curve of mixed segments and handle modes** in `examples/curve_editor.rs`, stored as
  the whole `ScalarCurve` written with `{:#?}`, which shows its structure with no new
  dependency. With it, an aligned handle's opposite: `apply` turns it from its current offset,
  so a drag that clamps it loses its length for the rest of the drag, and Cancel leaves it so.
  The crown profile as a curve could use aligned and free handles (an hourglass crown), though
  whether authors should shape a crown that way is open. The leaf outline is not a customer: its
  handles may point back past the base (ivy), so it is no function of length. E[2]
- **C161 Choosing segment kinds and handle modes**: the editor offers only the kinds and modes
  the consumer allows (CE3), but nothing yet lets an author change either (`SetSegment`,
  `SetHandleMode`). Starts by updating the mocks. E[2]
- **C132 Repeated forks**: a maple forks and forks again (sympodial), where the levels grow a
  leader with laterals (monopodial). A fork's copies may fork again, up to a number of
  generations, 1 being today's (TD3.7, "copies do not fork again"). A copy already reaches what
  its axis had left, so each generation is shorter with no new length. Drawn forks repeated
  crowd and cross, so each generation's plane turns about 90° from its parent's, as opposite
  buds set crosswise do, with a pull up and out, the drawn roll a spread about that. Test case:
  a maple forking at about a fifth of its height into risers that split again, under a
  hemispherical crown. E[3]
- **C133 Prune**: a plant operator after layout, thinning an overgrown crown to even, as a tree
  sheds its shaded shoots. Each tip scored by shade, the foliage above and around it in a coarse
  grid over the envelope; the most shaded removed first, with its subtree, and a limb left bare
  removed up its chain, until a target density or card count. It hollows the inside and lifts
  the crown from below. It removes only, so it closes no gap. The same grid gives `PlantStats` a
  figure: the share of empty cells inside the silhouette from the game camera. After C132,
  whose forks it could even out in place of a crowding cost. *Gate:* a tree overgrown by today's
  settings (the second level facing Around, dense) overfilling every region meant to be leafy;
  gaps left there are the layout's, for C130, C132 or X12. E[3]
- **C104 Fruit and flowers** (was part of X42), ornaments (C103): for plants grown for them
  (sunflower, tulip, lavender) as well as blossom on trees, heads at tips or nodes, small meshes
  in vertex color or cards from a region of the leaf texture. *Gate:* C103, and a preset wanting
  them. E[3]
- **C48 Arching stems**, for fountain shrubs: a lean past level, so a stem rises and then droops
  toward the ground, stopping above it. D24 stops the lean at level until a preset needs more
  (X18). The Weep operator on the trunk. E[2]
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
  base radius over the level's thickest, to a power (0 as now). A
  parameter of the Scions operator. E[2]
- **C54 Radiating foliage** (was X20, its gate C37 passed): cards radiating from a stem's tip or
  the ground, for palms, rosettes and tufts (agave, grass, fern), with no limbs (growth model,
  How the shapes are covered): C60's tip rosettes (TD3.8) with more cards and their own lean,
  from a stem's tip or the ground, an ornament (C103). Settings: the count, the angle from up
  with a spread, so young fronds stand and old ones droop, and the length. A palm's frond is one
  card arching along its length: several axial segments, each bending further down, in place of
  C39's one fold, its V from the lateral droop; not a limb level with tip leaves, whose card
  bends only as its twig does (C152) and whose rachis tubes cost triangles that do not show at
  game scale. Its texture, a midrib with leaflets either side, is a frond mode of C38's painter, a
  second region of the leaf texture (requirements, Foliage). E[3]
- **C112 View settings panel**: a gear icon opening a dialog of view settings, saved in
  `ViewSettings` (D28): the lighting and tone mapping (`GameLook`, TD1.1; per-project looks stay
  X6). Two buttons reset the lighting and tone
  mapping: one to Bevy's defaults, one to Panoply's. E[3]
- **C153 The leader's tip apart from steep limbs**: Upright tips take every tip steeper than 45°,
  so a hemlock whose leader should be bare loses the leaves of its upper third's limbs too. A
  choice for stem tips in the tip leaves (G4), today's Upright by default, so files grow as
  before. E[1]
- **C154 Wander straight only from above**: the run under a tip's card need only be straight in
  plan, now that a card bends with its twig (C152), which reads as leaves drooping or lifting. Its
  kinks keep their vertical part, their horizontal part laid on the line from the last kink before
  the run to the tip, by plan distance along the course, so the run's length still re-deals no
  kink (C149, G7). An envelope pulling a point back moves it along its whole offset, slightly off
  that line. Each kink kept under a card costs two triangles. Changes the leaves of every tree
  with wander. E[2]

## Deferred

- **X3** Distribution: the app as binaries on GitHub Releases, crates.io at most reserving the
  name (a crate with git dependencies cannot be published there). *Gate:* a Bevy release with
  what the app uses (0.21).
- **X5** A dual-thumb range slider (requirements, Authoring app, a **C** item). The span, the
  case that called for it, is gone (C46). *Gate:* the
  center-and-spread row proves clumsy.
- **X6** The target game's look set per project, for games that render unlike Panoply: a home
  elevation and field of view, and a view gizmo, for a camera unlike Panoply's (C4 has tilt by
  dragging, but its home view is Panoply's); and the lighting and tonemapping, `GameLook`
  (TD1.1), now Panoply's in its `Default`. Where project settings live, a project file beside the
  species or the app's settings, is decided then. *Gate:* another game using the tool.
- **X7** Propose a turntable camera for `bevy_camera_controller`, with C4 as the prototype. Bevy
  has free, pan and pan-orbit controllers; the issue tracker has nothing on a turntable (searched
  2026-10-11). Pan-orbit can turn off pan and fix the up direction, but cannot lock pitch, and it
  orbits about the point under the pointer rather than a fixed vertical axis through the subject.
  *Gate:* C4 landed and used long enough to settle its controls.
- **X12** Space colonization (`docs/prior-art.md`) choosing the directions of the last limb
  level only, competing for points in the envelope's shell. A mode of the level's Arrangement (`docs/patches.md`, Stacks). Or, as seedling chose between monopodial and pseudo-dichotomous, a
  plant-level choice replacing the levels, the skeleton kept: a second generator. *Gate:* holes
  left once C130 and C133 are in.
- **X15** Keeping old trees as they were when the generator changes. The game ships exported
  meshes, so a tree changes only when regenerated: edited again, or rebuilt by the command-line
  export if the asset pipeline runs one. Rather than keeping every past generator (Cargo
  could depend on several versions under different names), three measures:
  (1) the golden trees (C146), so no output changes by accident; C78's hash of four species
  built in code, in git history, served the skeleton's refactors (C79 to C82);
  (2) a generator version in the species file, raised only when output changes on
  purpose, which the export can warn about; (3) an additive change defaults to the old output
  when the file lacks its field (no wander, say), so a new species' default and a missing
  field's default are different things. What remains (fixes, reordered draws) is re-exported
  deliberately, the earlier export being the record. Versioned operators keep
  most changes out of this; what remains is the skeleton, the global parameters and realization.
  Reading a file with a key this generator retired prunes it with a warning (C124's processor,
  on its maintainer's advice); a policy per field (skip, warn or error), if one is wanted, goes
  in that processor. A variant renamed after the freeze keeps its old name as a custom attribute
  on the variant (`#[reflect(@Alias("Triple"))]`; variants carry them in the Bevy we build
  against), which that processor reads when a string names no variant, logging the repair;
  before the freeze, renames convert `reference/` instead (G6). *Gate:* the first species used in Panoply.
- **X18** Presets. The tool is for indie developers wanting the common Earth plant types
  (requirements, What it is for), so presets named after those types ("Fir", "Willow", "Live
  oak", "Clump"), each a saved species, are its starting points. Controls in the tree's terms
  are X22's, the node graph. *Gate:* the reference trees (C131) tuned.
- **X19** Pendulous levels, for willows: a level whose limbs may hang below the envelope, the
  envelope guiding rather than clipping them (prior art, How the shapes are covered). The
  Weep operator. *Gate:* a weeping preset (X18).
- **X22** Forms as patches (`docs/patches.md`), after Reaktor's front panels and Blender's
  geometry nodes: operators over an abstract plant, edited as a node graph of structure and
  value wires and saved as a file (patches, Node graph), each input a value or a formula over
  the author's named controls, evaluated per plant or per element, never per vertex; a Rust
  library realizes the result. Built in three steps: structure only, then per-element inputs
  (C145), then value wires and controls. A patch is a species of its own kind, tagged apart
  from today's in the glTF extra, so neither is read as the other. After stacks (C96 to C99),
  the generator split into operators and realization, in Rust, with no language. E[4],
  split as the doc's order of work. *Gate:* the milestone's first species exported, and C10's
  file leaving room for a species to name its patch.
- **X24** Cards that include their stem and stand in for the finest twigs, one card per twig
  of the last level in place of its tube, rather than leaves lined up with meshed twigs (D27),
  saving the triangles those twigs cost now that every stem is drawn (D22). Seen edge-on the twig vanishes, so it needs crossed cards (twice the
  triangles) or facing kept away from edge-on. *Gate:* the last level's twigs costing too much of the triangle budget.

- **X25** The property editor's attributes from `bevy::annotations` (bevy#26164, merged as `8550394`): its
  `ValueRange` (an `f64` range, read with `ValueRange::from_attributes`), `Step` and
  `DisplayName` in place of ours (`ValueRange<T>`, `Step<T>`, `Caption`), so a species reads
  the same in any Bevy inspector; `LimbLevel::limits` (TD2.2) reads through `from_attributes`.
  `Tooltip` for the species' doc comments. Ours stay where it has none: `Precision` and
  `Spinbox` (its follow-ups), `ShownCount`, and units, since its `Unit` is a label while ours
  names a feathers units format that also scales and parses. *Gate:* the follow-up annotations
  PR merged, so the attributes are migrated once, with the bump that brings it; bevy#26164 is
  in since C129.
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
  `set_resource`) move to `set` on the active document's entity. Open: closing a tab with unsaved changes, and whether a new document starts
  from defaults or a copy of the active one. *Gate:* C8 and C40 landed, and an author working on variants.

- **X30** Cards along a twig, for plants whose leaves crowd their shoots: spur-bearing trees
  (apple, larch, ginkgo) and conifers' needles (prior art, How the shapes are covered: "clusters
  along the limbs"). "Cards along the twig", default 0 with a small maximum, never past the tip's
  card, with no shell. They would also fill the crown's middle, which tip leaves alone leave as
  a wagon wheel: laterals carry their tips out toward the shell. Removed in C74, where they came
  from the shell (C37). Interior leaves
  that stand apart from any stem would want a texture of their own, without the painted stem
  (requirements, Foliage: more than one leaf texture region), and a second texture's fields
  would crowd the panel, so they wait to be an ornament (C103). Then a leafy share along the
  parent, after seedling's conifer (`artwork/reference/seedling/conifer.png`): lower laterals
  bare toward their base, upper ones leafy throughout; a card's back may poke past its lateral's
  base, which shows close up but not at game distance. *Gate:* a preset
  (X18) that another limb level cannot make dense enough.

- **X37** Variety among the tips' leaves: several sprigs painted as regions of the one leaf
  texture (requirements, Foliage), each tip's card drawing one from its foliage stream (D16).
  Its list of sprigs is C147's. *Gate:* C38 landed, and the crown's tips reading as one sprig repeated.

- **X38** Wander that falls off at smaller scales, so a limb sweeps in a few large kinks and its
  smaller ones are gentler; today's bolt is as jagged at every scale (D17). *Gate:* a preset
  (X18) that needs smoother limbs.

- **X43** Crowns widest at the top: profiles beside `Profile`'s five, an umbrella (a flat top
  over a narrow base, for acacia and stone pine) and a vase (widening to near the top, for elm
  and zelkova), or a squash of the existing ones. Layered crowns (cloud-pruned pines, tiered
  trees) are not this: they want several envelopes, which the skeleton does not have. *Gate:* a
  preset (X18) needing one.

- **X44** Many stems: bamboo groves, thickets and reed beds, past a ground fork's five stems,
  started over a disc of the clump's radius rather than from one point, and for bamboo its node
  rings as bands of bark color. Triangles grow with the count, so judge against the budget.
  *Gate:* a preset (X18) needing a clump the stems cannot make.

- **X45** A radius profile along an axis, beside the even taper (D22) and the flare (C122):
  the radius from a curve rather than a line, for bottle trunks (baobab) and bulging stems.
  Rings carry their own radii (C81), so it is the taper stage written another way. *Gate:* a
  preset needing a bulge.

- **X46** Roots as axes: buttress roots from a stem's base, and prop and aerial roots from its
  limbs (mangrove, banyan), each a scion traced down toward the ground and stopped there, rather
  than out toward the envelope. The skeleton takes them as axes of a new kind; what is new is
  the course and where they start. Buttresses are flattened fins, which a tube approximates
  with thick roots running together at the base. *Gate:* a preset (X18) needing them.

- **X47** Alpha coverage kept through the leaf texture's mipmaps: each level's alpha scaled so
  the share of pixels past the material's cutoff (0.5) matches the full-size texture's, since a
  box filter thins cut-out detail as it shrinks, and leaves would thin out with distance. A few
  lines in `plant.rs`'s `with_mips`, and the export's pipeline would need the same (C41).
  *Gate:* crowns reading sparser at game scale than close up.

- **X48** Report upstream: an `Image` replaced in `Assets` with the same texture descriptor keeps
  its GPU texture, and `GpuImage::prepare_asset` writes the new data with `as_image_copy()`, mip
  level 0 only (`bevy_render/src/texture/gpu_image.rs`, `prepare_asset`), so the other levels
  keep their first contents. The leaf texture works around it with a new image each build
  (TD3.10). Filed as bevy#26225; the fix (branch `mips-fix`) was tested here on 2026-10-22, with
  the leaf image replaced in place, and a color edit showed at full zoom-out. *Gate:* the fix in
  the Bevy we build against, and the workaround dropped.

- **X52** Teeth independent of the leaf's shape (was C115). Serration is hard to steer because
  the tapers and widths reach the teeth in `leaf::half_outline`: the edge is cut at equal steps of
  the curve's parameter, so a taper moves teeth along it; each tooth is pushed square to the edge
  at its segment's end, so the outline tilts it; and serration is in leaf lengths. Its uses to
  keep: deep teeth read as pine needles at a distance, on snow-covered trees especially; on a
  flared base, teeth square to the edge point back, as a maple's lower lobes do; and the tip's
  spike. Tried and stashed (2026-10-18):
  - Equal depth square to the edge, in tooth lengths, teeth spaced evenly along it: teeth alike
    across tapers, but needles needed serration past 1 (to 400%), and the spike, once dropped,
    was missed. Brought back by lengthening the last segment it was a thin tail; straight-sided,
    long and thin past 1.5.
  - The points on the outline inflated about the midrib's middle (the author's expectation: an
    inflated leaf through the points, spaced as their bases): on a narrow leaf the base teeth
    became prongs below the stem, the side teeth the shortest, the leaf a spindle. Not kept.
  Open: rake as an angle from square to the edge, so a lean holds at any depth; inflating across
  more than along. *Gate:* the author settling what serration should mean.

- **X56** Rosette cards shrunk before they are dropped (C70b), so a count steps down gently.
  The author doubts it would look good, and the game's trees are static, so a drop never pops on
  screen: a step shows only across the crown, which C70b's nudge spreads, or while dragging a
  setting. *Gate:* a line between counts seen in a crown despite the nudge.

- **X57** A sprig for rosettes without its stem and pairs, only the fan: on a leaf with long
  stems below the fan, rosette cards splayed from the tip look odd. Fewer leaves per card, so
  less density. A second sprig, one more region of the leaf texture (X37's), chosen by the
  tip's look. *Gate:* a species whose rosettes look odd for their stems.

- **X60** Leaves standing out of the crown (was C70c, twigs shortened by their overhang).
  Measured after C70b (2026-10-24). Along the twig, rosettes stand little past a level card: in
  card lengths 0.57 at most against 0.47 for the default species, and on floppy
  (`reference/bugs/`, axial droop 56°) 0.46 against 0.41. Sideways, on floppy, they stand out
  most: of 169 cards, 6 have a corner over half a card outside the envelope, and the five worst
  are rosettes of two or three on second-level twigs 0.2 to 0.45 m long, tipped 15° to 35° from
  up, 0.2 to 0.4 m inside the crown, their 0.66 m cards splayed up to 0.42 m out. Every
  first-level tip lands on the envelope (reach), its level card 0.3 m past it, which reads as
  the crown's edge. Perhaps only the spreads; fixes, if not: a rosette card that would end
  outside the envelope turned up or shrunk; cards scaled down on twigs much shorter than a card;
  or rosettes opening less. *Gate:* a second tree with isolated leaves out of its crown, the
  same kind of tip each time.

- **X61** Crown softness's two weak spots (C71). Bevy flips a double-sided material's back-face
  normal (`prepare_world_normal`), so a card seen from behind darkens as softness rises; and a
  card low in the crown faces up while the envelope there faces down, so near 50% its normal
  turns over in a step. Fixes, if either shows: a pair of one-sided cards back to back, each
  with its own normal (twice the leaf triangles), or a leaf shader that mirrors a back face's
  normal across the card. *Gate:* either read wrong from the game's camera on a soft species.

- **X58** The curve editor's plot drawn as scene content anchored to the plot node, to try the author's
  unified UI proposal (`https://hackmd.io/@dreamertalin/H1ruQhFnGl`) before offering it to
  Bevy: grid, curve and tangent lines as retained gizmos, one per width and style, under an
  orthographic camera in layout pixels with its own MSAA, since gizmo lines have no edge
  antialiasing of their own. The handles stay UI nodes and must draw over the curve, which over
  the panel, so it needs the proposal's anchoring and render bands (its steps 2 and 3) and none
  of its surfaces. Only the plot layer changes; model and editor stay. *Gate:* C155 landed on
  the path material.

- **X66** Several curves in one editor (CE35): each a child entity with its own value and
  constraints, the view's y range per curve. Every reference already names its curve
  ([curve-editor-design.md](curve-editor-design.md), 2, 8); what assumes the curve is on the
  editor's own entity is the selection (`select_curve`, `follow_point_count`), the markers and
  the menu. A row of its own in `examples/curve_editor.rs`. *Gate:* a host showing two curves
  together, such as Bias's Size and Leaves, or Panoply's editor.

- **X59** Zoom and pan in the curve editor, for handles that reach past the curve. A slope tangent
  drawn at a fixed length on screen, so only handles whose length means something (Bézier
  points, weighted tangents) leave the view; the view fits points and handles with padding when
  a drag ends, never during it, a handle clamped to the edge till then; wheel to zoom (an axis
  with a modifier), middle-drag to pan, a key and one header button to fit again. No scrollbars in
  the inline editor; a larger host may draw them from the view (CE36), the wheel over one zooming
  its axis.
  The model bounds the view (x in 0 to 1), the editor owns it within them. *Gate:* C159a landed.

- **X67** Feathers' number input, upstream: bugs to report, each worked around here or lived
  with until it is fixed in the Bevy we build against. The first three are about how the input
  formats and rounds its value, and could go as one issue. A no-value state (was X65) is not
  proposed: it drew objections on Discord, and the curve editor swaps in a placeholder instead
  (C160). *Gate:* each item fixed in the Bevy we build against, and its workaround dropped.
  - **Text left in its editing form** (was X35): an input left without a change (Enter, or focus
    lost) keeps its editing text, so an angle shows `20d` rather than `20°`, beside its interior
    caption (C77). Its text is rewritten only when `NumericValue` changes; leaving edit mode
    should rewrite it in the display form. Lived with.
  - **Chevrons skip the precision**: scrubbing and typing round to `NumberInputPrecision`, but
    `chevron_on_press` adds the step in `f64` and converts back unrounded, so a fractional step
    drifts by an `f32` step (0.35 becoming `0.35000002`). The curve editor rounds each value from
    its inputs to the axis's precision (`edit_from_input`).
  - **The display ignores the precision**, a question rather than a bug: the text is the
    shortest decimal that reproduces the `f32`, so a value off the precision prints in full.
    Precision governs only what a user enters. Formatting to it would have hidden both drifts
    above; ask whether that is intended.
  - **Re-enabled, drawn disabled** (was X64): an input re-enabled by removing
    `InteractionDisabled` stays drawn disabled. `number_input_on_remove_disabled` observes
    `Remove`, which runs while the component is still there, so `Has<InteractionDisabled>` reads
    true and it restyles as disabled. The curve editor re-inserts the text field's `Hovered`,
    which restyles from the input's state (`restyle_number_input`).
  - **Scrub speed tied to the step**: without a `SoftLimit`, a scrub moves `NumberInputStep` ×
    0.01 per pixel, so an input with a fine step creeps (the curve editor's Bias y, 1% per
    100 px), and a `SoftLimit` to speed it up draws the slider bar, wrong for an open-ended
    value. Either a component setting the speed alone, or a default formula that scrubs a step
    in a few pixels. Lived with.

- **X63** A curve field's x axis from an attribute: today every curve row takes x from 0 to 1, its
  ends pinned, as a place along the parent ([curve-editor-design.md](curve-editor-design.md), 6).
  *Gate:* a curve over another domain, such as height in metres.


## Notes

- Card facing (C70a) before C70b, 2026-10-24: a level card scores 0.71, an upright one 0.45. The
  reference trees run from 0.53 (the pines, poplar-orange, shadowfoil) to 0.67 (juniper1); the
  default species 0.63. After C70b the Rosette trees score lower, from 0.48 (shadowfoil) to 0.64
  (fireweed), juniper1 0.56, crown-med 0.60 (0.65), and bear fewer cards (juniper1 42, from 87),
  since steep tips' cards now rise toward 45° and drop out. The Single trees are unchanged.
- A hemispherical crown reads better from a Column profile with Bias shortening the upper
  laterals than from the Hemisphere profile (the author, 2026-10-18): evidence for presets
  (X18) and for profiles beside the five (X43).
- The default tree with more first-level limbs and a second level, from above (the author,
  2026-10-21). Reach is a share of the distance to the envelope along a limb's own heading, and
  leaves are at tips only, so every leaf lands on the envelope. With the second level to the
  sides, they land on its wall: a ring from above, bare inside. Facing Up lands them on the cap,
  filling the disc from above, but the side view shows long bare poles under a sheet of leaves;
  reach at 40% shortens the poles only in proportion. "Short limbs bearing long children"
  (2026-10-19) makes those poles once the children point up. Led to C130, C132 and C133.
- KTX2 for the pipeline (from C41's research): Bevy's glTF loader sends any image through
  `Image::from_buffer` by mime type or extension (`bevy_gltf/src/loader/mod.rs:1227`), so KTX2
  loads with Bevy's `ktx2` feature, embedded or by path, a path loaded as its own asset shared
  across models. Bevy does not read `KHR_texture_basisu` (`bevy_gltf/src/lib.rs:122`,
  bevy#19104), which gltf-transform's KTX2 conversion writes, so the pipeline points the image
  straight at the `.ktx2`, which Bevy accepts though the standard does not.
- Bevy source at the locked commit is in Cargo's git checkout,
  `~/.cargo/git/checkouts/bevy-*/2614884`; the local `../bevy` has `upstream/main` fetched
  at the same commit, but its checked-out branch differs.
- `plans/smoke.py` once failed at its first orbit hold (`orbit_left`, azimuth still 0) and
  passed on the next run; if it fails there again, look at the driver's press and hold.
- The driver cannot drag or scroll, so the turntable's gestures (spin, tilt, zoom, the axis
  choice) are checked by hand, on a trackpad. Drags are the driver's X8.
- The first `Cargo.lock` was seeded from `bevy_remote_driver`'s, so Bevy resolved to the
  family's commit rather than today's main. Seed a new repo in the family the same way.
