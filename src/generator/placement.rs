//! Where axes start and which way they leave: stems on the ground, and a level's laterals at
//! nodes along their parent.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::{
    math::{Quat, Vec2, Vec3},
    reflect::Typed,
};

use super::{
    envelope::{Envelope, distance_to_envelope, outline_point},
    foliage::straight_run,
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
/// Nodes placed below a parent's first, bearing no limbs, so its limbs' turns settle before the
/// first real node. A spiral starting from none takes about eight nodes to settle.
const PHANTOM_NODES: u32 = 10;
/// How much the gap after a node shrinks for the lean of the limbs so far (`balanced_places`):
/// after a lone limb, the next node comes at under half the gap; a settled spiral leans
/// about 0.2, and keeps its gaps alike.
const BALANCE_PULL: f32 = 0.6;
/// How much a limb pointing straight out from the plant's axis gains, a crown radius off it,
/// against crowding of 1 for a limb at the same azimuth one gap away.
const OUTWARD_WEIGHT: f32 = 1.0;

/// The stems: the trunk from the origin, upright at the ground and turning by the lean, if on, as
/// it rises toward a compass heading drawn from the seed (`stem_course`). With Fork on, it may fork
/// (D31): it ends at the fork, and copies carry it on in their directions (`copy_directions`),
/// each from the trunk's radius there and turning by what is left of the lean, away from the
/// middle, the rest of the trunk's height long, so every tip is the height from the ground along
/// the stems. Forked at its base, its copies take its place, several stems rising from the ground.
pub(super) fn stems(species: &Species) -> Vec<Axis> {
    let trunk = &species.trunk;
    let length = trunk.height.max(0.01);
    let heading = species.seed.stream("trunk").unit() * TAU;
    let rng = species.seed.stream("branching").fork(0);
    let trunk_lean = trunk.lean.get().map_or(0.0, |lean| lean.angle);
    let rings = stem_course(
        species,
        Vec3::ZERO,
        0.0,
        heading,
        trunk_lean,
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
    let lean = trunk_lean * (1.0 - drawn.at);
    let copies: Vec<Axis> = (copy_directions(&frame, fork, &drawn).into_iter())
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
                // Rolled to the trunk's side, so the first copy carries on its tube untwisted.
                let mut rings = rings;
                roll_to(&mut rings, frame.side);
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

/// Turns every ring's side about its tangent by one angle, the one bringing the first's to
/// `side`'s part across its tangent, so the tube keeps its own roll along it.
fn roll_to(rings: &mut [Ring], side: Vec3) {
    let first = rings[0];
    let Some(target) = side.reject_from_normalized(first.tangent).try_normalize() else {
        return;
    };
    let angle = first
        .side
        .cross(target)
        .dot(first.tangent)
        .atan2(first.side.dot(target));
    for ring in rings {
        ring.side = Quat::from_axis_angle(ring.tangent, angle) * ring.side;
    }
}

/// A stem's rings from `base`, `length` long: leaving `from_up` radians from up toward `heading`
/// and turning toward it by `lean`, but never past level, where it would sink again, then
/// zigzagging about that course by the trunk's wander, if on (`displace`), its tip where the
/// course ends and straight under its tip's card (`straight_run`), and narrowing from `radius` by
/// the plant's taper.
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
            straight_run(species),
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
/// at the level's angle from the parent, turned by Bias's `aim`, and reaching its share of the way
/// to the envelope, scaled by `bias_size`; the nodes then moved by `balanced_places`. A fork's
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
    let start = if parent.kind == AxisKind::ForkCopy {
        0.0
    } else {
        params.start
    };
    if nodes > 0 {
        // Nodes below the first, turned as real ones are but bearing no limbs, so the first real
        // node meets the pattern settled rather than starting it.
        let t = inside_start + inside * node_place(0, nodes, start, params.end, 0.5);
        let frame = parent.frame(t);
        let per_node = params.per_node.round().max(1.0) as u32;
        for phantom in (1..=PHANTOM_NODES).rev() {
            let at = t * length - phantom as f32 * gap;
            let crowding = Crowd {
                at,
                gap,
                placed: &crowd,
            };
            let azimuths = node_azimuths(&frame, per_node, params.facing, &crowding, envelope);
            crowd.extend(azimuths.into_iter().map(|azimuth| (at, azimuth)));
        }
    }
    let angle_limits = limits::<LimbLevel>("angle");
    let mut limb = 0;
    // Each node's place, and its limbs' drawn values: the places balanced once all are drawn.
    let mut places = Vec::with_capacity(nodes as usize);
    let mut node_limbs: Vec<Vec<Drawn>> = Vec::with_capacity(nodes as usize);
    for node in 0..nodes {
        let per_node = layout.round(params.per_node).max(1);
        let place = node_place(node, nodes, start, params.end, layout.unit());
        let t = inside_start + inside * place;
        let at = t * length;
        let crowding = Crowd {
            at,
            gap,
            placed: &crowd,
        };
        let azimuths =
            node_azimuths(&parent.frame(t), per_node, params.facing, &crowding, envelope);
        let mut drawn = Vec::with_capacity(azimuths.len());
        for azimuth in azimuths {
            let rng = parent.rng.fork(limb);
            limb += 1;
            let mut own = rng.fork(OWN_VALUES_STREAM);
            let angle = draw::<LimbLevel>(&params.angle, "angle", &mut own);
            let angle = angle.clamp(*angle_limits.start(), *angle_limits.end());
            let reach = draw::<LimbLevel>(&params.reach, "reach", &mut own);
            let azimuth = azimuth + (own.unit() * 2.0 - 1.0) * params.skew;
            crowd.push((at, azimuth));
            drawn.push(Drawn {
                angle,
                reach,
                azimuth,
                rng,
            });
        }
        places.push(place);
        node_limbs.push(drawn);
    }
    let azimuths: Vec<Vec<f32>> = (node_limbs.iter())
        .map(|limbs| limbs.iter().map(|limb| limb.azimuth).collect())
        .collect();
    let places = balanced_places(&places, &azimuths);
    for (node, (place, limbs)) in places.into_iter().zip(node_limbs).enumerate() {
        let node = node as u32;
        let t = inside_start + inside * place;
        let at = t * length;
        let frame = parent.frame(t);
        let across = frame.side.cross(frame.tangent);
        for Drawn {
            angle,
            reach,
            azimuth,
            rng,
        } in limbs
        {
            let size = bias_size(params, node, nodes);
            let (sin, cos) = azimuth.sin_cos();
            let (angle_sin, angle_cos) = angle.sin_cos();
            let mut direction =
                frame.tangent * angle_cos + (frame.side * cos + across * sin) * angle_sin;
            if let Some(bias) = params.bias.get().filter(|bias| bias.aim > 0.0) {
                let along = if nodes > 1 {
                    node as f32 / (nodes - 1) as f32
                } else {
                    0.5
                };
                direction = aim(envelope, frame.point, direction, along, bias.aim);
            }
            let reach = reach * size * distance_to_envelope(envelope, frame.point, direction);
            let start = Ring {
                tangent: direction,
                side: upper_side(direction, frame.side),
                radius: params.radius * frame.radius,
                ..frame.ring(0.0)
            };
            let kind = AxisKind::Lateral;
            placed.push(Axis {
                leaf_scale: parent.leaf_scale * bias_leaves(params, node, nodes),
                ..Axis::placed(level.number, kind, (index, at), start, reach, rng)
            });
        }
    }
}

/// A lateral's values drawn before its node's place is balanced: its angle from the parent, its
/// share of the way to the envelope, its azimuth after skew, and its stream.
struct Drawn {
    angle: f32,
    reach: f32,
    azimuth: f32,
    rng: Rng,
}

/// `places`, each node's along the parent, moved so the next node comes sooner while the limbs so
/// far lean to one side, as a lone limb low on the parent does: each gap shrinks by
/// `BALANCE_PULL` times the lean of the limbs at and below the node before it, then all are
/// scaled so the first and last nodes keep their places. The lean is the length of the sum of
/// the limbs' `azimuths` as unit vectors, faded by `CROWDING_FADE` a node, over their faded
/// count: 1 for one limb, 0 for a pair opposite.
pub(super) fn balanced_places(places: &[f32], azimuths: &[Vec<f32>]) -> Vec<f32> {
    let Some((&first, &last)) = places.first().zip(places.last()) else {
        return Vec::new();
    };
    let (mut sum, mut count) = (Vec2::ZERO, 0.0);
    let gaps: Vec<f32> = (places.windows(2).zip(azimuths))
        .map(|(pair, node)| {
            sum = sum * CROWDING_FADE + node.iter().map(|&a| Vec2::from_angle(a)).sum::<Vec2>();
            count = count * CROWDING_FADE + node.len() as f32;
            let lean = sum.length() / count.max(1e-6);
            (pair[1] - pair[0]) * (1.0 - BALANCE_PULL * lean)
        })
        .collect();
    let total: f32 = gaps.iter().sum();
    if total <= 0.0 {
        return places.to_vec();
    }
    let scale = (last - first) / total;
    let mut place = first;
    std::iter::once(first)
        .chain(gaps.iter().map(|gap| {
            place += gap * scale;
            place
        }))
        .collect()
}

/// The share of the reach for laterals at node `node` of `nodes`: with Bias on, 1 plus its size at
/// the first node, changing evenly through 1 at the middle to 1 less its size at the last;
/// otherwise, and for a lone node, 1.
pub(super) fn bias_size(params: &LimbLevel, node: u32, nodes: u32) -> f32 {
    match params.bias.get() {
        Some(bias) if nodes > 1 => {
            // −1 at the first node, 1 at the last.
            let place = 2.0 * node as f32 / (nodes - 1) as f32 - 1.0;
            1.0 - bias.size * place
        }
        _ => 1.0,
    }
}

/// The change to the leaves' size for laterals at node `node` of `nodes`: their `bias_size` over
/// the longest node's, so 1 at the longest and falling evenly from there, taken by Bias's leaves
/// share; otherwise, and for a lone node, 1.
pub(super) fn bias_leaves(params: &LimbLevel, node: u32, nodes: u32) -> f32 {
    match params.bias.get() {
        Some(bias) if nodes > 1 => {
            let ratio = bias_size(params, node, nodes) / (1.0 + bias.size.abs());
            1.0 - bias.leaves * (1.0 - ratio)
        }
        _ => 1.0,
    }
}

/// `direction`, from `origin`, turned by the share `aim` of the way toward the point `along` the
/// envelope's outline (`outline_point`) at its own compass heading. One pointing straight up or
/// down takes the heading out from the axis.
fn aim(envelope: &Envelope, origin: Vec3, direction: Vec3, along: f32, aim: f32) -> Vec3 {
    let heading = Vec3::new(direction.x, 0.0, direction.z)
        .try_normalize()
        .or_else(|| Vec3::new(origin.x, 0.0, origin.z).try_normalize())
        .unwrap_or(Vec3::X);
    let (out, height) = outline_point(envelope, along);
    let target = heading * out + Vec3::Y * height;
    let Some(toward) = (target - origin).try_normalize() else {
        return direction;
    };
    Quat::IDENTITY.slerp(Quat::from_rotation_arc(direction, toward), aim) * direction
}

/// Where node `node` of `nodes` sits, as a fraction of the stretch bearing them: centred in its
/// equal share of the stretch from `start` to `end`, moved by up to `NODE_JITTER` of a share by
/// `unit` (0 to 1). An end below the start puts every node at the start.
pub(super) fn node_place(node: u32, nodes: u32, start: f32, end: f32, unit: f32) -> f32 {
    let jitter = (unit * 2.0 - 1.0) * NODE_JITTER;
    let even = (node as f32 + 0.5 + jitter) / nodes as f32;
    let start = start.clamp(0.0, 1.0);
    let end = end.clamp(start, 1.0);
    start + (end - start) * even
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
