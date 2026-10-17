//! Where axes start and which way they leave: stems on the ground, and a level's laterals at
//! nodes along their parent.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::{
    math::{Vec2, Vec3},
    reflect::Typed,
};

use super::{
    envelope::{Envelope, distance_to_envelope},
    fork::{at_base, copy_directions, draw_fork},
    skeleton::{
        Axis, AxisKind, Frame, LAYOUT_STREAM, Level, OWN_VALUES_STREAM, Ring, Skeleton,
        WANDER_STREAM, taper_rings, upper_side,
    },
    trace::{end_at, kink_spacing, stem_rings},
    wander::displace,
};
use crate::{
    fuzzy::Fuzzy,
    random::Rng,
    species::{Facing, LimbLevel, Species, limits},
};

/// The most a node moves from its even place, as a share of the gap between places, so nodes are
/// not spaced like rungs. Under half, so neighbours never swap.
const NODE_JITTER: f32 = 0.3;

/// Turns tried for each node, evenly around the parent.
const AZIMUTH_CANDIDATES: u32 = 72;
/// How much of a placed limb's crowding is left one mean gap further along the parent.
const CROWDING_FADE: f32 = 0.7;
/// The power the nearness of two azimuths is raised to, so crowding falls off sharply around a
/// limb. With `CROWDING_FADE`, it gives the golden angle for one limb a node; gentler, the turn
/// falls to 180°, and sharper, it turns erratic.
const CROWDING_SHARPNESS: i32 = 4;
/// How much a limb pointing straight out from the plant's axis gains, a crown radius off it,
/// against crowding of 1 for a limb at the same azimuth one gap away.
const OUTWARD_WEIGHT: f32 = 1.0;

/// The stems: the trunk from the origin, upright at the ground and turning by the lean as it
/// rises toward a compass heading drawn from the seed (`stem_course`). With Fork on, it may fork
/// (D31): it ends at the fork, and copies carry it on in their directions (`copy_directions`),
/// each from the trunk's radius there and turning by what is left of the lean, away from the
/// middle, the rest of the trunk's height long, so every tip is the height from the ground along
/// the stems. Forked at its base, its copies take its place, several stems rising from the ground.
pub(super) fn stems(species: &Species) -> Vec<Axis> {
    let trunk = &species.trunk;
    let length = trunk.height.max(0.01);
    let heading = species.seed.stream("trunk").unit() * TAU;
    let rng = species.seed.stream("branching").fork(0);
    let rings = stem_course(
        species,
        Vec3::ZERO,
        0.0,
        heading,
        trunk.lean,
        length,
        trunk.radius,
        &rng,
    );
    let mut stem = Axis::stem(rings, rng);
    let Some(fork) = trunk.fork.get() else {
        return vec![stem];
    };
    let Some(drawn) = draw_fork(fork, &stem.rng) else {
        return vec![stem];
    };
    let frame = stem.frame(drawn.at);
    let fork_at = drawn.at * stem.length();
    let at_base = at_base(fork_at, frame.radius);
    let lean = trunk.lean * (1.0 - drawn.at);
    let copies: Vec<Axis> = (copy_directions(&frame, fork, drawn.half_angle).into_iter())
        .enumerate()
        .map(|(copy, direction)| {
            let rng = drawn.copies.fork(copy as u64);
            // Its heading and angle from up, never below level; straight up keeps the trunk's.
            let across = Vec2::new(direction.x, -direction.z);
            let heading = if across.length() > 1e-5 {
                across.to_angle()
            } else {
                heading
            };
            let from_up = direction.angle_between(Vec3::Y).min(FRAC_PI_2);
            let rest = length - fork_at;
            let rings = stem_course(
                species,
                frame.point,
                from_up,
                heading,
                lean,
                rest,
                frame.radius,
                &rng,
            );
            if at_base {
                Axis::stem(rings, rng)
            } else {
                let parent = (0, fork_at);
                let mut copy = Axis::placed(0, AxisKind::ForkCopy, parent, rings[0], rest, rng);
                copy.rings = rings;
                copy
            }
        })
        .collect();
    if at_base {
        return copies;
    }
    end_at(&mut stem.rings, frame.ring(fork_at));
    stem.scions.extend(1..=copies.len());
    let mut stems = vec![stem];
    stems.extend(copies);
    stems
}

/// A stem's rings from `base`, `length` long: leaving `from_up` radians from up toward `heading`
/// and turning toward it by `lean`, but never past level, where it would sink again, then
/// zigzagging about that course by the trunk's wander, if on (`displace`), its tip where the
/// course ends, and narrowing from `radius` by the plant's taper.
fn stem_course(
    species: &Species,
    base: Vec3,
    from_up: f32,
    heading: f32,
    lean: f32,
    length: f32,
    radius: f32,
    rng: &Rng,
) -> Vec<Ring> {
    let lean = lean.min((FRAC_PI_2 - from_up).max(0.0));
    let mut rings = stem_rings(length, from_up, lean, heading);
    for ring in &mut rings {
        ring.center += base;
    }
    if let Some(wander) = species.trunk.wander.get() {
        let spacing = kink_spacing(&Envelope::of(species));
        displace(
            &mut rings,
            rng.fork(WANDER_STREAM),
            wander.angle,
            spacing,
            length,
            None,
        );
    }
    taper_rings(&mut rings, radius, species.taper.clamp(0.0, 1.0));
    rings
}

/// Places the level's laterals on each axis of the level before (`place_on`), for
/// `trace_scions` to grow.
pub(super) fn place_scions(level: &Level, skeleton: &mut Skeleton) {
    let mut placed = Vec::new();
    for (index, parent) in skeleton.axes.iter().enumerate() {
        if parent.level + 1 == level.number {
            place_on(level, index, parent, &mut placed);
        }
    }
    skeleton.axes.extend(placed);
}

/// Places the laterals on `parent`, the axis at `index`: nodes along the part of the parent
/// inside the crown, as many as the level's density gives for its length, each placed by
/// `node_place`, its limbs turned about the parent by `node_azimuths` and their skew, each leaving
/// at the level's angle from the parent and reaching its share of the way to the envelope. A fork's
/// copy's nodes start from its base. Each lateral's base radius is the level's share of the
/// parent's radius where it leaves, so a level's radius changes that level alone.
fn place_on(level: &Level, index: usize, parent: &Axis, placed: &mut Vec<Axis>) {
    let (envelope, params) = (&level.envelope, level.params);
    let Some((inside_start, inside_end)) = parent.inside_part(envelope) else {
        return;
    };
    // The node count and each node's turn, limb count and place are the parent's, drawn from a
    // stream of its own beside its limbs'.
    let mut layout = parent.rng.fork(LAYOUT_STREAM);
    let length = parent.length();
    let inside = inside_end - inside_start;
    let inside_length = inside * length;
    let nodes = layout.round(params.density * inside_length / envelope.radius.max(0.01));
    let gap = inside_length / nodes.max(1) as f32;
    // Each limb placed so far: how far along the parent it leaves, and its azimuth.
    let mut crowd: Vec<(f32, f32)> = Vec::new();
    let mut limb = 0;
    for node in 0..nodes {
        let per_node = layout.round(params.per_node).max(1);
        let start = if parent.kind == AxisKind::ForkCopy {
            0.0
        } else {
            params.start
        };
        let place = node_place(node, nodes, start, layout.unit());
        let t = inside_start + inside * place;
        let at = t * length;
        let frame = parent.frame(t);
        let across = frame.side.cross(frame.tangent);
        let crowding = Crowd {
            at,
            gap,
            placed: &crowd,
        };
        let azimuths = node_azimuths(&frame, per_node, params.facing, &crowding, envelope);
        for azimuth in azimuths {
            let rng = parent.rng.fork(limb);
            limb += 1;
            let mut own = rng.fork(OWN_VALUES_STREAM);
            let angle = draw::<LimbLevel>(&params.angle, "angle", &mut own);
            let reach = draw::<LimbLevel>(&params.reach, "reach", &mut own);
            let azimuth = azimuth + (own.unit() * 2.0 - 1.0) * params.skew;
            crowd.push((at, azimuth));
            let (sin, cos) = azimuth.sin_cos();
            let (angle_sin, angle_cos) = angle.sin_cos();
            let direction =
                frame.tangent * angle_cos + (frame.side * cos + across * sin) * angle_sin;
            let reach = reach * distance_to_envelope(envelope, frame.point, direction);
            let start = Ring {
                tangent: direction,
                side: upper_side(direction, frame.side),
                radius: params.radius * frame.radius,
                ..frame.ring(0.0)
            };
            let kind = AxisKind::Lateral;
            placed.push(Axis::placed(
                level.number,
                kind,
                (index, at),
                start,
                reach,
                rng,
            ));
        }
    }
}

/// Where node `node` of `nodes` sits, as a fraction of the stretch bearing them: centred in its
/// equal share of the stretch past `start`, moved by up to `NODE_JITTER` of a share by `unit` (0
/// to 1).
pub(super) fn node_place(node: u32, nodes: u32, start: f32, unit: f32) -> f32 {
    let jitter = (unit * 2.0 - 1.0) * NODE_JITTER;
    let even = (node as f32 + 0.5 + jitter) / nodes as f32;
    let start = start.clamp(0.0, 1.0);
    start + (1.0 - start) * even
}

/// The limbs already on a parent, as a node about to be placed at `at` metres along it sees them.
struct Crowd<'a> {
    at: f32,
    /// The mean gap between nodes, in metres, over which crowding fades by `CROWDING_FADE`.
    gap: f32,
    /// Each limb's distance along the parent, in metres, and its azimuth.
    placed: &'a [(f32, f32)],
}

/// The azimuths of a node's `per_node` limbs, as `facing` places them before their skew:
/// - around, spaced evenly, turned to the least cost of `AZIMUTH_CANDIDATES` turns;
/// - to the sides, on alternate flanks, the first on the flank of lower cost;
/// - up or down, all on the parent's upper side or its underside.
///
/// A turn's cost is how crowded its limbs are by those already placed, less how far they point
/// out from the plant's axis. Crowding is how near each limb's azimuth is to each placed one's,
/// sharpened, and faded with their distance along the parent; so around, one limb a node settles
/// into a spiral of about 137.5°, pairs cross and whorls alternate, as phyllotaxis does.
fn node_azimuths(
    frame: &Frame,
    per_node: u32,
    facing: Facing,
    crowd: &Crowd,
    envelope: &Envelope,
) -> Vec<f32> {
    let across = frame.side.cross(frame.tangent);
    // The azimuth of the parent's upper side, nearest world up; an upright parent's is its side's.
    let up = Vec3::Y.reject_from_normalized(frame.tangent);
    let upper = if up.length_squared() > 1e-6 {
        up.dot(across).atan2(up.dot(frame.side))
    } else {
        0.0
    };
    // Out from the axis, as much as the node is off it, up to a crown radius.
    let out = Vec3::new(frame.point.x, 0.0, frame.point.z) / envelope.radius.max(0.01);
    let out = out.clamp_length_max(1.0);
    let cost = |azimuth: f32| {
        let crowding: f32 = crowd
            .placed
            .iter()
            .map(|&(at, other)| {
                let near = (1.0 + (azimuth - other).cos()) * 0.5;
                let fade = CROWDING_FADE.powf((crowd.at - at).abs() / crowd.gap.max(1e-3));
                fade * near.powi(CROWDING_SHARPNESS)
            })
            .sum();
        let (sin, cos) = azimuth.sin_cos();
        crowding - OUTWARD_WEIGHT * (frame.side * cos + across * sin).dot(out)
    };
    // The limbs from a first azimuth, `apart` between each, and the first of `candidates` whose
    // limbs cost least together.
    let group = |first: f32, apart: f32| -> Vec<f32> {
        (0..per_node)
            .map(|limb| first + apart * limb as f32)
            .collect()
    };
    let least = |candidates: &mut dyn Iterator<Item = f32>, apart: f32| -> Vec<f32> {
        let mut best = (0.0, f32::INFINITY);
        for first in candidates {
            let total: f32 = group(first, apart).into_iter().map(cost).sum();
            if total < best.1 {
                best = (first, total);
            }
        }
        group(best.0, apart)
    };
    match facing {
        Facing::Around => least(
            &mut (0..AZIMUTH_CANDIDATES).map(|i| TAU * i as f32 / AZIMUTH_CANDIDATES as f32),
            TAU / per_node as f32,
        ),
        Facing::Sides => least(&mut [upper + FRAC_PI_2, upper - FRAC_PI_2].into_iter(), PI),
        Facing::Up => group(upper, 0.0),
        Facing::Down => group(upper + PI, 0.0),
    }
}

/// A value drawn from `fuzzy`, clamped to the limits of `T`'s field `field`.
pub(super) fn draw<T: Typed>(fuzzy: &Fuzzy, field: &str, rng: &mut Rng) -> f32 {
    let limits = limits::<T>(field);
    fuzzy.sample(rng).clamp(*limits.start(), *limits.end())
}
