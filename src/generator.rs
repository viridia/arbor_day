//! The generator: a species in, a plant's geometry out. Plain functions on Bevy's math types, with
//! no ECS, so they can move to a library of their own (ledger X1).

use std::f32::consts::TAU;

use bevy::math::Vec3;

use crate::species::{Envelope, Profile, Species, Trunk};

/// The largest turn of a tube's axis between two rings, so rings are spaced by curvature: a
/// straight tube is one segment, and a bent one gets a ring per this much turn.
const MAX_RING_TURN_DEGREES: f32 = 15.0;

/// Halvings in finding where a ray leaves the envelope: the largest envelope's 90 m bound to under
/// a tenth of a millimetre.
const ENVELOPE_BISECTIONS: u32 = 20;

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

/// A limb as laid out on the trunk, before meshing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limb {
    pub origin: Vec3,
    /// Unit direction.
    pub direction: Vec3,
    pub length: f32,
    pub base_radius: f32,
}

/// The whole plant: the trunk and its limbs. Returns the geometry and the number of limbs.
pub fn plant(species: &Species) -> (Geometry, usize) {
    let mut geometry = trunk(&species.trunk);
    let limbs = limbs(species);
    let sides = species.limbs.sides.max(3) as u32;
    let taper = species.trunk.taper.clamp(0.0, 1.0);
    for limb in &limbs {
        let path = |t: f32| (limb.origin + limb.direction * limb.length * t, limb.direction);
        let radii = (limb.base_radius, limb.base_radius * (1.0 - taper));
        let binormal = limb.direction.any_orthonormal_vector();
        tube(&mut geometry, path, binormal, limb.length, 1, radii, sides);
    }
    (geometry, limbs.len())
}

/// The trunk: a tube of rings rising from the origin, bending toward +X along a circular arc,
/// closed at the top by a fan. The base is left open, since it stands in the ground.
pub fn trunk(trunk: &Trunk) -> Geometry {
    let height = trunk.height.max(0.01);
    let segments = trunk_segments(trunk);
    let mut geometry = Geometry::default();
    // The arc lies in the XY plane, so Z is square to the axis everywhere.
    tube(
        &mut geometry,
        |t| arc(height, trunk.bend, t),
        Vec3::Z,
        height,
        segments,
        trunk_radii(trunk),
        trunk.sides.max(3) as u32,
    );
    geometry
}

/// The limbs' layout: nodes spaced evenly along the span, each turned by the divergence from the
/// one below, its limbs spaced evenly around it. Azimuth 0 is toward +X. A limb whose start lies
/// outside the envelope, or that would be no longer than it is thick, is left out.
pub fn limbs(species: &Species) -> Vec<Limb> {
    let limbs = &species.limbs;
    let trunk = &species.trunk;
    let (base_radius, top_radius) = trunk_radii(trunk);
    let nodes = limbs.nodes.max(0) as u32;
    let per_node = limbs.per_node.max(1) as u32;
    let (elevation_sin, elevation_cos) = limbs.elevation.sin_cos();

    let mut laid_out = Vec::new();
    for node in 0..nodes {
        // Centered in equal shares of the span, so one node sits halfway along it.
        let t = limbs.start + (limbs.end - limbs.start) * (node as f32 + 0.5) / nodes as f32;
        let origin = trunk_axis(trunk, t);
        let base_radius = limbs.radius * (base_radius + (top_radius - base_radius) * t);
        for limb in 0..per_node {
            let azimuth = node as f32 * limbs.divergence + TAU * limb as f32 / per_node as f32;
            let (sin, cos) = azimuth.sin_cos();
            let direction = Vec3::new(elevation_sin * cos, elevation_cos, elevation_sin * sin);
            let length = limbs.reach * distance_to_envelope(&species.envelope, origin, direction);
            if length > base_radius {
                laid_out.push(Limb {
                    origin,
                    direction,
                    length,
                    base_radius,
                });
            }
        }
    }
    laid_out
}

/// Segments in the trunk's tube, by how far it bends.
fn trunk_segments(trunk: &Trunk) -> u32 {
    ((trunk.bend.abs() / MAX_RING_TURN_DEGREES.to_radians()).ceil() as u32).max(1)
}

/// The point at `t` (0 to 1) along the trunk's axis as meshed: straight between ring centers, so
/// limbs start inside the bark. The arc itself bows away from those chords, by as much as the
/// trunk's radius at the default bend.
fn trunk_axis(trunk: &Trunk, t: f32) -> Vec3 {
    let height = trunk.height.max(0.01);
    let segments = trunk_segments(trunk);
    let along = t.clamp(0.0, 1.0) * segments as f32;
    let segment = (along.floor() as u32).min(segments - 1);
    let ring = |ring: u32| arc(height, trunk.bend, ring as f32 / segments as f32).0;
    ring(segment).lerp(ring(segment + 1), along - segment as f32)
}

/// The trunk's radius at its base and its top.
fn trunk_radii(trunk: &Trunk) -> (f32, f32) {
    (
        trunk.radius,
        trunk.radius * (1.0 - trunk.taper.clamp(0.0, 1.0)),
    )
}

/// Adds a tube of `segments` along `path`, which gives the point and unit tangent at 0 to 1 and
/// must lie in a plane square to `binormal`, so the rings never roll. Radii go from `radii.0` at
/// the start to `radii.1` at the end, which a fan closes. The start is left open.
fn tube(
    geometry: &mut Geometry,
    path: impl Fn(f32) -> (Vec3, Vec3),
    binormal: Vec3,
    length: f32,
    segments: u32,
    radii: (f32, f32),
    sides: u32,
) {
    let first = geometry.positions.len() as u32;
    let (base_radius, top_radius) = radii;
    // A tapering tube's surface leans in toward the end, so its normals lean forward by the same
    // angle.
    let (lean_sin, lean_cos) = ((base_radius - top_radius) / length).atan().sin_cos();

    for ring in 0..=segments {
        let t = ring as f32 / segments as f32;
        let (center, tangent) = path(t);
        let across = tangent.cross(binormal);
        let radius = base_radius + (top_radius - base_radius) * t;
        for side in 0..sides {
            let (sin, cos) = (TAU * side as f32 / sides as f32).sin_cos();
            let outward = across * cos + binormal * sin;
            geometry.positions.push(center + outward * radius);
            geometry
                .normals
                .push((outward * lean_cos + tangent * lean_sin).normalize());
        }
    }
    for ring in 0..segments {
        for side in 0..sides {
            let a = first + ring * sides + side;
            let b = first + ring * sides + (side + 1) % sides;
            let (c, d) = (a + sides, b + sides);
            geometry.indices.extend([a, c, d, a, d, b]);
        }
    }

    let (end, tangent) = path(1.0);
    let tip = geometry.positions.len() as u32;
    geometry.positions.push(end);
    geometry.normals.push(tangent);
    let last = first + segments * sides;
    for side in 0..sides {
        geometry
            .indices
            .extend([last + side, tip, last + (side + 1) % sides]);
    }
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

/// The envelope's radius at height `y`: 0 below the crown base and above the top.
pub fn envelope_radius(envelope: &Envelope, y: f32) -> f32 {
    let (base, top) = crown_span(envelope);
    if y < base || y > top || top <= base {
        return 0.0;
    }
    envelope.radius * profile_width(envelope.profile, (y - base) / (top - base))
}

/// How far a ray from `origin` along the unit `direction` runs before it leaves the envelope, or 0
/// if `origin` is outside it. Every profile makes a convex solid, so the ray leaves it once, and
/// bisection between the origin and a point beyond its bounds finds where.
pub fn distance_to_envelope(envelope: &Envelope, origin: Vec3, direction: Vec3) -> f32 {
    let inside = |p: Vec3| p.x.hypot(p.z) < envelope_radius(envelope, p.y);
    if !inside(origin) {
        return 0.0;
    }
    // Further than any two points of the envelope are apart.
    let (mut near, mut far) = (0.0, 2.0 * (envelope.height.abs() + envelope.radius.abs()));
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
    fn envelope_radius_is_zero_outside_the_crown() {
        let envelope = Envelope {
            profile: Profile::Column,
            ..Envelope::default()
        };
        // Crown base at 1.8 m, top at 6 m.
        assert_eq!(envelope_radius(&envelope, 1.7), 0.0);
        assert_eq!(envelope_radius(&envelope, 3.0), envelope.radius);
        assert_eq!(envelope_radius(&envelope, 6.1), 0.0);
    }

    #[test]
    fn rays_leave_the_envelope_at_its_surface() {
        let envelope = Envelope {
            profile: Profile::Column,
            ..Envelope::default()
        };
        let origin = Vec3::Y * 3.0;
        assert!((distance_to_envelope(&envelope, origin, Vec3::X) - envelope.radius).abs() < 1e-3);
        // Straight up, out through the top.
        assert!((distance_to_envelope(&envelope, origin, Vec3::Y) - 3.0).abs() < 1e-3);
        // From below the crown base, nothing.
        assert_eq!(distance_to_envelope(&envelope, Vec3::Y, Vec3::X), 0.0);
    }

    #[test]
    fn limbs_reach_their_share_of_the_way_to_the_envelope() {
        let species = Species::default();
        let limbs = limbs(&species);
        assert_eq!(limbs.len(), 8);
        for limb in &limbs {
            let tip = limb.origin + limb.direction * limb.length;
            let rest = distance_to_envelope(&species.envelope, tip, limb.direction);
            let whole = limb.length + rest;
            assert!((limb.length - species.limbs.reach * whole).abs() < 1e-3);
            let elevation = limb.direction.angle_between(Vec3::Y);
            assert!((elevation - species.limbs.elevation).abs() < 1e-5);
        }
    }

    #[test]
    fn limbs_start_on_the_trunk_as_meshed() {
        // At the default bend the trunk is one straight segment, base to tip.
        let species = Species::default();
        let tip = *trunk(&species.trunk).positions.last().unwrap();
        for limb in limbs(&species) {
            assert!(limb.origin.cross(tip).length() < 1e-5, "{limb:?}");
        }
    }

    #[test]
    fn nodes_hold_their_limbs_evenly_around() {
        let mut species = Species::default();
        species.limbs.nodes = 2;
        species.limbs.per_node = 3;
        let limbs = limbs(&species);
        assert_eq!(limbs.len(), 6);
        // A node's limbs share an origin and stand a third of a turn apart.
        assert_eq!(limbs[0].origin, limbs[2].origin);
        let azimuth = |limb: &Limb| limb.direction.z.atan2(limb.direction.x);
        let apart = (azimuth(&limbs[1]) - azimuth(&limbs[0])).rem_euclid(TAU);
        assert!((apart - TAU / 3.0).abs() < 1e-5);
        // The next node turns by the divergence.
        let turned = (azimuth(&limbs[3]) - azimuth(&limbs[0])).rem_euclid(TAU);
        assert!((turned - species.limbs.divergence).abs() < 1e-5);
    }

    #[test]
    fn limbs_starting_below_the_crown_are_left_out() {
        let mut species = Species::default();
        // Nodes at about 0.45, 1.35, 2.25, 3.15 and 4.05 m; the crown base is at 1.8 m.
        species.limbs.start = 0.0;
        species.limbs.nodes = 5;
        assert_eq!(limbs(&species).len(), 3);
    }

    #[test]
    fn winding_faces_the_normals() {
        for bend in [0.0, 45.0, 90.0] {
            let species = Species {
                trunk: trunk_with(5, bend),
                ..Species::default()
            };
            let (geometry, _) = plant(&species);
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
