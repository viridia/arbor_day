# Guidelines

How this project makes the judgements it makes repeatedly. Read the first group before scoping any
chunk, the second before scoping one that adds to the species.

**What this document admits.** Advice a maintainer would otherwise have to learn by being corrected.
The test is whether someone new, following it, would choose differently than without it. A guideline
may be departed from, with the reason stated where the departure is made ("departs from G1 because
…"); a requirement in [requirements.md](requirements.md) may not. Why a particular choice was made
belongs in [decisions.md](decisions.md).

**Where a guideline comes from.** A correction the author has had to make more than once, written
with the case that produced it, since the case is what lets a reader recognize the next one.

**Numbering.** Each guideline's number is a permanent identity, never reused. A gap in the sequence
is a retired guideline.

**Next: 4.**

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
A predictable range beats a guessed curve until a model is worth exposing (X18, X22). Left out
until a preset needs them: a jitter per level (a fir's even whorls), and a sharply bare stretch
such as a bare leader tip.

### G2 — Vary a value where its instances are many

A value drawn once per plant is invisible within a tree and differs only from seed to seed, so it
varies only where trees of one species should differ: the envelope's lean and squash, the trunk's
lean direction. The trunk varies in nothing else. A limb's angle and reach vary per limb, the most
visible variety, and its skew is a turn either way of up to its value. Not varied: bend toward and
bend, since wander already irregularizes; wander, itself the randomness; radius, a share of the
parent's (D22); sides, which are mesh detail; and start, since each node's jitter varies the places.
