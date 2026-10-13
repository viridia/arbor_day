//! The generator: a species in, a plant's geometry out. Plain functions on Bevy's math types, with
//! no ECS, so they can move to a library of their own (ledger X1).

use std::f32::consts::TAU;

use bevy::math::{Quat, Vec2, Vec3};

use crate::{
    fuzzy::Fuzzy,
    random::Rng,
    species::{Envelope, LimbLevel, Profile, Species},
};

/// The largest turn of a tube's axis between two rings, so rings are spaced by curvature: a
/// straight tube is one segment, and a bent one gets a ring per this much turn.
const MAX_RING_TURN_DEGREES: f32 = 15.0;

/// The step in tracing a limb, in metres.
const TRACE_STEP: f32 = 0.02;

/// The distance along a limb between wander's random turns, in metres.
const WANDER_SPACING: f32 = 1.0;

/// Streams forked from a stem's own: its children take theirs by index from 0, so these take
/// indices no child reaches.
const WANDER_STREAM: u64 = u64::MAX;
/// The values drawn for a limb itself: its angle and reach.
const OWN_VALUES_STREAM: u64 = u64::MAX - 1;
/// The layout of a stem's children: their span and the turns between nodes.
const LAYOUT_STREAM: u64 = u64::MAX - 2;

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

/// A ring along a stem, the trunk or a limb, as meshed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ring {
    pub center: Vec3,
    /// Unit direction of the stem.
    pub tangent: Vec3,
    /// Unit vector square to the tangent, carried along the stem without twisting (a
    /// rotation-minimizing frame). It sets the ring's roll, and azimuth 0 for children starting
    /// there.
    pub side: Vec3,
    /// Distance along the stem from its base.
    pub at: f32,
}

/// A limb as grown on its parent, before meshing.
#[derive(Debug, Clone, PartialEq)]
pub struct Limb {
    /// The limb's level, 0 for the first.
    pub level: usize,
    /// At least two, base to tip.
    pub rings: Vec<Ring>,
    pub radii: (f32, f32),
    /// The limb's own random stream, before any draw; its children's streams fork from it.
    rng: Rng,
}

#[cfg(test)]
impl Limb {
    pub fn origin(&self) -> Vec3 {
        self.rings[0].center
    }

    /// Unit direction at the base.
    pub fn direction(&self) -> Vec3 {
        self.rings[0].tangent
    }

    pub fn length(&self) -> f32 {
        self.rings[self.rings.len() - 1].at
    }
}

/// A parent at one point along it: where its children start and how they are oriented.
#[derive(Debug, Clone, Copy)]
struct Frame {
    point: Vec3,
    /// Unit direction of the parent.
    tangent: Vec3,
    /// Unit vector square to the tangent, toward azimuth 0.
    side: Vec3,
}

/// The whole plant: the stems and their limbs, less limbs thinner at their base than the bark's
/// thinnest meshed radius, which are left to the foliage. Returns the geometry and the number of
/// limbs grown, meshed or not. Each stem is a tube closed at the top by a fan and left open at
/// the base, since it stands in the ground.
pub fn plant(species: &Species) -> (Geometry, usize) {
    let (stems, limbs) = grow(species);
    let mut geometry = Geometry::default();
    let trunk_sides = species.trunk.sides.max(3) as u32;
    for stem in &stems {
        tube(&mut geometry, &stem.rings, stem.radii, trunk_sides);
    }
    let levels = species.grown_levels();
    for limb in limbs
        .iter()
        .filter(|limb| limb.radii.0 >= species.bark.min_radius)
    {
        let sides = levels[limb.level].sides.max(3) as u32;
        tube(&mut geometry, &limb.rings, limb.radii, sides);
    }
    (geometry, limbs.len())
}

/// A stem rising from the ground, as grown, before meshing.
#[derive(Debug, Clone, PartialEq)]
pub struct Stem {
    /// At least two, base to tip.
    pub rings: Vec<Ring>,
    pub radii: (f32, f32),
    /// The stem's own random stream; its limbs' streams fork from it.
    rng: Rng,
}

/// The stems: `stems` of them from the origin, spaced evenly around, each leaving the ground at
/// the splay from up and turning by the lean as it rises, away from the middle. The first's
/// compass heading is drawn from the seed.
fn stems(species: &Species) -> Vec<Stem> {
    let trunk = &species.trunk;
    let count = trunk.stems.max(1) as u32;
    let length = (trunk.leader * species.envelope.height).max(0.01);
    let heading = species.seed.stream("trunk").unit() * TAU;
    let branching = species.seed.stream("branching");
    let top = trunk.radius * (1.0 - trunk.taper.clamp(0.0, 1.0));
    (0..count)
        .map(|stem| Stem {
            rings: stem_rings(
                length,
                trunk.splay,
                trunk.lean,
                heading + TAU * stem as f32 / count as f32,
            ),
            radii: (trunk.radius, top),
            rng: branching.fork(stem as u64),
        })
        .collect()
}

/// The limbs, as `grow` grows them.
#[cfg(test)]
pub fn limbs(species: &Species) -> Vec<Limb> {
    grow(species).1
}

/// The stems, and their limbs level by level, each level on the limbs of the one before and the
/// first on the stems, in level order. A stem's radius at its top is what its limbs leave it
/// (`grow_level`), so it is set once the level after has grown.
fn grow(species: &Species) -> (Vec<Stem>, Vec<Limb>) {
    let taper = species.trunk.taper.clamp(0.0, 1.0);
    let exponent = species.bark.pipe_exponent.max(1.0);
    let mut stems = stems(species);
    let mut grown: Vec<Limb> = Vec::new();
    for (level, params) in species.grown_levels().iter().enumerate() {
        let mut children = Vec::new();
        let mut grow_on = |parent| {
            grow_level(
                &species.envelope,
                params,
                level,
                taper,
                exponent,
                parent,
                &mut children,
            )
        };
        if level == 0 {
            for stem in &mut stems {
                stem.radii.1 = grow_on(Parent {
                    rings: &stem.rings,
                    radius: stem.radii.0,
                    rng: &stem.rng,
                });
            }
        }
        for limb in grown.iter_mut().filter(|limb| limb.level + 1 == level) {
            limb.radii.1 = grow_on(Parent {
                rings: &limb.rings,
                radius: limb.radii.0,
                rng: &limb.rng,
            });
        }
        grown.extend(children);
    }
    (stems, grown)
}

/// A stem limbs grow on: the trunk, or a limb of the level before.
#[derive(Clone, Copy)]
struct Parent<'a> {
    rings: &'a [Ring],
    /// Radius at the base.
    radius: f32,
    rng: &'a Rng,
}

impl Parent<'_> {
    /// The parent at `t` (0 to 1) along it as meshed: on the straight chord between ring
    /// centers, so children start inside the bark, with that chord's direction.
    fn frame(&self, t: f32) -> Frame {
        let t = t.clamp(0.0, 1.0);
        let at = t * self.rings[self.rings.len() - 1].at;
        let next = self
            .rings
            .partition_point(|ring| ring.at <= at)
            .clamp(1, self.rings.len() - 1);
        let (a, b) = (self.rings[next - 1], self.rings[next]);
        let along = if b.at > a.at {
            (at - a.at) / (b.at - a.at)
        } else {
            0.0
        };
        let tangent = (b.center - a.center).normalize_or(a.tangent);
        let side = a
            .side
            .lerp(b.side, along)
            .reject_from_normalized(tangent)
            .normalize_or(a.side);
        Frame {
            point: a.center.lerp(b.center, along),
            tangent,
            side,
        }
    }
}

/// Adds one parent's limbs of one level: nodes spaced evenly along the span, as many as the
/// level's density gives for the span's length, each turned by the divergence from the one below,
/// its limbs spaced evenly around it, each leaving at the level's angle from the parent and then
/// grown (`grow_limb`). A limb whose start lies outside the envelope, or that would be no longer
/// than it is thick, is left out.
///
/// Radii follow the pipe model: a stem's radius raised to `exponent` is shared among the limbs
/// leaving it and what carries on past them. Each limb's radius is the level's share of the
/// parent's radius where it leaves, and the parent is that much thinner past it. Returns the
/// parent's radius at its top: what its limbs leave it, or its base radius tapered if that is
/// thinner, so the taper is the least a stem narrows.
fn grow_level(
    envelope: &Envelope,
    params: &LimbLevel,
    level: usize,
    taper: f32,
    exponent: f32,
    parent: Parent,
    grown: &mut Vec<Limb>,
) -> f32 {
    // The parent's radius raised to the exponent, which each limb leaving it takes its share of.
    let mut section = parent.radius.powf(exponent);
    // The span, the node count and each node's turn and limb count are the parent's, drawn from a
    // stream of its own beside its limbs'.
    let mut layout = parent.rng.fork(LAYOUT_STREAM);
    let middle = draw(&params.span_middle, "span_middle", &mut layout);
    let half = draw(&params.span_length, "span_length", &mut layout) * 0.5;
    let (span_start, span_end) = ((middle - half).max(0.0), (middle + half).min(1.0));
    let span_length = (span_end - span_start) * parent.rings[parent.rings.len() - 1].at;
    let nodes = layout.round(params.density * span_length / envelope.radius.max(0.01));
    let mut node_azimuth = 0.0;
    let mut index = 0;
    for node in 0..nodes {
        // A turn is any angle, so the divergence is not clamped; adding it is the same modulo a
        // turn.
        let divergence = params.divergence.sample(&mut layout);
        if node > 0 {
            node_azimuth += divergence;
        }
        let per_node = layout.round(params.per_node).max(1);
        // Centered in equal shares of the span, so one node sits halfway along it.
        let t = span_start + (span_end - span_start) * (node as f32 + 0.5) / nodes as f32;
        let frame = parent.frame(t);
        let across = frame.side.cross(frame.tangent);
        let base_radius = params.radius * section.powf(exponent.recip());
        for limb in 0..per_node {
            let rng = parent.rng.fork(index);
            index += 1;
            let mut own = rng.fork(OWN_VALUES_STREAM);
            let angle = draw(&params.angle, "angle", &mut own);
            let reach = draw(&params.reach, "reach", &mut own);
            let azimuth = node_azimuth + TAU * limb as f32 / per_node as f32;
            let (sin, cos) = azimuth.sin_cos();
            let (angle_sin, angle_cos) = angle.sin_cos();
            let direction =
                frame.tangent * angle_cos + (frame.side * cos + across * sin) * angle_sin;
            let reach = reach * distance_to_envelope(envelope, frame.point, direction);
            let start = Ring {
                center: frame.point,
                tangent: direction,
                side: upper_side(direction, frame.side),
                at: 0.0,
            };
            let rings = grow_limb(envelope, params, start, reach, rng.fork(WANDER_STREAM));
            if rings.len() >= 2 && rings[rings.len() - 1].at > base_radius {
                section = (section - base_radius.powf(exponent)).max(0.0);
                grown.push(Limb {
                    level,
                    rings,
                    radii: (base_radius, base_radius * (1.0 - taper)),
                    rng,
                });
            }
        }
    }
    // The limbs' narrowing, or at least the taper's.
    section
        .powf(exponent.recip())
        .min(parent.radius * (1.0 - taper))
}

/// A limb's rings, from `start` for up to `length` along it. The limb is traced in short steps,
/// its direction turning toward the level's bend and wandering at random, and it stops early where
/// it would leave the envelope. A ring is placed wherever it has turned `MAX_RING_TURN_DEGREES`
/// since the last, and at the tip.
fn grow_limb(
    envelope: &Envelope,
    params: &LimbLevel,
    start: Ring,
    length: f32,
    mut rng: Rng,
) -> Vec<Ring> {
    // Wander's turns are drawn at points `WANDER_SPACING` apart and blended smoothly between, so
    // the limb's shape does not depend on the step.
    let knots: Vec<Vec2> = (0..(length / WANDER_SPACING).ceil() as usize + 2)
        .map(|_| Vec2::new(rng.unit(), rng.unit()) * 2.0 - 1.0)
        .collect();
    let wander_at = |at: f32| {
        let along = at / WANDER_SPACING;
        let knot = (along.floor() as usize).min(knots.len() - 2);
        let blend = along - knot as f32;
        let blend = blend * blend * (3.0 - 2.0 * blend);
        knots[knot]
            .lerp(knots[knot + 1], blend)
            .clamp_length_max(1.0)
    };
    let (bend_sin, bend_cos) = params.bend_toward.sin_cos();
    // The share of the angle left to the bend that one step closes. Shares compound, so a run of
    // steps closes the same share whatever the step.
    let kept_per_crown = 1.0 - params.bend.clamp(0.0, 0.99);
    let crown = envelope.radius.max(0.01);
    let max_turn = MAX_RING_TURN_DEGREES.to_radians();

    let mut rings = vec![start];
    let mut ring = start;
    let mut last_ring_tangent = start.tangent;
    while ring.at < length {
        let step = TRACE_STEP.min(length - ring.at);
        let center = ring.center + ring.tangent * step;
        if !inside_envelope(envelope, center) {
            break;
        }
        let mut tangent = ring.tangent;
        // Toward the bend, keeping the compass heading; a limb pointing straight up or down takes
        // its side's.
        let heading = Vec3::new(tangent.x, 0.0, tangent.z)
            .try_normalize()
            .or_else(|| Vec3::new(ring.side.x, 0.0, ring.side.z).try_normalize());
        if let Some(heading) = heading {
            let toward = Vec3::Y * bend_cos + heading * bend_sin;
            let closed = 1.0 - kept_per_crown.powf(step / crown);
            tangent = turn_toward(tangent, toward, tangent.angle_between(toward) * closed);
        }
        let wander = wander_at(ring.at);
        let push = ring.side * wander.x + ring.side.cross(ring.tangent) * wander.y;
        if let Some(axis) = tangent.cross(push).try_normalize() {
            tangent = Quat::from_axis_angle(axis, params.wander * step * wander.length()) * tangent;
        }
        let tangent = tangent.normalize();
        let side = (Quat::from_rotation_arc(ring.tangent, tangent) * ring.side)
            .reject_from_normalized(tangent)
            .normalize_or(ring.side);
        ring = Ring {
            center,
            tangent,
            side,
            at: ring.at + step,
        };
        if tangent.angle_between(last_ring_tangent) >= max_turn {
            rings.push(ring);
            last_ring_tangent = tangent;
        }
    }
    if ring.at > rings[rings.len() - 1].at {
        rings.push(ring);
    }
    rings
}

/// A value drawn from `fuzzy`, clamped to the limits of `LimbLevel`'s field `field`.
fn draw(fuzzy: &Fuzzy, field: &str, rng: &mut Rng) -> f32 {
    let limits = LimbLevel::limits(field);
    fuzzy.sample(rng).clamp(*limits.start(), *limits.end())
}

/// `from` turned toward `toward` by at most `angle`, both unit vectors.
fn turn_toward(from: Vec3, toward: Vec3, angle: f32) -> Vec3 {
    let between = from.angle_between(toward);
    if between < 1e-6 {
        return from;
    }
    let axis = from
        .cross(toward)
        .try_normalize()
        .unwrap_or_else(|| from.any_orthonormal_vector());
    Quat::from_axis_angle(axis, between.min(angle)) * from
}

/// The unit vector square to `direction` nearest world up: a limb's upper side. A limb pointing
/// straight up or down has none, and takes its parent's side, made square to it.
fn upper_side(direction: Vec3, parent_side: Vec3) -> Vec3 {
    let up = Vec3::Y.reject_from_normalized(direction);
    if up.length_squared() > 1e-6 {
        up.normalize()
    } else {
        parent_side
            .reject_from_normalized(direction)
            .try_normalize()
            .unwrap_or_else(|| direction.any_orthonormal_vector())
    }
}

/// A stem's rings, evenly spaced along its arc (`arc`), turned about the vertical to `heading`.
/// The arc lies in one vertical plane, so a side square to it in that plane never twists: azimuth
/// 0 is toward the heading.
fn stem_rings(length: f32, splay: f32, lean: f32, heading: f32) -> Vec<Ring> {
    let segments = ((lean.abs() / MAX_RING_TURN_DEGREES.to_radians()).ceil() as u32).max(1);
    let turn = Quat::from_rotation_y(heading);
    (0..=segments)
        .map(|ring| {
            let t = ring as f32 / segments as f32;
            let (center, tangent) = arc(length, splay, lean, t);
            Ring {
                center: turn * center,
                tangent: turn * tangent,
                side: turn * tangent.cross(Vec3::Z),
                at: length * t,
            }
        })
        .collect()
}

/// Adds a tube through `rings`, each turned to its ring's side, so the tube rolls only as the
/// sides do. Radii go from `radii.0` at the start to `radii.1` at the end, which a fan closes. The
/// start is left open.
fn tube(geometry: &mut Geometry, rings: &[Ring], radii: (f32, f32), sides: u32) {
    let first = geometry.positions.len() as u32;
    let (base_radius, top_radius) = radii;
    let length = rings[rings.len() - 1].at;
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
    let inside = |p: Vec3| inside_envelope(envelope, p);
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

/// Whether `point` lies inside the envelope.
fn inside_envelope(envelope: &Envelope, point: Vec3) -> bool {
    point.x.hypot(point.z) < envelope_radius(envelope, point.y)
}

/// The heights of the crown base and the top.
fn crown_span(envelope: &Envelope) -> (f32, f32) {
    let top = envelope.height.max(0.01);
    (top * envelope.crown_base.clamp(0.0, 1.0), top)
}

/// The point and unit tangent at `t` (0 to 1) along an arc of length `length` that starts at the
/// origin `start` radians from up toward +X and turns by `bend` radians further toward +X.
fn arc(length: f32, start: f32, bend: f32, t: f32) -> (Vec3, Vec3) {
    let angle = start + bend * t;
    let (sin, cos) = angle.sin_cos();
    let tangent = Vec3::new(sin, cos, 0.0);
    if bend.abs() < 1e-4 {
        return (tangent * length * t, tangent);
    }
    let radius = length / bend;
    let (start_sin, start_cos) = start.sin_cos();
    (
        Vec3::new(radius * (start_cos - cos), radius * (sin - start_sin), 0.0),
        tangent,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fuzzy::Distribution, random::Seed, species::Trunk};
    use bevy::math::Vec3Swizzles;

    /// A lone 5 m stem, standing straight up and leaning toward +X.
    fn stem_mesh(sides: u32, lean_degrees: f32) -> Geometry {
        let mut geometry = Geometry::default();
        let rings = stem_rings(5.0, 0.0, lean_degrees.to_radians(), 0.0);
        tube(&mut geometry, &rings, (0.2, 0.1), sides);
        geometry
    }

    fn trunk_with(sides: i32, lean_degrees: f32) -> Trunk {
        Trunk {
            sides,
            lean: lean_degrees.to_radians(),
            ..Trunk::default()
        }
    }

    #[test]
    fn rings_follow_curvature() {
        // Straight: one segment of 6 quads and a 6-triangle cap.
        assert_eq!(stem_mesh(6, 0.0).triangles(), 6 * 2 + 6);
        // 90° at 15° a ring: six segments.
        assert_eq!(stem_mesh(4, 90.0).triangles(), 4 * 2 * 6 + 4);
    }

    #[test]
    fn vertices_are_shared() {
        let geometry = stem_mesh(5, 30.0);
        // Two segments: three rings of five, and the tip.
        assert_eq!(geometry.positions.len(), 3 * 5 + 1);
        assert_eq!(geometry.normals.len(), geometry.positions.len());
    }

    #[test]
    fn base_stands_on_the_origin() {
        let geometry = stem_mesh(6, 40.0);
        let base = &geometry.positions[..6];
        let center = base.iter().copied().sum::<Vec3>() / 6.0;
        assert!(center.length() < 1e-5);
        assert!(base.iter().all(|p| p.y.abs() < 1e-5));
    }

    #[test]
    fn a_straight_stem_reaches_the_leader() {
        let mut species = Species::default();
        species.trunk.lean = 0.0;
        species.trunk.leader = 0.5;
        let tip = stems(&species)[0].rings.last().unwrap().center;
        let height = 0.5 * species.envelope.height;
        assert!((tip - Vec3::Y * height).length() < 1e-5, "{tip}");
    }

    #[test]
    fn stems_splay_evenly_around() {
        let mut species = Species::default();
        species.trunk.stems = 3;
        species.trunk.splay = 30.0_f32.to_radians();
        species.trunk.lean = 0.0;
        let stems = stems(&species);
        assert_eq!(stems.len(), 3);
        let directions: Vec<Vec3> = stems.iter().map(|stem| stem.rings[0].tangent).collect();
        for direction in &directions {
            assert!((direction.angle_between(Vec3::Y).to_degrees() - 30.0).abs() < 1e-3);
        }
        let heading = |v: Vec3| v.z.atan2(v.x);
        for pair in directions.windows(2) {
            let apart = (heading(pair[0]) - heading(pair[1])).rem_euclid(TAU);
            assert!((apart - TAU / 3.0).abs() < 1e-4, "{}", apart.to_degrees());
        }
    }

    #[test]
    fn the_seed_turns_the_lean() {
        let tip = |seed| {
            let species = Species {
                seed: Seed(seed),
                ..Species::default()
            };
            stems(&species)[0].rings.last().unwrap().center
        };
        // The same height and lean, in another compass direction.
        let (one, two) = (tip(1), tip(2));
        assert!((one.y - two.y).abs() < 1e-5);
        assert!((one.xz().length() - two.xz().length()).abs() < 1e-5);
        assert!(one.xz().angle_to(two.xz()).abs() > 0.1, "{one} {two}");
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
            height: 6.0,
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
            height: 6.0,
            ..Envelope::default()
        };
        let origin = Vec3::Y * 3.0;
        assert!((distance_to_envelope(&envelope, origin, Vec3::X) - envelope.radius).abs() < 1e-3);
        // Straight up, out through the top.
        assert!((distance_to_envelope(&envelope, origin, Vec3::Y) - 3.0).abs() < 1e-3);
        // From below the crown base, nothing.
        assert_eq!(distance_to_envelope(&envelope, Vec3::Y, Vec3::X), 0.0);
    }

    /// The default species with every level's limbs straight and nothing varied.
    fn regular() -> Species {
        let mut species = Species::default();
        for level in &mut species.limbs {
            level.bend = 0.0;
            level.wander = 0.0;
            for fuzzy in [
                &mut level.divergence,
                &mut level.span_middle,
                &mut level.span_length,
                &mut level.angle,
                &mut level.reach,
            ] {
                fuzzy.spread = 0.0;
            }
        }
        species
    }

    #[test]
    fn limbs_reach_their_share_of_the_way_to_the_envelope() {
        let species = regular();
        let limbs = limbs(&species);
        assert_eq!(limbs.len(), 8);
        for limb in &limbs {
            assert_eq!(limb.rings.len(), 2, "a straight limb is one segment");
            let tip = limb.origin() + limb.direction() * limb.length();
            let rest = distance_to_envelope(&species.envelope, tip, limb.direction());
            let whole = limb.length() + rest;
            assert!((limb.length() - species.limbs[0].reach.center * whole).abs() < 1e-3);
        }
    }

    #[test]
    fn limbs_leave_the_trunk_at_their_angle() {
        // At the default lean the trunk is one straight segment, base to tip.
        let species = regular();
        let tip = stems(&species)[0].rings.last().unwrap().center;
        for limb in limbs(&species) {
            let angle = limb.direction().angle_between(tip);
            assert!(
                (angle - species.limbs[0].angle.center).abs() < 1e-5,
                "{limb:?}"
            );
        }
    }

    #[test]
    fn varied_values_stay_within_their_spread_and_limits() {
        let mut species = regular();
        species.limbs[0].angle = Fuzzy::new(
            60.0_f32.to_radians(),
            10.0_f32.to_radians(),
            Distribution::Uniform,
        );
        // Reaching past the envelope is clamped to reaching it.
        species.limbs[0].reach = Fuzzy::new(0.95, 0.2, Distribution::Uniform);
        let tip = stems(&species)[0].rings.last().unwrap().center;
        let limbs = limbs(&species);
        let angles: Vec<f32> = limbs
            .iter()
            .map(|limb| limb.direction().angle_between(tip).to_degrees())
            .collect();
        assert!(
            angles.iter().all(|angle| (49.99..=70.01).contains(angle)),
            "{angles:?}"
        );
        assert!(
            angles.iter().any(|angle| (angle - angles[0]).abs() > 0.1),
            "{angles:?}"
        );
        for limb in &limbs {
            let whole = distance_to_envelope(&species.envelope, limb.origin(), limb.direction());
            assert!(limb.length() <= whole + 1e-3);
        }
    }

    #[test]
    fn deeper_levels_grow_on_the_level_before() {
        let mut species = regular();
        let first = limbs(&species);
        species.limb_levels = 2;
        let both = limbs(&species);
        // The first level is unchanged but for its tips, which its limbs narrow, and the second
        // follows it.
        let shape = |limbs: &[Limb]| -> Vec<(Vec<Ring>, f32)> {
            limbs
                .iter()
                .map(|limb| (limb.rings.clone(), limb.radii.0))
                .collect()
        };
        assert_eq!(shape(&both[..first.len()]), shape(&first));
        let second = &both[first.len()..];
        assert!(!second.is_empty());
        for limb in second {
            assert_eq!(limb.level, 1);
            // Starts on a first-level limb, at the level's angle from it.
            let parent = first
                .iter()
                .find(|parent| {
                    let from = limb.origin() - parent.origin();
                    from.cross(parent.direction()).length() < 1e-3
                        && from.dot(parent.direction()) >= 0.0
                        && from.length() <= parent.length()
                })
                .expect("a parent under the limb");
            let angle = limb.direction().angle_between(parent.direction());
            assert!((angle - species.limbs[1].angle.center).abs() < 1e-3);
        }
    }

    #[test]
    fn limbs_bend_toward_their_bend() {
        let mut species = regular();
        species.limbs[0].bend = 0.5;
        let target = species.limbs[0].bend_toward;
        let crown = species.envelope.radius;
        for limb in limbs(&species) {
            let tip = limb.rings[limb.rings.len() - 1].tangent;
            // Half the angle left is closed over each crown radius of length.
            let left_at_base = target - limb.direction().angle_between(Vec3::Y);
            let left_at_tip = target - tip.angle_between(Vec3::Y);
            let expected = left_at_base * 0.5_f32.powf(limb.length() / crown);
            assert!(
                (left_at_tip - expected).abs() < 1e-3,
                "{left_at_tip} {expected}"
            );
            // The compass heading is kept.
            let heading = |v: Vec3| v.z.atan2(v.x);
            assert!((heading(tip) - heading(limb.direction())).abs() < 1e-3);
        }
    }

    #[test]
    fn every_bend_target_moves_the_tip() {
        let tips = |degrees: f32| {
            let mut species = regular();
            species.limbs[0].bend = 0.5;
            species.limbs[0].bend_toward = degrees.to_radians();
            limbs(&species)
                .iter()
                .map(|limb| limb.rings[limb.rings.len() - 1].tangent)
                .collect::<Vec<_>>()
        };
        let mut previous = tips(90.0);
        for degrees in [110.0, 130.0, 150.0, 170.0] {
            let next = tips(degrees);
            // Each limb's tip droops further, or the limb stops sooner at the envelope.
            assert_ne!(previous, next, "{degrees}°");
            previous = next;
        }
    }

    #[test]
    fn default_plants_stay_within_budget() {
        // Orders of magnitude, not counts: the low hundreds for one level, and for three, under
        // seedling's trees (about 2,500), since most of the third level is left to the foliage.
        let triangles = |limb_levels| {
            let species = Species {
                limb_levels,
                ..Species::default()
            };
            plant(&species).0.triangles()
        };
        assert!((100..1_000).contains(&triangles(1)), "{}", triangles(1));
        assert!(triangles(3) < 3_000, "{}", triangles(3));
    }

    #[test]
    fn rings_are_spaced_by_turn() {
        let species = Species {
            limb_levels: 3,
            ..Species::default()
        };
        let max_turn = MAX_RING_TURN_DEGREES.to_radians();
        for limb in limbs(&species) {
            for pair in limb.rings.windows(2) {
                // A ring is placed once the turn reaches the limit, so by at most a step's turn more.
                let turn = pair[0].tangent.angle_between(pair[1].tangent);
                assert!(turn < max_turn + 0.05, "{}", turn.to_degrees());
                assert!(pair[1].at > pair[0].at);
            }
        }
    }

    #[test]
    fn limbs_stay_inside_the_envelope() {
        let species = Species {
            limb_levels: 3,
            ..Species::default()
        };
        for limb in limbs(&species) {
            for ring in &limb.rings[1..] {
                assert!(inside_envelope(&species.envelope, ring.center), "{ring:?}");
            }
        }
    }

    #[test]
    fn the_seed_sets_the_wander() {
        let positions = |seed| {
            let species = Species {
                seed: Seed(seed),
                limb_levels: 2,
                ..Species::default()
            };
            plant(&species).0.positions
        };
        assert_eq!(positions(1), positions(1));
        assert_ne!(positions(1), positions(2));
    }

    #[test]
    fn hidden_levels_do_not_grow() {
        let mut species = Species::default();
        species.limbs[2].density = 40.0;
        let one = limbs(&species).len();
        species.limb_levels = 2;
        let two = limbs(&species);
        assert!(two.len() > one);
        assert!(two.iter().all(|limb| limb.level < 2));
        species.limb_levels = 3;
        assert!(limbs(&species).iter().any(|limb| limb.level == 2));
    }

    #[test]
    fn azimuth_zero_is_the_upper_side() {
        // A level limb's side is world up; one pointing straight up keeps its parent's.
        assert!((upper_side(Vec3::X, Vec3::Z) - Vec3::Y).length() < 1e-6);
        assert!((upper_side(Vec3::Y, Vec3::Z) - Vec3::Z).length() < 1e-6);
    }

    #[test]
    fn limbs_start_on_the_trunk_as_meshed() {
        // At the default lean the trunk is one straight segment, base to tip.
        let species = Species::default();
        let tip = stems(&species)[0].rings.last().unwrap().center;
        for limb in limbs(&species) {
            assert!(limb.origin().cross(tip).length() < 1e-5, "{limb:?}");
        }
    }

    #[test]
    fn nodes_hold_their_limbs_evenly_around() {
        let mut species = regular();
        // Straight, so azimuth about the trunk is azimuth about world up.
        species.trunk.lean = 0.0;
        // Two nodes on the span's 2.5 m, a crown radius.
        species.limbs[0].density = 2.0;
        species.limbs[0].per_node = 3.0;
        let limbs = limbs(&species);
        assert_eq!(limbs.len(), 6);
        // A node's limbs share an origin and stand a third of a turn apart.
        assert_eq!(limbs[0].origin(), limbs[2].origin());
        let azimuth = |limb: &Limb| limb.direction().z.atan2(limb.direction().x);
        let apart = (azimuth(&limbs[1]) - azimuth(&limbs[0])).rem_euclid(TAU);
        assert!((apart - TAU / 3.0).abs() < 1e-5);
        // The next node turns by the divergence.
        let turned = (azimuth(&limbs[3]) - azimuth(&limbs[0])).rem_euclid(TAU);
        assert!((turned - species.limbs[0].divergence.center).abs() < 1e-5);
    }

    #[test]
    fn limbs_starting_below_the_crown_are_left_out() {
        let mut species = regular();
        // Five nodes along the whole trunk, two crown radii, at 0.5, 1.5, 2.5, 3.5 and 4.5 m; the
        // crown base is at 1.875 m.
        species.limbs[0].span_middle = Fuzzy::exactly(0.5);
        species.limbs[0].span_length = Fuzzy::exactly(1.0);
        species.limbs[0].density = 2.5;
        assert_eq!(limbs(&species).len(), 3);
    }

    #[test]
    fn limbs_share_their_parent_section() {
        let mut species = regular();
        species.limbs[0].density = 2.0;
        species.limbs[0].radius = 0.6;
        species.bark.pipe_exponent = 2.0;
        let trunk = species.trunk.radius;
        let (stems, limbs) = grow(&species);
        let top = stems[0].radii.1;
        assert_eq!(limbs.len(), 2);
        // The first takes its share of the whole trunk; the second, of what is left past it.
        let first = 0.6 * trunk;
        let past_first = (trunk.powi(2) - first.powi(2)).sqrt();
        let second = 0.6 * past_first;
        assert!((limbs[0].radii.0 - first).abs() < 1e-6);
        assert!((limbs[1].radii.0 - second).abs() < 1e-6);
        // The trunk's top is what is left past both, unless the taper narrows it further.
        let left = (past_first.powi(2) - second.powi(2)).sqrt();
        let tapered = trunk * (1.0 - species.trunk.taper);
        assert!((top - left.min(tapered)).abs() < 1e-6, "{top} {left} {tapered}");
        // A higher exponent narrows the trunk less.
        species.bark.pipe_exponent = 3.0;
        species.trunk.taper = 0.0;
        let higher = grow(&species).0[0].radii.1;
        assert!(higher > left, "{higher} {left}");
    }

    #[test]
    fn thin_limbs_are_grown_but_not_meshed() {
        let mut species = Species {
            limb_levels: 3,
            ..Species::default()
        };
        let (all, limbs) = plant(&species);
        species.bark.min_radius = 1.0;
        let (bare, still) = plant(&species);
        assert_eq!(still, limbs);
        // The trunk alone: at the default lean, one segment of 6 quads and a 6-triangle cap.
        assert_eq!(bare.triangles(), 6 * 2 + 6);
        assert!(all.triangles() > bare.triangles());
    }

    #[test]
    fn the_span_is_cut_short_at_the_parent_end() {
        let mut species = regular();
        species.trunk.lean = 0.0;
        // From halfway up the 5 m trunk to its top, one crown radius, not the 5 m asked for.
        species.limbs[0].span_middle = Fuzzy::exactly(1.0);
        species.limbs[0].span_length = Fuzzy::exactly(1.0);
        species.limbs[0].density = 2.0;
        let limbs = limbs(&species);
        assert_eq!(limbs.len(), 2);
        assert!(limbs.iter().all(|limb| (2.5..5.0).contains(&limb.origin().y)));
    }

    #[test]
    fn nodes_are_counted_in_crown_radii() {
        let mut species = regular();
        species.trunk.lean = 0.0;
        // The span is 2.5 m, one crown radius, then two.
        species.limbs[0].density = 2.0;
        assert_eq!(limbs(&species).len(), 2);
        species.envelope.radius = 1.25;
        assert_eq!(limbs(&species).len(), 4);
    }

    #[test]
    fn fractional_counts_average_out_over_seeds() {
        // Limbs on the trunk for each of many seeds, with `density` and `per_node` as given.
        let counts = |density: f32, per_node: f32| -> Vec<usize> {
            (0..400)
                .map(|seed| {
                    let mut species = regular();
                    species.seed = Seed(seed);
                    species.limbs[0].density = density;
                    species.limbs[0].per_node = per_node;
                    limbs(&species).len()
                })
                .collect()
        };
        let mean = |counts: &[usize]| counts.iter().sum::<usize>() as f32 / counts.len() as f32;
        // 2.5 nodes on the span's crown radius: two or three.
        let nodes = counts(2.5, 1.0);
        assert!(nodes.iter().all(|count| (2..=3).contains(count)), "{nodes:?}");
        assert!((mean(&nodes) - 2.5).abs() < 0.1, "{}", mean(&nodes));
        // Eight nodes of one or two limbs.
        let limbs = counts(8.0, 1.5);
        assert!(limbs.iter().all(|count| (8..=16).contains(count)), "{limbs:?}");
        assert!((mean(&limbs) - 12.0).abs() < 0.3, "{}", mean(&limbs));
    }

    #[test]
    fn winding_faces_the_normals() {
        for lean in [0.0, 45.0, 90.0] {
            let species = Species {
                trunk: trunk_with(5, lean),
                limb_levels: 3,
                ..Species::default()
            };
            let (geometry, _) = plant(&species);
            for triangle in geometry.indices.chunks(3) {
                let [a, b, c] = [0, 1, 2].map(|i| triangle[i] as usize);
                let p = &geometry.positions;
                let facing = (p[b] - p[a]).cross(p[c] - p[a]);
                let n = &geometry.normals;
                assert!(
                    facing.dot(n[a] + n[b] + n[c]) > 0.0,
                    "lean {lean}: {triangle:?}"
                );
            }
        }
    }
}
