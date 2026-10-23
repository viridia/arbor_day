# Guidelines

How this project makes the judgements it makes repeatedly. Read the first group before scoping any
chunk, the second before scoping one that adds to the species.

**What this document admits.** Advice a maintainer would otherwise have to learn by being corrected.
The test is whether someone new, following it, would choose differently than without it. A guideline
may be departed from, with the reason stated where the departure is made ("departs from G1 because
…"); a requirement in [requirements.md](requirements.md) may not. Why a particular choice was made
belongs in [decisions.md](decisions.md).

**Where a guideline comes from.** A correction the author has had to make, or expects to, written
with a case that shows it, since the case is what lets a reader recognize the next one.

**Numbering.** Each guideline's number is a permanent identity, never reused. A gap in the sequence
is a retired guideline.

**Next: 7.**

---

## 1. Scoping

### G3 — Estimate effort on an exponential scale

An estimate is for sorting and prioritizing, not scheduling, so a wrong one costs a misplaced item
and nothing more. Each level is several times the one below, judged by what the change touches
rather than by how long it seems:

| Level | Touches                                                                        |
| ----- | ------------------------------------------------------------------------------ |
| E[0]  | One line, or prose only                                                        |
| E[1]  | One function or one document section, with its test                            |
| E[2]  | Several functions; the design stays as it is                                   |
| E[3]  | A new mechanism, or a `design.md` section rewritten: one chunk                 |
| E[4]  | More than a chunk, or a changed requirement or decision; split before it is written |
| E[5]  | A core subsystem redesigned                                                    |

A chunk carries its level at the end of its entry in Next. A level is written once and overwritten
when scoping proves it wrong, not kept up to date. It measures effort, not risk: a one-line change
that alters every plant's output is still E[0]. After `../ldd_template`'s G7.

## 2. The species

### G1 — Change a field's meaning before adding a field

A new behaviour changes what a field means, or is derived from what is there, before it adds a
field, since a limb level has many already. So the span's two fields became one start with a
constant jitter; a facing and a skew replaced the divergence, competition placing the children
(D26); wander became kinks at a fixed spacing rather than gaining a smoothness and a spacing (D17).
A predictable range beats a guessed curve until a model is worth exposing (X22). Left out
until a preset needs them: a jitter per level (a fir's even whorls), and a sharply bare stretch
such as a bare leader tip.

### G4 — Put a new field in an operator

Operators layer the species: a plant grows from its levels' own fields, and each operator is a
layer an author adds when wanted, showing no rows while off. So a field a behaviour needs goes in
the operator the behaviour belongs to, or in a new operator, before it goes on the level, where
every author meets it. Where G1 finds no field to change, this says where the new one goes. So
leaf size by Bias (C138) is a Bias setting, shown only where Bias is on.

A part the algorithm cannot do without stays on the level. Every limb has a radius, and a radius
of zero means nothing, so a Radius operator would override a default the author never chose,
which no panel can explain.

### G5 — Where the algorithm decides, offer one deviation, not an override

A control that the model's own choice overrules promises what it cannot deliver: the author moves
it and sees nothing, or sees another control's effect. Where the algorithm decides, as crowding
places a limb around its parent and Aim turns it toward the outline, the author gets at most one
control of how far to stray from that choice, named by its effect, and no direct setting; an
override per limb or per choice would multiply the controls past what a panel can hold. Seedling
went the other way, the author drawing every angle at random: an organic look, but limbs sticking
out at odd angles. So C134 turns skew, a random turn after crowding's pick, into an irregularity,
and leaves facing off the trunk, which has no upper side to measure it from.

### G6 — Count a change's cost to the finished trees

The author's finished trees are work a change can undo, in two ways: a file stops reading as it
did, losing values to pruning (C124), or it reads the same but grows a different tree, which
nothing reports but the golden trees (C146). So a change to the species or the generator is one
of three, in order of preference:

- **Additive**: a new field or operator whose default or off state grows today's plant. Free.
  Curves (C145) were taken for additive until Size's number was seen to become a curve.
- **Migrated**: a field's type or meaning changes, and a script converts `reference/` in the
  same chunk, as `scripts/from_seedling.py` converted seedling's files, so no file falls behind.
- **Output changed**: the golden trees fail, and the author looks at each: close enough, saved
  again as it is; retuned; or the change goes behind an option.

Migrate early: before the freeze (C113) the items that need it land while the trees are few. A
patch (X22) is a species of its own kind, tagged apart in the glTF extra, so it costs today's
files nothing.

### G2 — Vary a value where its instances are many

A value drawn once per plant is invisible within a tree and differs only from seed to seed, so it
varies only where trees of one species should differ: the envelope's lean and squash, the trunk's
lean direction. The trunk varies in nothing else. A limb's angle and reach vary per limb, the most
visible variety, and its skew is a turn either way of up to its value. Not varied: sag and rise,
since wander already irregularizes; wander, itself the randomness; radius, a share of the
parent's (D22); sides, which are mesh detail; and start, since each node's jitter varies the places.
