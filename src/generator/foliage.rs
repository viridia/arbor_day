//! Foliage (D27): a leaf cluster at each branch end.

use super::{
    envelope::Envelope,
    skeleton::{Axis, Card},
};
use crate::species::Species;

/// Where a card's axial droop bends it, as a share of its length from the base. Halfway, as
/// seedling's grid has it, until the painted leaf (C38) can say where its sprig parts.
pub(super) const AXIAL_FOLD: f32 = 0.5;

/// A card's width and length: a share of the crown's radius, so a scaled species keeps its form.
pub(super) fn card_size(species: &Species) -> f32 {
    species.foliage.size * Envelope::of(species).radius.max(0.01)
}

/// The leaf clusters, unless leaves are off: one at the tip of each limb bearing none, wherever
/// it ends, so no branch end is bare. Its card reaches back along the twig, its fold at the tip,
/// so the sprig's stem overlaps the twig's end, as seedling's did; on a twig shorter than that,
/// from its base. The twig runs straight there (`wander_axis`), so the card lies along it.
pub(super) fn clusters(species: &Species, axes: &[Axis]) -> Vec<Card> {
    if !species.foliage.leaves {
        return Vec::new();
    }
    let size = card_size(species);
    (axes.iter().enumerate())
        .filter(|(_, axis)| axis.level > 0 && axis.scions.is_empty())
        .map(|(index, twig)| {
            let length = twig.length();
            let back = (AXIAL_FOLD * size).min(length);
            let t = (1.0 - back / length.max(f32::EPSILON)).clamp(0.0, 1.0);
            Card {
                axis: index,
                at: t * length,
            }
        })
        .collect()
}
