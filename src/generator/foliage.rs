//! Foliage (D27): a card for the sprig at each branch end.

use bevy::math::{Vec2, Vec3};

use super::{
    mesh::Geometry,
    skeleton::{Axis, Frame, upper_side},
};
use crate::species::Foliage;

/// Corners in one card: four quads of four, since each has its own normal (`card`).
pub const CARD_CORNERS: usize = 16;

/// Where a card's axial droop bends it, as a share of its length from the base. Halfway, as
/// seedling's grid has it, until the painted leaf (C38) can say where its sprig parts.
pub(super) const AXIAL_FOLD: f32 = 0.5;

/// Leaf clusters on `twigs`, the limbs bearing none: one card (`card`) at each twig's tip,
/// wherever it ends, so no branch end is bare. It reaches back along the twig, its fold at the
/// tip, so the sprig's stem overlaps the twig's end, as seedling's did; on a twig shorter than
/// that, from its base. The twig runs straight there (`grow_limb`), so the card lies along it.
/// Its size is a share of `crown`, the crown's radius.
pub(super) fn foliage<'a>(
    crown: f32,
    foliage: &Foliage,
    twigs: impl Iterator<Item = &'a Axis>,
) -> Geometry {
    let size = foliage.size * crown.max(0.01);
    let mut leaves = Geometry::default();
    if !foliage.leaves {
        return leaves;
    }
    for twig in twigs {
        let tip = &twig.rings[twig.rings.len() - 1];
        let back = (AXIAL_FOLD * size).min(tip.at);
        let frame = twig.frame(1.0 - back / tip.at.max(f32::EPSILON));
        card(&mut leaves, &frame, size, foliage);
    }
    leaves
}

/// Adds a square card `size` across for a sprig growing from `frame`: its base on the twig, its
/// axis along the twig, so the sprig's painted stem lines up with it, and its face toward the
/// twig's upper side, world up made square to the twig, so it is turned as near up as the twig
/// allows. The texture's bottom is the base.
///
/// The card is two by two quads, after seedling, folded twice: each half turns down about the
/// midrib by the lateral droop (up, cupping, if negative), and the far half bends down about the
/// line across at `AXIAL_FOLD` by the axial droop. The folds shape the outline but not the
/// shading: every quad keeps the flat card's normal, so a facet turned from the light is not cut
/// dark against its neighbours. The corners go near quads before far, left before right, each
/// quad's counter-clockwise from its face starting at its corner nearest the base on the left.
pub(super) fn card(geometry: &mut Geometry, frame: &Frame, size: f32, foliage: &Foliage) {
    let axis = frame.tangent;
    let normal = upper_side(axis, frame.side);
    // Square to both, so corners run counter-clockwise seen from the face.
    let across = axis.cross(normal);
    let (lateral_sin, lateral_cos) = foliage.lateral_droop.sin_cos();
    let (axial_sin, axial_cos) = foliage.axial_droop.sin_cos();
    // A point on the flat card, `u` across from the midrib and `v` along from the base, both in
    // card widths, folded and placed.
    let fold = |u: f32, v: f32| {
        let mut out = Vec3::new(u * lateral_cos, -u.abs() * lateral_sin, v);
        if v > AXIAL_FOLD {
            let beyond = Vec2::new(out.z - AXIAL_FOLD, out.y);
            out.z = AXIAL_FOLD + beyond.x * axial_cos + beyond.y * axial_sin;
            out.y = beyond.y * axial_cos - beyond.x * axial_sin;
        }
        frame.point + (across * out.x + normal * out.y + axis * out.z) * size
    };
    for (v0, v1) in [(0.0, AXIAL_FOLD), (AXIAL_FOLD, 1.0)] {
        for (u0, u1) in [(-0.5, 0.0), (0.0, 0.5)] {
            let first = geometry.positions.len() as u32;
            let corners = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)].map(|(u, v)| (u, v, fold(u, v)));
            for (u, v, position) in corners {
                geometry.positions.push(position);
                geometry.normals.push(normal);
                geometry.uvs.push(Vec2::new(u + 0.5, 1.0 - v));
            }
            geometry
                .indices
                .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
        }
    }
}
