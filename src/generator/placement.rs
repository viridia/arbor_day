//! Where axes start and which way they leave: stems on the ground, and a level's limbs at nodes
//! along their parent.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::math::Vec3;

use super::{
    envelope::{Envelope, distance_to_envelope},
    foliage::AXIAL_FOLD,
    fork::fork,
    skeleton::{
        Axis, AxisKind, CARRY_ON_STREAM, Frame, LAYOUT_STREAM, OWN_VALUES_STREAM, Ring,
        WANDER_STREAM, upper_side,
    },
    trace::{Kinks, grow_limb, kink_spacing, stem_rings},
};
use crate::{
    fuzzy::Fuzzy,
    random::Rng,
    species::{Facing, LimbLevel, Species},
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

/// The stems: `stems` of them from the origin, spaced evenly around, each leaving the ground at
/// the splay from up and turning by the lean as it rises, away from the middle, but never past
/// level, where it would sink again, and kinking about that course by the trunk's wander. The
/// first's compass heading is drawn from the seed.
pub(super) fn stems(species: &Species) -> Vec<Axis> {
    let trunk = &species.trunk;
    let count = trunk.stems.max(1) as u32;
    let length = trunk.height.max(0.01);
    let lean = trunk.lean.min((FRAC_PI_2 - trunk.splay).max(0.0));
    let heading = species.seed.stream("trunk").unit() * TAU;
    let branching = species.seed.stream("branching");
    let top = trunk.radius * (1.0 - species.taper.clamp(0.0, 1.0));
    let spacing = kink_spacing(&Envelope::of(species));
    (0..count)
        .map(|stem| {
            let rng = branching.fork(stem as u64);
            let kinks = Kinks::new(spacing, trunk.wander, rng.fork(WANDER_STREAM));
            let rings = stem_rings(
                length,
                trunk.splay,
                lean,
                heading + TAU * stem as f32 / count as f32,
                kinks,
                species.detail,
            );
            Axis::new(0, AxisKind::Stem, rings, (trunk.radius, top), rng)
        })
        .collect()
}

/// What a parent's limbs leave it.
pub(super) struct Bearing {
    /// Its limbs, and the one carrying it on, as indices in the plant's axes.
    pub scions: Vec<usize>,
    /// Its new tip, at its last node, and its radius there, if a limb carries it on from there.
    pub tip: Option<(Ring, f32)>,
}

/// Adds one parent's limbs of level `level` to `grown`, whose first lies at `first_id` among the
/// plant's axes: nodes along the part of the parent inside the crown, as many as the level's
/// density gives for its length, each placed by `node_place`, its limbs turned about the parent by
/// `node_azimuths` and their skew, each leaving at the level's angle from the parent and then
/// grown (`grow_limb`). A limb whose start lies outside the crown, or that would be no longer than
/// it is thick, is left out.
///
/// A parent with limbs ends at its last node, and one more limb of the level carries it on from
/// there, along it: at angle 0, with no skew, but otherwise grown as the others are. A parent
/// `forked` is carried on by its copies instead, and a copy's nodes start from its base. Each limb
/// grown may fork (`fork`).
///
/// Every axis narrows evenly by the plant's taper, from its base to its tip as grown, and each
/// limb's base radius is the level's share of the parent's radius where it leaves, so a level's
/// radius changes that level alone. A parent cut at its last node keeps the radius it has there,
/// and the limb carrying it on starts at that radius.
pub(super) fn grow_level(
    species: &Species,
    level: usize,
    parent: &Axis,
    forked: bool,
    first_id: usize,
    grown: &mut Vec<Axis>,
) -> Bearing {
    let envelope = &Envelope::of(species);
    let params = &species.grown_levels()[level - 1];
    let taper = species.taper.clamp(0.0, 1.0);
    let spacing = kink_spacing(envelope);
    let kinks = |rng: &Rng| Kinks::new(spacing, params.wander, rng.fork(WANDER_STREAM));
    let tolerance = species.detail;
    // Where a tip's card lies, from its base to its fold, whether or not leaves grow, so turning
    // them off leaves the branches alone.
    let straight = AXIAL_FOLD * species.foliage.size * envelope.radius.max(0.01);
    // The parent's radius `t` (0 to 1) along it as grown.
    let radius_at = |t: f32| parent.radii.0 * (1.0 - taper * t);
    // A limb grown, added with its copies if it forks; returns its index.
    let add = |mut limb: Axis, grown: &mut Vec<Axis>| {
        let copies = fork(
            &mut limb, params, envelope, taper, spacing, straight, tolerance,
        );
        let id = first_id + grown.len();
        limb.scions.extend(id + 1..=id + copies.len());
        grown.push(limb);
        grown.extend(copies);
        id
    };
    let Some((inside_start, inside_end)) = parent.inside_part(envelope) else {
        return Bearing {
            scions: Vec::new(),
            tip: None,
        };
    };
    let mut scions = Vec::new();
    // Where along the parent the last limb grown leaves it.
    let mut last_node = 0.0;
    // The node count and each node's turn, limb count and place are the parent's, drawn from a
    // stream of its own beside its limbs'.
    let mut layout = parent.rng.fork(LAYOUT_STREAM);
    let length = parent.rings[parent.rings.len() - 1].at;
    let inside = inside_end - inside_start;
    let inside_length = inside * length;
    let nodes = layout.round(params.density * inside_length / envelope.radius.max(0.01));
    let gap = inside_length / nodes.max(1) as f32;
    // Each limb placed so far: how far along the parent it leaves, and its azimuth.
    let mut placed: Vec<(f32, f32)> = Vec::new();
    let mut index = 0;
    for node in 0..nodes {
        let per_node = layout.round(params.per_node).max(1);
        let start = if parent.kind == AxisKind::ForkCopy {
            0.0
        } else {
            params.start
        };
        let place = node_place(node, nodes, start, layout.unit());
        let t = inside_start + inside * place;
        let frame = parent.frame(t);
        let across = frame.side.cross(frame.tangent);
        let crowd = Crowd {
            at: t * length,
            gap,
            placed: &placed,
        };
        let azimuths = node_azimuths(&frame, per_node, params.facing, &crowd, envelope);
        let base_radius = params.radius * radius_at(t);
        for azimuth in azimuths {
            let rng = parent.rng.fork(index);
            index += 1;
            let mut own = rng.fork(OWN_VALUES_STREAM);
            let angle = draw(&params.angle, "angle", &mut own);
            let reach = draw(&params.reach, "reach", &mut own);
            let azimuth = azimuth + (own.unit() * 2.0 - 1.0) * params.skew;
            placed.push((t * length, azimuth));
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
            let rings = grow_limb(
                envelope,
                params,
                start,
                reach,
                kinks(&rng),
                straight,
                tolerance,
            );
            if rings.len() >= 2 && rings[rings.len() - 1].at > base_radius {
                last_node = t;
                let radii = (base_radius, base_radius * (1.0 - taper));
                let limb = Axis::new(level, AxisKind::Lateral, rings, radii, rng);
                scions.push(add(limb, grown));
            }
        }
    }
    if scions.is_empty() || forked {
        return Bearing { scions, tip: None };
    }
    let top = radius_at(last_node);
    // The limb carrying the parent on, from a stream of its own, so the limbs before it leave its
    // draws alone. It starts on the parent's new tip, turned as that is, so the two tubes meet.
    let frame = parent.frame(last_node);
    let tip = Ring {
        center: frame.point,
        tangent: frame.tangent,
        side: frame.side,
        at: last_node * length,
    };
    let rng = parent.rng.fork(CARRY_ON_STREAM);
    let reach = draw(&params.reach, "reach", &mut rng.fork(OWN_VALUES_STREAM))
        * distance_to_envelope(envelope, frame.point, frame.tangent);
    let start = Ring { at: 0.0, ..tip };
    let rings = grow_limb(
        envelope,
        params,
        start,
        reach,
        kinks(&rng),
        straight,
        tolerance,
    );
    let carried_on = rings.len() >= 2 && rings[rings.len() - 1].at > top;
    if carried_on {
        let radii = (top, top * (1.0 - taper));
        let limb = Axis::new(level, AxisKind::CarryOn, rings, radii, rng);
        scions.push(add(limb, grown));
    }
    Bearing {
        scions,
        tip: carried_on.then_some((tip, top)),
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

/// A value drawn from `fuzzy`, clamped to the limits of `LimbLevel`'s field `field`.
pub(super) fn draw(fuzzy: &Fuzzy, field: &str, rng: &mut Rng) -> f32 {
    let limits = LimbLevel::limits(field);
    fuzzy.sample(rng).clamp(*limits.start(), *limits.end())
}
