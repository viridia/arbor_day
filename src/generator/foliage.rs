//! Foliage (D27): a leaf cluster at each branch end.

use std::f32::consts::TAU;

use bevy::math::{Quat, Vec3};

use super::{
    envelope::Envelope,
    leaf::{self, CardShape},
    skeleton::{Axis, AxisKind, Card, FOLIAGE_STREAM, upper_side},
};
use crate::species::{Species, Upright};

/// The card holding the sprig, in leaf lengths (`leaf::CardShape`). Its axial droop bends it at
/// its fold, where the painted sprig's fan starts, as seedling's did.
pub(super) fn card_shape(species: &Species) -> CardShape {
    let tip_leaves = &species.ornaments.tip_leaves.settings;
    leaf::layout(tip_leaves, species.seed, species.leaf_stroke()).card
}

/// How far back from its tip an axis runs straight, under its tip's card (D17), whether or not
/// leaves grow, so turning them off leaves the branches alone: the card's stretch below the fan,
/// at full size.
pub(super) fn straight_run(species: &Species) -> f32 {
    below_fan(&card_shape(species)) * leaf_length(species)
}

/// The card's stretch below the fan, in leaf lengths, which reaches back along the twig.
pub(super) fn below_fan(card: &CardShape) -> f32 {
    card.fold * card.height
}

/// The tilt from straight up below which a tip is steep, where one card along the twig would
/// begin to stand on end from the game's camera.
const STEEP: f32 = 45.0_f32.to_radians();

/// How far above level an upright twig's rosette cards rise: three quarters of the way from the
/// twig to level, so the tip reads as leaves splayed toward the light rather than one card on end.
const UPRIGHT_RISE: f32 = 22.5_f32.to_radians();

/// The turn between neighbouring cards of an upright rosette, three around the twig: four read
/// as an orthogonal cross.
const ROSETTE_SPACING: f32 = 120.0_f32.to_radians();

/// The least turn about the twig between neighbouring cards of a closing rosette, and how far
/// each tip's strays from it, as a share, so tips drop cards at different tilts.
const LEAST_SPACING: f32 = 60.0_f32.to_radians();
const SPACING_NUDGE: f32 = 0.3;

/// The middle leaf's length at full size: a share of the crown's radius, so a scaled species keeps
/// its form. The card around the sprig follows from it (`card_shape`).
pub(super) fn leaf_length(species: &Species) -> f32 {
    species.ornaments.tip_leaves.settings.size * Envelope::of(species).radius.max(0.01)
}

/// The share of the full size of the cards at `twig`'s tip: its last lateral's `leaf_scale`, as
/// its last node's laterals take, or its own if it bears none, clamped between `smallest` (the
/// tip leaves') and 1, so the straight run under a full card covers every card.
pub(super) fn tip_scale(axes: &[Axis], twig: &Axis, smallest: f32) -> f32 {
    let laterals = twig.scions.iter().map(|&scion| &axes[scion]);
    let last = (laterals.filter(|scion| scion.kind == AxisKind::Lateral))
        .filter_map(|scion| Some((scion.parent?.1, scion.leaf_scale)))
        .max_by(|a, b| a.0.total_cmp(&b.0));
    let scale = last.map_or(twig.leaf_scale, |(_, scale)| scale);
    scale.clamp(smallest, 1.0)
}

/// The cards of a rosette whose twig is tilted `closing` of the way from steep to upright (0 at
/// `STEEP`, 1 straight up), as turns about world up from the twig's heading. Upright, three
/// `ROSETTE_SPACING` apart at a seeded roll, the middle one nearest the heading; tilting, each
/// turn shrinks with `closing`, so the rosette closes toward the heading as a fan. A card goes
/// once the fan's span cannot hold its count `least` apart: the middle first, the ends keeping
/// their turns, then the end further from the heading, so no card jumps when one goes.
pub(super) fn rosette_turns(closing: f32, roll: f32, least: f32) -> Vec<f32> {
    let middle = roll.rem_euclid(ROSETTE_SPACING) - ROSETTE_SPACING / 2.0;
    let ends = [middle - ROSETTE_SPACING, middle + ROSETTE_SPACING];
    let span = 2.0 * ROSETTE_SPACING * closing;
    let turns = if span >= 2.0 * least {
        vec![ends[0], middle, ends[1]]
    } else if span >= least {
        ends.to_vec()
    } else {
        vec![ends[usize::from(middle < 0.0)]]
    };
    turns.into_iter().map(|turn| turn * closing).collect()
}

/// The leaf clusters, if the tip leaves are on: at the tip of each stem or limb that did not fork,
/// whether or not it bears limbs, so no branch end is bare unless the species leaves steep ones so.
/// A tip bears one card, or on a steep twig what the species' `Upright` says. One card reaches
/// back along the twig, its fold at the tip, so the sprig's stem overlaps the twig's end, as
/// seedling's did; on a twig shorter than that, from behind its root, so a short twig's leaves
/// stay at its tip, close in to its parent. The twig runs straight there (`wander_axis`), so the
/// card lies along it.
///
/// A rosette's cards (`rosette_turns`) each point at its own heading, `UPRIGHT_RISE` above level
/// on an upright twig. As the twig tilts, they rise toward the twig's own elevation at `STEEP`,
/// whatever the twig's, so they stay low rather than standing with it, and reach back from the
/// tip, so at `STEEP` the last is the one card along the twig. Each tip's roll and least spacing
/// come from its foliage stream. Each tip's cards are scaled by `tip_scale`.
pub(super) fn clusters(species: &Species, axes: &[Axis]) -> Vec<Card> {
    let Some(tip_leaves) = species.ornaments.tip_leaves.get() else {
        return Vec::new();
    };
    let full = leaf_length(species);
    let below = below_fan(&card_shape(species));
    let mut cards = Vec::new();
    for (index, twig) in axes.iter().enumerate() {
        if twig.bears(axes, AxisKind::ForkCopy) {
            continue;
        }
        let size = full * tip_scale(axes, twig, tip_leaves.smallest);
        let length = twig.length();
        let tip = twig.frame_at(length);
        let tilt = tip.tangent.y.clamp(-1.0, 1.0).acos();
        let along = |at: f32| Card {
            axis: index,
            at,
            direction: twig.frame_at(at).tangent,
            size,
        };
        match tip_leaves.upright {
            _ if tilt >= STEEP => cards.push(along(length - below * size)),
            Upright::Bare => {}
            Upright::Single => cards.push(along(length - below * size)),
            Upright::Rosette => {
                let closing = 1.0 - tilt / STEEP;
                let mut rng = twig.rng.fork(FOLIAGE_STREAM);
                let roll = rng.unit() * TAU;
                let least = LEAST_SPACING * (1.0 + SPACING_NUDGE * (2.0 * rng.unit() - 1.0));
                // Level, the way the twig points; straight up, its upper side's opposite.
                let heading = -upper_side(tip.tangent, tip.side).reject_from_normalized(Vec3::Y);
                let heading = heading.normalize_or(Vec3::X);
                let at = length - below * size * (1.0 - closing);
                let rise = STEEP - (STEEP - UPRIGHT_RISE) * closing;
                for turn in rosette_turns(closing, roll, least) {
                    let level = Quat::from_rotation_y(turn) * heading;
                    cards.push(Card {
                        axis: index,
                        at,
                        direction: level * rise.cos() + Vec3::Y * rise.sin(),
                        size,
                    });
                }
            }
        }
    }
    cards
}
