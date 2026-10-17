//! Forks (D31): a limb ending partway, and copies of it carrying it on.

use std::f32::consts::TAU;

use bevy::math::Quat;

use super::{
    envelope::distance_to_envelope,
    placement::draw,
    skeleton::{
        Axis, AxisKind, FORK_STREAM, Level, OWN_VALUES_STREAM, Ring, Skeleton, taper_rings,
        upper_side,
    },
    trace::{end_at, trace},
};

/// Forks each of the level's axes with the chance its level gives (`fork_axis`), adding its
/// copies as its scions. Copies do not fork again.
pub(super) fn fork(level: &Level, skeleton: &mut Skeleton) {
    let axes = &mut skeleton.axes;
    for index in 0..axes.len() {
        if axes[index].level != level.number {
            continue;
        }
        let copies = fork_axis(level, index, &mut axes[index]);
        let first = axes.len();
        axes[index].scions.extend(first..first + copies.len());
        axes.extend(copies);
    }
}

/// Forks `limb`, the axis at `index`, with the chance its level gives, returning the copies
/// carrying it on, or none. It ends at the fork, at its radius there, and the copies leave from
/// there, spread evenly around its direction, each turned from it by half the fork angle, the
/// first toward its flank turned up by the fork roll: so two copies make a Y lying level at no
/// roll and upright at 90°. Each copy starts at the limb's radius at the fork and is grown and
/// tapered as a limb of its level is, reaching toward the envelope. A copy that would be no longer
/// than it is thick is left out, and with no copies the limb does not fork.
fn fork_axis(level: &Level, index: usize, limb: &mut Axis) -> Vec<Axis> {
    let params = level.params;
    let streams = limb.rng.fork(FORK_STREAM);
    let mut draws = streams.fork(OWN_VALUES_STREAM);
    if draws.unit() >= params.fork_chance {
        return Vec::new();
    }
    let at = draw(&params.fork_at, "fork_at", &mut draws);
    let (half_sin, half_cos) = (draw(&params.fork_angle, "fork_angle", &mut draws) * 0.5).sin_cos();
    let frame = limb.frame(at);
    let fork_at = at * limb.length();
    let upper = upper_side(frame.tangent, frame.side);
    let (roll_sin, roll_cos) = params.fork_roll.sin_cos();
    let first = frame.tangent.cross(upper) * roll_cos + upper * roll_sin;
    let count = params.fork_copies.max(2) as u32;
    let copies: Vec<Axis> = (0..count)
        .filter_map(|copy| {
            let rng = streams.fork(copy as u64);
            let around = Quat::from_axis_angle(frame.tangent, TAU * copy as f32 / count as f32);
            let direction = frame.tangent * half_cos + around * first * half_sin;
            let reach = draw(&params.reach, "reach", &mut rng.fork(OWN_VALUES_STREAM))
                * distance_to_envelope(&level.envelope, frame.point, direction);
            let start = Ring {
                tangent: direction,
                side: upper_side(direction, frame.side),
                ..frame.ring(0.0)
            };
            let kind = AxisKind::ForkCopy;
            let parent = (index, fork_at);
            let mut copy = Axis::placed(limb.level, kind, parent, start, reach, rng);
            trace(level, &mut copy).then(|| {
                taper_rings(&mut copy.rings, frame.radius, level.taper);
                copy
            })
        })
        .collect();
    if !copies.is_empty() {
        end_at(&mut limb.rings, frame.ring(fork_at));
    }
    copies
}
