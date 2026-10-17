//! Foliage (D27): a leaf cluster at each branch end.

use std::f32::consts::TAU;

use bevy::math::{Quat, Vec3};

use super::{
    envelope::Envelope,
    skeleton::{Axis, Card, upper_side},
};
use crate::species::{Species, Upright};

/// Where a card's axial droop bends it, as a share of its length from the base. Halfway, as
/// seedling's grid has it, until the painted leaf (C38) can say where its sprig parts.
pub(super) const AXIAL_FOLD: f32 = 0.5;

/// The elevation above level from which a tip is steep, where one card along the twig would
/// begin to stand on end from the game's camera.
const STEEP: f32 = 45.0_f32.to_radians();

/// The share of the angle between a twig and the level heading of one of its rosette's cards
/// that the card turns, from the twig toward that heading.
const ROSETTE_LEAN: f32 = 0.75;

/// A card's width and length: a share of the crown's radius, so a scaled species keeps its form.
pub(super) fn card_size(species: &Species) -> f32 {
    species.ornaments.tip_leaves.size * Envelope::of(species).radius.max(0.01)
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

/// The leaf clusters, unless leaves are off: at the tip of each limb bearing none, wherever it
/// ends, so no branch end is bare unless the species leaves steep ones so. A tip bears one card,
/// or on a steep twig what the species' `Upright` says (`cards_at`). One card reaches back along the twig, its fold at the tip, so the sprig's stem
/// overlaps the twig's end, as seedling's did; on a twig shorter than that, from its base. The
/// twig runs straight there (`wander_axis`), so the card lies along it. A rosette's cards start at
/// the tip, spaced evenly around the twig from the side it points toward, each turned from the
/// twig by `ROSETTE_LEAN` of the way to level, keeping its heading, so an upright tip reads as
/// leaves splayed toward the light rather than one card on end.
pub(super) fn clusters(species: &Species, axes: &[Axis]) -> Vec<Card> {
    if !species.ornaments.leaves {
        return Vec::new();
    }
    let size = card_size(species);
    let mut cards = Vec::new();
    for (index, twig) in axes.iter().enumerate() {
        if twig.level == 0 || !twig.scions.is_empty() {
            continue;
        }
        let length = twig.length();
        let tip = twig.frame_at(length);
        let count = cards_at(
            tip.tangent.y.clamp(-1.0, 1.0).asin(),
            species.ornaments.tip_leaves.upright,
        );
        if count == 1 {
            let back = (AXIAL_FOLD * size).min(length);
            let t = (1.0 - back / length.max(f32::EPSILON)).clamp(0.0, 1.0);
            let at = t * length;
            cards.push(Card {
                axis: index,
                at,
                direction: twig.frame_at(at).tangent,
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
            });
        }
    }
    cards
}
