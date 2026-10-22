//! The crown's envelope: its shape, where rays leave it, and its helper for the preview.

use std::f32::consts::TAU;

use bevy::math::{Vec2, Vec3};

use crate::species::{Profile, Species};

/// Halvings in finding where a ray leaves the envelope: the largest envelope's 90 m bound to under
/// a tenth of a millimetre.
pub(super) const ENVELOPE_BISECTIONS: u32 = 20;

/// Points down each meridian of the envelope's helper, base to top.
const HELPER_STEPS: u32 = 16;
/// Points along the profile in finding a point on the outline (`outline_point`).
const OUTLINE_STEPS: u32 = 32;
/// Meridians around the envelope's helper.
const HELPER_MERIDIANS: u32 = 8;
/// A ring around the helper every this many steps.
const HELPER_RING_EVERY: u32 = 2;
/// Segments in each ring of the helper.
const HELPER_RING_SIDES: u32 = 24;

/// The crown's envelope in meters, as the species' crown and trunk height place it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Envelope {
    pub profile: Profile,
    /// The heights of the crown's base and top above the ground.
    pub base: f32,
    pub top: f32,
    /// The radius where the crown is widest.
    pub radius: f32,
}

impl Envelope {
    pub fn of(species: &Species) -> Self {
        let (height, crown) = (species.trunk.height.max(0.01), &species.crown);
        let top = height * crown.top.max(0.01);
        Self {
            profile: crown.profile,
            base: (height * crown.base).clamp(0.0, top),
            top,
            radius: crown.radius,
        }
    }
}

#[cfg(test)]
impl Default for Envelope {
    fn default() -> Self {
        Self::of(&Species::default())
    }
}

/// The envelope's width at `t`, 0 at the crown base to 1 at the top, as a fraction of its radius.
pub(super) fn profile_width(profile: Profile, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match profile {
        Profile::Cone => 1.0 - t,
        Profile::Ellipsoid => (1.0 - (2.0 * t - 1.0).powi(2)).max(0.0).sqrt(),
        Profile::Hemisphere => (1.0 - t * t).max(0.0).sqrt(),
        Profile::Column => 1.0,
        // Weber and Penn's flame.
        Profile::Flame if t <= 0.7 => t / 0.7,
        Profile::Flame => (1.0 - t) / 0.3,
    }
}

/// The envelope's helper, as the preview draws it: meridians closed across the base and the top,
/// and rings around it. Pairs of points, a segment each.
pub fn envelope_helper(envelope: &Envelope) -> Vec<Vec3> {
    let Envelope { base, top, .. } = *envelope;
    let profile: Vec<(f32, f32)> = (0..=HELPER_STEPS)
        .map(|step| {
            let t = step as f32 / HELPER_STEPS as f32;
            let width = envelope.radius * profile_width(envelope.profile, t);
            (width, base + (top - base) * t)
        })
        .collect();
    let mut lines = Vec::new();
    let mut segment = |a: Vec3, b: Vec3| {
        // A profile that closes to a point leaves no spoke at that end.
        if a.distance_squared(b) > 1e-8 {
            lines.extend([a, b]);
        }
    };

    for meridian in 0..HELPER_MERIDIANS {
        let (sin, cos) = (TAU * meridian as f32 / HELPER_MERIDIANS as f32).sin_cos();
        let point = |(width, y): (f32, f32)| Vec3::new(cos * width, y, sin * width);
        segment(Vec3::Y * base, point(profile[0]));
        for pair in profile.windows(2) {
            segment(point(pair[0]), point(pair[1]));
        }
        segment(point(profile[profile.len() - 1]), Vec3::Y * top);
    }

    for &(width, y) in profile.iter().step_by(HELPER_RING_EVERY as usize) {
        for side in 0..HELPER_RING_SIDES {
            let around = |side: u32| {
                let (sin, cos) = (TAU * side as f32 / HELPER_RING_SIDES as f32).sin_cos();
                Vec3::new(cos * width, y, sin * width)
            };
            segment(around(side), around(side + 1));
        }
    }
    lines
}

/// A point on the envelope's outline, seen in a vertical slice through the axis, as its distance
/// from the axis and its height. The outline runs from where the profile is widest, up its side,
/// then across the top to the axis. `along` is how far along it by length, 0 at its start and 1
/// at the axis.
pub(super) fn outline_point(envelope: &Envelope, along: f32) -> (f32, f32) {
    let Envelope { base, top, .. } = *envelope;
    let mut points: Vec<Vec2> = (0..=OUTLINE_STEPS)
        .map(|step| {
            let t = step as f32 / OUTLINE_STEPS as f32;
            let width = envelope.radius * profile_width(envelope.profile, t);
            Vec2::new(width, base + (top - base) * t)
        })
        .collect();
    let widest = (points.iter().enumerate()).fold(
        0,
        |best, (i, p)| if p.x > points[best].x { i } else { best },
    );
    points.drain(..widest);
    points.push(Vec2::new(0.0, top));
    let total: f32 = points
        .windows(2)
        .map(|pair| pair[0].distance(pair[1]))
        .sum();
    let mut left = along.clamp(0.0, 1.0) * total;
    for pair in points.windows(2) {
        let length = pair[0].distance(pair[1]);
        if left <= length && length > 0.0 {
            let point = pair[0].lerp(pair[1], left / length);
            return (point.x, point.y);
        }
        left -= length;
    }
    (0.0, top)
}

/// The envelope's radius at height `y`: 0 below the crown base and above the top.
pub(super) fn envelope_radius(envelope: &Envelope, y: f32) -> f32 {
    let Envelope { base, top, .. } = *envelope;
    if y < base || y > top || top <= base {
        return 0.0;
    }
    envelope.radius * profile_width(envelope.profile, (y - base) / (top - base))
}

/// How far a ray from `origin` along the unit `direction` runs before it leaves the envelope, or 0
/// if `origin` is outside it. Every profile makes a convex solid, so the ray leaves it once, and
/// bisection between the origin and a point beyond its bounds finds where.
pub(super) fn distance_to_envelope(envelope: &Envelope, origin: Vec3, direction: Vec3) -> f32 {
    let inside = |p: Vec3| inside_envelope(envelope, p);
    if !inside(origin) {
        return 0.0;
    }
    // Further than any two points of the envelope are apart.
    let (mut near, mut far) = (0.0, 2.0 * (envelope.top.abs() + envelope.radius.abs()));
    for _ in 0..ENVELOPE_BISECTIONS {
        let middle = (near + far) * 0.5;
        if inside(origin + direction * middle) {
            near = middle;
        } else {
            far = middle;
        }
    }
    near
}

/// Whether `point` lies inside the envelope.
pub(super) fn inside_envelope(envelope: &Envelope, point: Vec3) -> bool {
    point.x.hypot(point.z) < envelope_radius(envelope, point.y)
}
