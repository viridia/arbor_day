//! Foliage (D27): a leaf cluster at each branch end.

use std::f32::consts::TAU;

use bevy::math::{Quat, Vec3};

use super::{
    envelope::Envelope,
    leaf,
    skeleton::{Axis, AxisKind, Card, upper_side},
};
use crate::species::{Species, Upright};

/// Where a card's axial droop bends it, as a share of its length from the base: where the painted
/// sprig's fan starts, as seedling's did.
pub(super) fn axial_fold(species: &Species) -> f32 {
    let tip_leaves = &species.ornaments.tip_leaves.settings;
    leaf::layout(tip_leaves, species.seed, species.leaf_stroke()).fold
}

/// How far back from its tip an axis runs straight, under its tip's card (D17), whether or not
/// leaves grow, so turning them off leaves the branches alone.
pub(super) fn straight_run(species: &Species) -> f32 {
    axial_fold(species) * card_size(species)
}

/// The elevation above level from which a tip is steep, where one card along the twig would
/// begin to stand on end from the game's camera.
const STEEP: f32 = 45.0_f32.to_radians();

/// The share of the angle between a twig and the level heading of one of its rosette's cards
/// that the card turns, from the twig toward that heading.
const ROSETTE_LEAN: f32 = 0.75;

/// A full card's width and length: a share of the crown's radius, so a scaled species keeps its
/// form.
pub(super) fn card_size(species: &Species) -> f32 {
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

/// How many cards a tip whose twig rises at `elevation` bears: one, unless it is `STEEP`, when
/// `upright` says, a count of 2 or 3 being a rosette.
pub(super) fn cards_at(elevation: f32, upright: Upright) -> u32 {
    match upright {
        _ if elevation < STEEP => 1,
        Upright::Bare => 0,
        Upright::Single => 1,
        Upright::Double => 2,
        Upright::Triple => 3,
    }
}

/// The leaf clusters, if the tip leaves are on: at the tip of each stem or limb that did not fork,
/// whether or not it bears limbs, so no branch end is bare unless the species leaves steep ones so. A tip bears one card,
/// or on a steep twig what the species' `Upright` says (`cards_at`). One card reaches back along the twig, its fold at the tip, so the sprig's stem
/// overlaps the twig's end, as seedling's did; on a twig shorter than that, from behind its root,
/// so a short twig's leaves stay at its tip, close in to its parent. The twig runs straight there (`wander_axis`), so the card lies along it. A rosette's cards start at
/// the tip, spaced evenly around the twig from the side it points toward, each turned from the
/// twig by `ROSETTE_LEAN` of the way to level, keeping its heading, so an upright tip reads as
/// leaves splayed toward the light rather than one card on end. Each tip's cards are scaled by
/// `tip_scale`.
pub(super) fn clusters(species: &Species, axes: &[Axis]) -> Vec<Card> {
    let Some(tip_leaves) = species.ornaments.tip_leaves.get() else {
        return Vec::new();
    };
    let full = card_size(species);
    let fold = axial_fold(species);
    let mut cards = Vec::new();
    for (index, twig) in axes.iter().enumerate() {
        if twig.bears(axes, AxisKind::ForkCopy) {
            continue;
        }
        let size = full * tip_scale(axes, twig, tip_leaves.smallest);
        let length = twig.length();
        let tip = twig.frame_at(length);
        let count = cards_at(tip.tangent.y.clamp(-1.0, 1.0).asin(), tip_leaves.upright);
        if count == 1 {
            let at = length - fold * size;
            cards.push(Card {
                axis: index,
                at,
                direction: twig.frame_at(at).tangent,
                size,
            });
            continue;
        }
        // Level, the way the twig points; straight up, its upper side's opposite.
        let heading = -upper_side(tip.tangent, tip.side).reject_from_normalized(Vec3::Y);
        let heading = heading.normalize_or(Vec3::X);
        for card in 0..count {
            let turn = Quat::from_rotation_y(TAU * card as f32 / count as f32);
            cards.push(Card {
                axis: index,
                at: length,
                direction: tip.tangent.slerp(turn * heading, ROSETTE_LEAN),
                size,
            });
        }
    }
    cards
}
