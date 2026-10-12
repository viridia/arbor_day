//! The generator: a species in, a plant's geometry out. Plain functions on Bevy's math types, with
//! no ECS, so they can move to a library of their own (ledger X1).

use std::f32::consts::TAU;

use bevy::math::Vec3;

use crate::species::{Envelope, Profile, Trunk};

/// The largest turn of a tube's axis between two rings, so rings are spaced by curvature: a
/// straight tube is one segment, and a bent one gets a ring per this much turn.
const MAX_RING_TURN_DEGREES: f32 = 15.0;

/// Points down each meridian of the envelope's outline, base to top.
const OUTLINE_STEPS: u32 = 16;
/// Meridians around the envelope's outline.
const OUTLINE_MERIDIANS: u32 = 8;
/// A ring around the outline every this many steps.
const OUTLINE_RING_EVERY: u32 = 2;
/// Segments in each ring of the outline.
const OUTLINE_RING_SIDES: u32 = 24;

/// Triangles with one normal per vertex. Every triangle meeting at a vertex shares it, so normals
/// are continuous across the surface and the outline shell has no gaps.
#[derive(Debug, Default)]
pub struct Geometry {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    /// Counter-clockwise seen from outside, as glTF and Bevy take front faces.
    pub indices: Vec<u32>,
}

impl Geometry {
    pub fn triangles(&self) -> usize {
        self.indices.len() / 3
    }
}

/// The trunk: a tube of rings rising from the origin, bending toward +X along a circular arc,
/// closed at the top by a fan. The base is left open, since it stands in the ground.
pub fn trunk(trunk: &Trunk) -> Geometry {
    let sides = trunk.sides.max(3) as u32;
    let height = trunk.height.max(0.01);
    let segments = ((trunk.bend.abs() / MAX_RING_TURN_DEGREES.to_radians()).ceil() as u32).max(1);
    let base_radius = trunk.radius;
    let top_radius = trunk.radius * (1.0 - trunk.taper.clamp(0.0, 1.0));
    // A tapering tube's surface leans in toward the top, so its normals lean up by the same angle.
    let (lean_sin, lean_cos) = ((base_radius - top_radius) / height).atan().sin_cos();

    let mut geometry = Geometry::default();
    for ring in 0..=segments {
        let t = ring as f32 / segments as f32;
        let (center, tangent) = arc(height, trunk.bend, t);
        // The arc lies in the XY plane, so Z is square to the axis everywhere and the ring frame
        // never rolls.
        let across = tangent.cross(Vec3::Z);
        let radius = base_radius + (top_radius - base_radius) * t;
        for side in 0..sides {
            let (sin, cos) = (TAU * side as f32 / sides as f32).sin_cos();
            let outward = across * cos + Vec3::Z * sin;
            geometry.positions.push(center + outward * radius);
            geometry
                .normals
                .push((outward * lean_cos + tangent * lean_sin).normalize());
        }
    }
    for ring in 0..segments {
        for side in 0..sides {
            let a = ring * sides + side;
            let b = ring * sides + (side + 1) % sides;
            let (c, d) = (a + sides, b + sides);
            geometry.indices.extend([a, c, d, a, d, b]);
        }
    }

    let (top, tangent) = arc(height, trunk.bend, 1.0);
    let tip = geometry.positions.len() as u32;
    geometry.positions.push(top);
    geometry.normals.push(tangent);
    let last = segments * sides;
    for side in 0..sides {
        geometry
            .indices
            .extend([last + side, tip, last + (side + 1) % sides]);
    }
    geometry
}

/// The envelope's width at `t`, 0 at the crown base to 1 at the top, as a fraction of its radius.
pub fn profile_width(profile: Profile, t: f32) -> f32 {
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

/// The envelope drawn as lines: meridians closed across the base and the top, and rings around
/// it. Pairs of points, a segment each.
pub fn envelope_outline(envelope: &Envelope) -> Vec<Vec3> {
    let (base, top) = crown_span(envelope);
    let profile: Vec<(f32, f32)> = (0..=OUTLINE_STEPS)
        .map(|step| {
            let t = step as f32 / OUTLINE_STEPS as f32;
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

    for meridian in 0..OUTLINE_MERIDIANS {
        let (sin, cos) = (TAU * meridian as f32 / OUTLINE_MERIDIANS as f32).sin_cos();
        let point = |(width, y): (f32, f32)| Vec3::new(cos * width, y, sin * width);
        segment(Vec3::Y * base, point(profile[0]));
        for pair in profile.windows(2) {
            segment(point(pair[0]), point(pair[1]));
        }
        segment(point(profile[profile.len() - 1]), Vec3::Y * top);
    }

    for &(width, y) in profile.iter().step_by(OUTLINE_RING_EVERY as usize) {
        for side in 0..OUTLINE_RING_SIDES {
            let around = |side: u32| {
                let (sin, cos) = (TAU * side as f32 / OUTLINE_RING_SIDES as f32).sin_cos();
                Vec3::new(cos * width, y, sin * width)
            };
            segment(around(side), around(side + 1));
        }
    }
    lines
}

/// The heights of the crown base and the top.
fn crown_span(envelope: &Envelope) -> (f32, f32) {
    let top = envelope.height.max(0.01);
    (top * envelope.crown_base.clamp(0.0, 1.0), top)
}

/// The point and unit tangent at `t` (0 to 1) along an arc of length `length` that starts at the
/// origin going up and turns by `bend` radians toward +X.
fn arc(length: f32, bend: f32, t: f32) -> (Vec3, Vec3) {
    if bend.abs() < 1e-4 {
        return (Vec3::Y * length * t, Vec3::Y);
    }
    let radius = length / bend;
    let (sin, cos) = (bend * t).sin_cos();
    (
        Vec3::new(radius * (1.0 - cos), radius * sin, 0.0),
        Vec3::new(sin, cos, 0.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trunk_with(sides: i32, bend_degrees: f32) -> Trunk {
        Trunk {
            sides,
            bend: bend_degrees.to_radians(),
            ..Trunk::default()
        }
    }

    #[test]
    fn rings_follow_curvature() {
        // Straight: one segment of 6 quads and a 6-triangle cap.
        assert_eq!(trunk(&trunk_with(6, 0.0)).triangles(), 6 * 2 + 6);
        // 90° at 15° a ring: six segments.
        assert_eq!(trunk(&trunk_with(4, 90.0)).triangles(), 4 * 2 * 6 + 4);
    }

    #[test]
    fn vertices_are_shared() {
        let geometry = trunk(&trunk_with(5, 30.0));
        // Two segments: three rings of five, and the tip.
        assert_eq!(geometry.positions.len(), 3 * 5 + 1);
        assert_eq!(geometry.normals.len(), geometry.positions.len());
    }

    #[test]
    fn base_stands_on_the_origin() {
        let geometry = trunk(&trunk_with(6, 40.0));
        let base = &geometry.positions[..6];
        let center = base.iter().copied().sum::<Vec3>() / 6.0;
        assert!(center.length() < 1e-5);
        assert!(base.iter().all(|p| p.y.abs() < 1e-5));
    }

    #[test]
    fn straight_trunk_reaches_its_height() {
        let geometry = trunk(&trunk_with(6, 0.0));
        let tip = *geometry.positions.last().unwrap();
        assert!((tip - Vec3::Y * Trunk::default().height).length() < 1e-5);
    }

    #[test]
    fn profiles_close_where_their_shapes_do() {
        let ends = |profile| (profile_width(profile, 0.0), profile_width(profile, 1.0));
        assert_eq!(ends(Profile::Cone), (1.0, 0.0));
        assert_eq!(ends(Profile::Ellipsoid), (0.0, 0.0));
        assert_eq!(ends(Profile::Hemisphere), (1.0, 0.0));
        assert_eq!(ends(Profile::Column), (1.0, 1.0));
        assert_eq!(ends(Profile::Flame), (0.0, 0.0));
        assert!((profile_width(Profile::Flame, 0.7) - 1.0).abs() < 1e-6);
        assert!((profile_width(Profile::Ellipsoid, 0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn outline_skips_spokes_and_rings_of_no_width() {
        let segments = |profile| {
            let envelope = Envelope {
                profile,
                ..Envelope::default()
            };
            envelope_outline(&envelope).len() / 2
        };
        // Nine rings of 24, and 8 meridians of 16 steps. The ellipsoid closes at both ends, so it
        // loses the end rings and both spokes; the cone only at the top.
        assert_eq!(segments(Profile::Ellipsoid), 7 * 24 + 8 * 16);
        assert_eq!(segments(Profile::Cone), 8 * 24 + 8 * 17);
        assert_eq!(segments(Profile::Column), 9 * 24 + 8 * 18);
    }

    #[test]
    fn winding_faces_the_normals() {
        for bend in [0.0, 45.0, 90.0] {
            let geometry = trunk(&trunk_with(5, bend));
            for triangle in geometry.indices.chunks(3) {
                let [a, b, c] = [0, 1, 2].map(|i| triangle[i] as usize);
                let p = &geometry.positions;
                let facing = (p[b] - p[a]).cross(p[c] - p[a]);
                let n = &geometry.normals;
                assert!(facing.dot(n[a] + n[b] + n[c]) > 0.0, "bend {bend}: {triangle:?}");
            }
        }
    }
}
