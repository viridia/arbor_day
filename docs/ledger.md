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
| C6    | Growth-model survey, `docs/growth-model.md` (D11). |
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
| C116  | Bias in place of Size bias: size and angle by node index about the middle node, neutral when added (TD3.3); a fork's copies reach what their limb had left (TD3.7); a fork's roll is `Fuzzy`, and its chance 50% when added. |
| C58   | Stems below the ground (TD3.2). |
| C122  | Flare as a trunk operator (TD3.2), from X45. |
| C127  | A trunk fork's copies rolled to the trunk's side, so the first carries on its tube untwisted (TD3.7). |
| C124  | Best-effort open: a file's fields the species cannot take pruned and logged, the document marked edited (D32, TD2.3). |
| C129  | Bevy main at `2614884` (D1), the family together; the leaf golden test writes its failure image to the temp directory, since `target/` is now shared above the repo (TD3.10). |
| C130  | Aim in place of Bias's angle: laterals turned toward points on the envelope's outline, the upper ones toward its top (TD3.3), from the crown tests (Notes, 2026-10-21). |
| C135  | Phantom nodes below a parent's first, so the lowest limbs join a settled spiral rather than leaving one side bare (TD3.3). |
| C136  | Node gaps by balance: a node follows sooner while the limbs below it lean to one side, so the lowest limb is not left alone (TD3.3). |
| C137  | Seedling models converted to species files, best effort, by `scripts/from_seedling.py`, into `reference/` in seedling's subfolders, the crown and its profile measured from the old leaves, a negative angle bias as Aim; `plans/gallery/reference.py` opens and shoots each. First three: poplar-orange, leechgrass, palecress. |
| C139  | Operators as the species' layers (G4); guidelines admit expected corrections. |
| C138  | Leaf size by Bias: Bias's leaves share scales the cards by each lateral's length over its parent's longest, multiplied down the levels, a parent's tip taking its last lateral's, no smaller than half (TD3.3, TD3.8). |
| C140  | A fixed panel header: the document's name and file menu stay above the rows that scroll (TD1.7). |
| C141  | A short twig's card reaches back past its root, its fold at the tip, as seedling's did, so palecress's leaves sit at its stalk (TD3.8). |
| C142  | The trunk's radius down to 1 cm, the thinnest that reads at Panoply's camera distance, for flower stems and herbs. |

| C47   | A sense of scale: a measuring stick and a silhouette beside the crown, with a HUD toggle of its own (TD1.6). |

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
slots (C103), the painted leaf (C38), outlines (C66), carry-on's removal (C117) and aim (C130);
tips by steepness next.

1. **C70 Tips by steepness**: one card on a level tip and a rosette on an upright one (TD3.8)
   meet at a sudden switch, between them three looks:
   - Level: one card along the twig, its side leaves painted pairs or side branches (D27).
   - Inclined: two or three cards in a level fan centred on the twig's heading.
   - Upright: a rosette, the painted twig standing in for the real one; or none, as conifers
     need, which the upright tips field keeps.

   One rule covers them: a level fan about the twig's heading, its arc widening with steepness,
   its cards at a fixed spacing near the texture's own fan width, so steepness adds cards rather
   than closing them up. A fan of fans at small angles reads as clutter. Each tip's thresholds
   take a seeded nudge, so no line between looks runs across the crown. D27 rejected a count
   rising with steepness, since a third card showed only in a narrow range of settings; the level
   fan on the heading and the nudged thresholds are what differ.

   The switch from round twig to flat card shows where the two run in one line. An overlap hides
   it, as the level card reaches back over its twig to its fold; so does a turn, as a rosette's
   cards leave at an angle. A fan's cards are level, so on a rising twig even the middle card
   turns. A card leaving its twig at angle θ stands past the tip by about its length × cos θ, so
   rosettes and fans lengthen a limb where a level card does not: shorten each twig by its look's
   overhang, so a tip's foliage ends at the limb's reach whatever the look. A long card, such as
   a pine's needles, makes a rosette a starburst; pines bear none on uprights, which the upright
   tips field keeps. An upright's cards folded back along the twig, overlapping it as the level
   card does, is a later choice for that field.

   First, a figure in `PlantStats` for how far cards are from edge-on to the game's camera (45°
   down), to judge by numbers as well as by eye. Density counts per limb: a lateral's card and
   its side branches' against a riser's rosette. If tops grow dense, side-branch density falls as
   a limb steepens, rather than rosettes shrinking; no placement setting varies by steepness
   yet (C116's Bias varies along the parent). Card counts multiply with levels, eight triangles
   each since C39, so look at density against the budget too. E[3]
2. **C8 Working document in settings.** A settings group holding the working species and, once
   set, the file path, so a restart returns to unsaved edits and Save writes to the right file,
   as seedling did. The document's name comes from the file name. Serialized by reflection, as
   `bevy_settings` does. A working document from before a schema change is lost (D32), but not
   silently: `bevy_settings` ignores unknown keys only at a resource's top level, and below it
   drops a field that fails to deserialize without a word (`load_properties`), so the species is
   read as a species file is (C124, TD2.3), its pruned fields logged and the document marked
   edited. Stored as a list of open documents holding
   one, and the active one's index, so several (X26) need no change of format. E[3]
3. **C113 File format freeze**, at D32's gate: from here a change that renames, moves or removes
   a species field, or changes its type, needs a migration (X15). Before it, each item below
   either lands, is dropped, or is accepted as a later migration; prune the list as those
   choices are made. E[1], the items aside.
   - C70 Tips by steepness, if its fan changes the upright tips field's choices.
   - X18 Forms: a species holding every form's parameters with a selector.
   - X22 Forms as patches: today's species becomes the default patch.
   - X37 Several sprigs, the tip leaves' one sprig likely becoming a list.
   - X12 Space colonization, if Arrangement becomes a choice of modes.

### Quick wins

- **C131 Reference trees**: the species in `reference/`, converted from seedling (C137), tuned
  by the author and saved from arbor_day, which fills in their meshes; more converted once the
  first three are judged. `plans/gallery/reference.py` shoots each from the default view; add a
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
- **C125 Steady nodes**: moving Start or End moves each node, which jumps limbs about in two
  ways. A lateral's length changes, so its own node count steps; its nodes draw in order from
  one `layout` stream and its limbs' streams are numbered by a running count, so one node more
  re-deals every node after it, and every level below. Each node from a stream of its own
  (`layout.fork(node)`), and each limb's from its node and slot, so a count change adds or
  drops one node. And a node's azimuth is the cheapest of its candidates (`node_azimuths`),
  which flips as crowding changes with the nodes' spacing; blend the two cheapest when their
  costs are near. Changes every plant's layout (X15's generator version). E[2]
- **C93 Typed values lost in plans** (was X29, its gate passed): find whether a click right after
  Enter lands before the panel has rebuilt, or the typing is lost. Three failures, each once of two
  runs: `plans/levels.py` (2026-10-14) at the second level's density typed as 1.5, the plant
  keeping the default's 70 limbs; `plans/levels.py` (2026-10-16) just after that edit, the second
  level's − click not taking; `plans/cards.py` (2026-10-16), the axial droop typed as 0 keeping
  its default; `plans/operators.py` (2026-10-17), a ✕ below the fold clicked just after its
  operator was added from the menu not taking, until two frames' wait (the plan keeps it);
  `plans/widgets.py` (2026-10-17), twice, each a typed spread keeping its old value, the reach's
  0 and the sprig spacing's 10; `plans/cards.py` (2026-10-18), the lateral droop typed as 45
  becoming 90°, a wrong value rather than the old one, passing on the rerun; `plans/levels.py`
  (2026-10-19), the first level's add click not taking, one level left, passing on the rerun.
  E[2]
- **C89 `growth-model.md` against `design.md`**: its Consequences describe the model as built;
  keep the survey and the reasons, citing TD for the mechanism. E[2]
- **C90 `show.py`** from `../ldd_template`, printing a D, TD or X entry by its number, so a session
  reads the entries it needs rather than whole files. Propose it before code. E[2]
- **C42 Computed values are rounded where written**, from the first writer (Reset, Randomize,
  values derived from the generator): the number input never rounds what it shows, so round to
  the field's `Precision` in the units shown, as a drag does: through the unit's
  `canonical_to_display` and `display_to_canonical` (feathers' `NumberInputPrecision::round` is
  private).
  Defaults are written in the units shown
  (`15.0_f32.to_radians()`, not `0.3`). E[2]
- **C67 The leaf texture on the HUD** (D28), for tuning the leaf (requirements, Authoring app):
  the painted texture as an inset over a checkerboard, so its transparency shows, behind a
  toggle saved in `ViewSettings`. After C38. E[2]
- **C128 A notice for pruned fields**: a file opened with fields pruned (C124) says so only in
  the log and the dot. A line at the foot of the HUD, opposite the triangle count, fading after
  some seconds: how many fields were reset, the first few paths. Neither feathers nor
  `bevy_ui_widgets` has a toast. E[2]
- **C123 Flare's icon**: Flare (C122) borrows `icons/operators/taper.png`; an icon of its own.
  E[0]

### Later

- **C134 Facing and skew are hard to use** (the author, 2026-10-21). Facing picks how a node's
  limbs turn about the parent (around, to the sides, up or down) and skew turns each up to its
  value either way. Open: what makes them hard, whether the choices, the results being hard to
  foresee, or their part on the trunk's level once Aim (C130) chooses where limbs point. E[2]
- **C132 Repeated forks**: a maple forks and forks again (sympodial), where the levels grow a
  leader with laterals (monopodial). A fork's copies may fork again, up to a number of
  generations, 1 being today's (TD3.7, "copies do not fork again"). A copy already reaches what
  its axis had left, so each generation is shorter with no new length. Drawn forks repeated
  crowd and cross, so each generation's plane turns about 90° from its parent's, as opposite
  buds set crosswise do, with a pull up and out, the drawn roll a spread about that. Test case:
  X53's maple. E[3]
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
- **C71 Crown softness** (was part of C39): the cards' shading normal blended toward the
  envelope's outward normal by a weight (growth model, Foliage), so the crown shades as one
  mass; built as a weight and judged in the preview before it becomes an option or is fixed
  either way. Panoply's side is X13. E[2]
- **C72 Crossed cards** (was part of C39): a share of cards with a second card through the
  first along its axis, turned 90° about it, so a crown reads from any side (requirements,
  Foliage). Cupped cards are C39's negative lateral droop. *Gate:* the author wanting them
  after C70. E[2]
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
  runs straight along the twig and whose rachis tubes cost triangles that do not show at game
  scale. Its texture, a midrib with leaflets either side, is a frond mode of C38's painter, a
  second region of the leaf texture (requirements, Foliage). E[3]
- **C111 Scale**: a species field, default 1.0, that resizes a finished plant without changing
  its design. The plant grows as the species describes it, then its positions are multiplied by
  the scale; the bark outline's width and the detail threshold apply in final units, so neither
  scales. Raising the height instead redesigns the plant, since most fields are relative to it,
  and scaling in Panoply thickens or thins outlines with the model. E[2]
- **C114 The trunk's lean split in two** (was X36): a tilt where the stem leaves the ground, and
  the curve after it from the bends (C94). Lean's `angle` becomes the tilt at the base, the stem
  straight from there toward its heading (G1: the field's meaning changes, no field is added);
  the trunk gains Sag and Rise, as limbs have, acting on the heading the tilt gives, and a fork's
  stems (C75) turn by them past the fork. Rise then gives a trunk leaning at its base and
  recovering, as on slopes; sag, a bow; past level, Weep (C48). The heading stays drawn from the
  seed (D23); a fixed one would do, since Panoply turns placed instances, but changes nothing
  needed here. Old files read, a leaning trunk growing differently; not a file breaker (C113).
  TD3.2 rewritten. E[3]
- **C112 View settings panel**: a gear icon opening a dialog of view settings, saved in
  `ViewSettings` (D28): the lighting and tone mapping (`GameLook`, TD1.1; per-project looks stay
  X6). Two buttons reset the lighting and tone
  mapping: one to Bevy's defaults, one to Panoply's. E[3]
- **C126 No crown**: a Profile of Off, the envelope unbounded, for growing a plant by its levels
  alone; the crown helper hidden. A lateral's reach is then a share of the parent's length
  rather than of the distance to the envelope (X54's measure), and the node density, per crown
  radius today, per metre of the parent. E[3]

## Deferred

- **X1** The generator as its own library crate, beside the app: no Bevy dependency beyond
  `bevy_math` (its `libm` feature gives the same float results on every platform), with the
  export command as a binary of that crate, so the asset pipeline and the determinism tests
  build without the renderer. Until then the generator is `src/generator/`, on Bevy's math
  types with no ECS. The species types need `bevy_reflect` too, and their editor attributes
  (`ValueRange` and the rest, now in `src/properties.rs`) would move with them. *Gate:* the
  command-line export.
- **X3** Publishing, or reserving the name on crates.io. A crate with git dependencies cannot be
  published. *Gate:* a Bevy release with what the app uses (0.21), or the author choosing to
  reserve the name with a placeholder.
- **X4** A fast modifier for scrubbing feathers number inputs, upstream (Shift for slow is all
  there is). *Gate:* spread inputs prove slow to drag.
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
  level only, competing for points in the envelope's shell. A mode of the level's Arrangement (`docs/patches.md`, Stacks). Or, as seedling chose between monopodial and pseudo-dichotomous, a
  plant-level choice replacing the levels, the skeleton kept: a second generator. *Gate:* holes
  left once C130 and C133 are in.
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
  deliberately, the earlier export being the record. Versioned operators keep
  most changes out of this; what remains is the skeleton, the global parameters and realization.
  Reading a file with a key this generator retired prunes it with a warning (C124's processor,
  on its maintainer's advice); a policy per field (skip, warn or error), if one is wanted, goes
  in that processor. *Gate:* the first species used in Panoply.
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
  envelope guiding rather than clipping them (growth model, How the shapes are covered). The
  Weep operator. *Gate:* a weeping preset (X18).
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
  library realizes the result. Today's species becomes the default patch. After stacks (C96 to C99),
  the generator split into operators and realization, in Rust, with no language. E[4],
  split as the doc's order of work. *Gate:* the milestone's first species exported, and C10's
  file leaving room for a species to name its patch.
- **X23** Kinks at the nodes, the limb deflecting away from each child it gives off, as
  sympodial growth zigzags (oak, hawthorn). Nodes are laid out after a limb is traced, so this
  means laying them out first. An option of Wander. *Gate:* D17's kinks,
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
  replace `plans/gallery/shapes.py`. Unattended exploration runs in a subagent, keeping the
  images out of the main conversation: sweeping parameters needs only writing sweep files and
  running the one command, so a narrow allowlist covers it; trying generator changes (a form,
  say) runs in a git worktree that is thrown away afterwards, the findings written up and built
  properly. Deferred as a large, disruptive change (the crate split) whose payoff is unproven.
  E[3] *Gate:* tuning in the editor proving too slow to compare designs at game scale, or
  presets (X18) begun.

- **X30** Cards along a twig, for plants whose leaves crowd their shoots: spur-bearing trees
  (apple, larch, ginkgo) and conifers' needles (growth model, How the shapes are covered: "clusters
  along the limbs"). "Cards along the twig", default 0 with a small maximum, never past the tip's
  card, with no shell. They would also fill the crown's middle, which tip leaves alone leave as
  a wagon wheel: laterals carry their tips out toward the shell. Removed in C74, where they came
  from the shell (C37). Interior leaves
  that stand apart from any stem would want a texture of their own, without the painted stem
  (requirements, Foliage: more than one leaf texture region), and a second texture's fields
  would crowd the panel, so they wait to be an ornament (C103). *Gate:* a preset
  (X18) that another limb level cannot make dense enough.

- **X33** A fork's copies (C30) competing as one parent for their children's places (D26), so
  each copy's first child goes to its outer flank, away from the crotch; alternation still puts
  later ones inside, as in real amphitonic forks. E[2] *Gate:* C30's screenshot of a level-roll
  Y with sides facing shows the crotch crowded.

- **X35** Report upstream: a feathers number input left without a change (Enter, or focus lost)
  keeps its editing text, so an angle shows `20d` rather than `20°`, beside its interior caption
  (C77). Its text is rewritten only when `NumericValue` changes; leaving edit mode should
  rewrite it in the display form. *Gate:* a fix in the Bevy we build against.

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

- **X41** Log scale for sizes (was C32), a convenience: an `@LogScale` attribute whose unit keeps
  the number input in ln(metres), so a drag moves evenly in the log, with text printed and
  parsed in metres, since feathers drags in display units and won't drag in canonical ones.
  *Gate:* sizing small plants (herbs, under a metre) proving fiddly on the linear slider.

- **X40** Roll spread (was part of C60): each card turned about its axis at random, by up to a
  species angle, drawn from a foliage stream of the twig's own (D16), so a plant reads from the
  side, for games with a lower camera than Panoply's. Lateral droop (C39) already turns a card's
  halves toward the side. *Gate:* a game viewing plants from near level, where lateral droop
  does not keep cards from reading edge-on.

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
  (TD3.10). *Gate:* the author filing it; then a fix in the Bevy we build against, and the
  workaround dropped.

- **X49** Panoply draws every outline at `OutlineMaterial`'s default width, 0.015: it reads the
  `"outline"` extra's presence (`models::process_material_extras`) but not its value, so the
  width the export writes (C41, C66) has no effect. A material per distinct width, cached. An
  item for Panoply's ledger. *Gate:* the first species exported (C41).

- **X51** Key Symbols (C107) upstream in feathers, for the Bevy editor's menus too, the author
  proposing it. *Gate:* it landing in the Bevy we build against; then `src/shortcut.rs` uses
  feathers' and `assets/fonts/KeySymbols.ttf` goes.

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

- **X53** Values by place along the parent, in place of carry-on, revising D30. Placement works
  near the crown's base and not further up. Carry-on (TD3.6) switches level at the parent's last
  node: from the cut the axis takes the child level's sag, sides, wander and reach, and its
  laterals start at the grandchild level's start, leaving a bare stretch at each joint. Instead,
  placement settings vary with `t`, the node's place along the parent inside the crown (X22's
  `t`): angle, reach, and a leafy share, how much of a lateral bears foliage; perhaps density and
  fork chance. Nodes run to the tip, which is the smallest shoot, so no post or trident (D30's
  reasons); carry-on went in C117. Bias (C116) varies size and angle by node index.
  - Leafy share, after seedling's conifers: cards shrink only to about half before their baked
    outlines show, but lower laterals bare before their foliage and upper ones leafy throughout
    give about 4:1. Built on cards along the twig (X30). A card's back may poke past its
    lateral's base, which shows close up but not at game distance.
  - Leaf size by Bias: split out as C138.
  - The reference value is the midpoint's (C116), so the mean holds as the gradient changes;
    the base's, easiest to see, was the other choice.
  - Test cases: seedling's conifer (`artwork/reference/seedling/conifer.png`): upper laterals
    shorter, steeper and leafy throughout. A maple: forks at about a fifth of its height into
    risers that split, under a hemispherical crown, sprigs radiating outward near its surface,
    few leaves inside. A fig: floppy, drooping leaves throughout, forking near the surface. A
    citrus: many small shoots rising straight up, leaves along their whole length.
  - Not covered: depth in the crown, for the maple's shell (sprigs only in an outer shell,
    facing outward; X12 aimed at it), a later step. In a real tree the small branches fork
    just inside the crown, rather than every level stopping at it (C118): forks drawn by
    distance from the envelope rather than share of length, or each level's reach stopping
    a shell's depth short of it.
  - Open: which settings vary; the panel's control (a row split into base and tip, or a center
    and a change); how the leafy share and X30 meet; what Start becomes; sag, wander and sides
    along a limb, which stay per level.
  E[4], split before it is written. *Gate:* the author agreeing the proposal; before C113.

- **X54** Apical dominance: a lateral's reach no more than the parent's length left past its
  node, so laterals shorten toward the tip and those near it, no longer than thick, drop out;
  no field (G1). Every parent then dominant, as excurrent trees are, decurrent forms coming
  from forks; a share in place of the 1 if too strong. Under C116's Bias, which shapes the
  laterals below the cap. E[2] *Gate:* tridents still showing once End (C119) is in.

## Notes

- A hemispherical crown reads better from a Column profile with Bias shortening the upper
  laterals than from the Hemisphere profile (the author, 2026-10-18): evidence for presets
  (X18) and for profiles beside the five (X43).
- The default tree with more first-level limbs and a second level, from above (the author,
  2026-10-21). Reach is a share of the distance to the envelope along a limb's own heading, and
  leaves are at tips only, so every leaf lands on the envelope. With the second level to the
  sides, they land on its wall: a ring from above, bare inside. Facing Up lands them on the cap,
  filling the disc from above, but the side view shows long bare poles under a sheet of leaves;
  reach at 40% shortens the poles only in proportion. X53's "short limbs bearing long children"
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
  choice) are checked by hand, on a trackpad.
- The first `Cargo.lock` was seeded from `bevy_remote_driver`'s, so Bevy resolved to the
  family's commit rather than today's main. Seed a new repo in the family the same way.
