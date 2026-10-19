//! Where stems meet the ground: the flare widening their base, and the stretch below the ground
//! that keeps a plant standing on a slope from showing a gap on its low side.

use std::f32::consts::FRAC_PI_4;

use bevy::math::Vec3;

use super::{
    hint::simplify,
    skeleton::{AxisKind, Ring, Skeleton},
    trace::{TRACE_STEP, halfway},
};
use crate::species::Species;

/// How far up a stem its flare reaches, in base radii.
const FLARE_HEIGHT: f32 = 3.0;
/// The steepest slope a plant stands on.
const STEEPEST_SLOPE: f32 = FRAC_PI_4;

/// With Flare on, widens each stem rising from the ground by its width share of its base radius
/// at the ground, the widening falling off as the square of the way up to `FLARE_HEIGHT` base
/// radii, so it meets the taper without a crease. Rings are added where the widening's curve lies
/// further than the bark's detail threshold from the rings either side, as hinting keeps them.
pub(super) fn flare(species: &Species, skeleton: &mut Skeleton) {
    let Some(flare) = species.trunk.flare.get() else {
        return;
    };
    let stems = skeleton.axes.iter_mut();
    for axis in stems.filter(|axis| axis.kind == AxisKind::Stem) {
        let base = axis.rings[0].radius;
        let height = (FLARE_HEIGHT * base).min(axis.length());
        let extra = |at: f32| base * flare.width * (1.0 - at / height).max(0.0).powi(2);
        let steps = (height / TRACE_STEP).ceil().max(1.0) as usize;
        let profile: Vec<Vec3> = (0..=steps)
            .map(|step| {
                let at = height * step as f32 / steps as f32;
                Vec3::new(at, extra(at), 0.0)
            })
            .collect();
        let added: Vec<Ring> = (simplify(&profile, species.detail).into_iter())
            .map(|i| profile[i].x)
            .filter(|&at| axis.rings.iter().all(|ring| (ring.at - at).abs() > 1e-4))
            .map(|at| axis.frame_at(at).ring(at))
            .collect();
        axis.rings.extend(added);
        axis.rings.sort_by(|a, b| a.at.total_cmp(&b.at));
        for ring in &mut axis.rings {
            ring.radius += extra(ring.at);
        }
    }
}

/// Extends each stem rising from the ground straight down, by its base radius times the tangent of
/// `STEEPEST_SLOPE`, so on a slope that steep its low side still meets the ground. The added ring's
/// distance along the axis is below 0, and the ground ring faces halfway between the segments
/// meeting there.
pub(super) fn sink(skeleton: &mut Skeleton) {
    let stems = skeleton.axes.iter_mut();
    for axis in stems.filter(|axis| axis.kind == AxisKind::Stem) {
        let ground = axis.rings[0];
        let depth = ground.radius * STEEPEST_SLOPE.tan();
        let below = Ring {
            center: ground.center - Vec3::Y * depth,
            tangent: Vec3::Y,
            side: (ground.side.reject_from_normalized(Vec3::Y)).normalize_or(Vec3::X),
            at: ground.at - depth,
            radius: ground.radius,
        };
        let next = axis.rings[1].center;
        let ring = &mut axis.rings[0];
        ring.tangent = halfway(below.center, ring.center, next, ring.tangent);
        ring.side = (ring.side.reject_from_normalized(ring.tangent)).normalize_or(ring.side);
        axis.rings.insert(0, below);
    }
}
