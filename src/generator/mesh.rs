//! Meshing a grown plant: a tube along each axis, and the foliage's cards.

use std::f32::consts::TAU;

use bevy::math::{Vec2, Vec3};

use super::{
    envelope::Envelope,
    foliage::foliage,
    skeleton::{AxisKind, Ring, grow},
};
use crate::species::Species;

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
    /// One card for each leaf cluster, textured with the leaf.
    pub leaves: Geometry,
    /// Limbs grown on each grown level.
    pub level_limbs: Vec<usize>,
}

/// The whole plant: the stems and their limbs, none drawn narrower than the bark's detail
/// threshold, and the foliage (`foliage`) on the limbs bearing none. Each stem is a tube left open
/// at the base, since it stands in the ground. A tube is closed at the top by a fan, unless an axis
/// carries it on: that axis starts on its top ring, at its top radius.
pub fn plant(species: &Species) -> Grown {
    let axes = grow(species);
    let min_radius = species.detail * 0.5;
    let levels = species.grown_levels();
    let mut bark = Geometry::default();
    let mut level_limbs = vec![0; levels.len()];
    for axis in &axes {
        let sides = match axis.level {
            0 => species.trunk.sides,
            level => levels[level - 1].sides,
        };
        let (base, top) = axis.radii;
        let radii = (base.max(min_radius), top.max(min_radius));
        let cap = !axis.bears(&axes, AxisKind::CarryOn);
        tube(&mut bark, &axis.rings, radii, sides.max(3) as u32, cap);
        if axis.level > 0 {
            level_limbs[axis.level - 1] += 1;
        }
    }
    let ends = (axes.iter()).filter(|axis| axis.level > 0 && axis.scions.is_empty());
    Grown {
        bark,
        leaves: foliage(Envelope::of(species).radius, &species.foliage, ends),
        level_limbs,
    }
}

/// Adds a tube through `rings`, each turned to its ring's side, so the tube rolls only as the
/// sides do. The radius narrows evenly from `radii.0` at the start to `radii.1` at the end, which a
/// fan closes if `cap` is set. The start is left open.
pub(super) fn tube(
    geometry: &mut Geometry,
    rings: &[Ring],
    radii: (f32, f32),
    sides: u32,
    cap: bool,
) {
    let first = geometry.positions.len() as u32;
    let (base_radius, top_radius) = radii;
    let length = rings[rings.len() - 1].at.max(1e-4);
    // A tapering tube's surface leans in toward the end, so its normals lean forward by the same
    // angle.
    let (lean_sin, lean_cos) = ((base_radius - top_radius) / length).atan().sin_cos();

    for ring in rings {
        let across = ring.side.cross(ring.tangent);
        let radius = base_radius + (top_radius - base_radius) * ring.at / length;
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
