//! Meshing a grown plant: a tube along each axis, and the foliage's cards.

use std::f32::consts::TAU;

use bevy::math::{Vec2, Vec3};

use super::{
    foliage::{AXIAL_FOLD, card_size},
    skeleton::{AxisKind, Frame, Ring, grow, upper_side},
};
use crate::species::{Foliage, Species};

/// Corners in one card: four quads of four, since each has its own normal (`card`).
pub const CARD_CORNERS: usize = 16;

/// Triangles with one normal per vertex. Every triangle meeting at a vertex shares it, so normals
/// are continuous across the surface and the outline shell has no gaps.
#[derive(Debug, Default)]
pub struct Geometry {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    /// Texture coordinates, one per vertex, or none for an untextured mesh.
    pub uvs: Vec<Vec2>,
    /// Counter-clockwise seen from outside, as glTF and Bevy take front faces.
    pub indices: Vec<u32>,
}

impl Geometry {
    pub fn triangles(&self) -> usize {
        self.indices.len() / 3
    }
}

/// A plant as generated.
#[derive(Debug, Default)]
pub struct Grown {
    pub bark: Geometry,
    /// The leaf cards, textured with the leaf.
    pub leaves: Geometry,
    /// Branch ends bearing leaves, one card each or a rosette.
    pub leafy_tips: usize,
    /// Limbs grown on each grown level.
    pub level_limbs: Vec<usize>,
}

/// The whole plant, grown (`grow`) and meshed: the stems and their limbs, none drawn narrower than
/// the bark's detail threshold, and a card (`card`) for each leaf cluster. Each stem is a tube
/// left open at the base, since it stands in the ground. A tube is closed at the top by a fan,
/// unless an axis carries it on: that axis starts on its top ring, at its top radius.
pub fn plant(species: &Species) -> Grown {
    let skeleton = grow(species);
    let axes = &skeleton.axes;
    let min_radius = species.detail * 0.5;
    let levels = species.grown_levels();
    let mut bark = Geometry::default();
    let mut level_limbs = vec![0; levels.len()];
    for axis in axes {
        let sides = match axis.level {
            0 => species.trunk.sides,
            level => levels[level - 1].sides,
        };
        let cap = !axis.bears(axes, AxisKind::CarryOn);
        tube(&mut bark, &axis.rings, min_radius, sides.max(3) as u32, cap);
        if axis.level > 0 {
            level_limbs[axis.level - 1] += 1;
        }
    }
    let size = card_size(species);
    let mut leaves = Geometry::default();
    let mut leafy_tips = 0;
    let mut last_axis = None;
    for cluster in &skeleton.cards {
        if last_axis != Some(cluster.axis) {
            leafy_tips += 1;
            last_axis = Some(cluster.axis);
        }
        let twig = axes[cluster.axis].frame_at(cluster.at);
        let frame = Frame {
            tangent: cluster.direction,
            ..twig
        };
        card(&mut leaves, &frame, size, &species.foliage);
    }
    Grown {
        bark,
        leaves,
        leafy_tips,
        level_limbs,
    }
}

/// Adds a square card `size` across for a sprig growing from `frame`: its base at the frame's
/// point, its axis along the frame's tangent, so a sprig along its twig lines up with it, and its
/// face toward the axis's upper side, world up made square to it, so it is turned as near up as
/// the axis allows. The texture's bottom is the base.
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

/// Adds a tube through `rings`, each turned to its ring's side, so the tube rolls only as the
/// sides do, and drawn at its ring's radius, but none narrower than `min_radius`. The end is closed
/// by a fan if `cap` is set; the start is left open.
pub(super) fn tube(
    geometry: &mut Geometry,
    rings: &[Ring],
    min_radius: f32,
    sides: u32,
    cap: bool,
) {
    let first = geometry.positions.len() as u32;
    let drawn = |ring: &Ring| ring.radius.max(min_radius);
    for (i, ring) in rings.iter().enumerate() {
        // A narrowing tube's surface leans in, so its normals lean forward by the same angle,
        // measured between the rings either side.
        let (before, after) = (
            &rings[i.saturating_sub(1)],
            &rings[(i + 1).min(rings.len() - 1)],
        );
        let slope = (drawn(before) - drawn(after)) / (after.at - before.at).max(1e-4);
        let (lean_sin, lean_cos) = slope.atan().sin_cos();
        let across = ring.side.cross(ring.tangent);
        let radius = drawn(ring);
        for side in 0..sides {
            let (sin, cos) = (TAU * side as f32 / sides as f32).sin_cos();
            let outward = ring.side * cos + across * sin;
            geometry.positions.push(ring.center + outward * radius);
            geometry
                .normals
                .push((outward * lean_cos + ring.tangent * lean_sin).normalize());
        }
    }
    let segments = rings.len() as u32 - 1;
    for ring in 0..segments {
        for side in 0..sides {
            let a = first + ring * sides + side;
            let b = first + ring * sides + (side + 1) % sides;
            let (c, d) = (a + sides, b + sides);
            geometry.indices.extend([a, c, d, a, d, b]);
        }
    }
    if !cap {
        return;
    }

    let end = rings[rings.len() - 1];
    let tip = geometry.positions.len() as u32;
    geometry.positions.push(end.center);
    geometry.normals.push(end.tangent);
    let last = first + segments * sides;
    for side in 0..sides {
        geometry
            .indices
            .extend([last + side, tip, last + (side + 1) % sides]);
    }
}
