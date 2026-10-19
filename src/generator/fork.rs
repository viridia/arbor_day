//! Forks (D31): a stem or limb ending partway, and copies of it carrying it on.

use std::f32::consts::TAU;

use bevy::math::{Quat, Vec3};

use super::{
    placement::draw,
    skeleton::{
        Axis, AxisKind, FORK_STREAM, Frame, Level, OWN_VALUES_STREAM, Ring, Skeleton, taper_rings,
        upper_side,
    },
    trace::{end_at, trace},
    wander::wander_axis,
};
use crate::{
    random::Rng,
    species::Fork,
};

/// A fork as drawn for one axis.
pub(super) struct Drawn {
    /// Where along the axis, as a share of its length.
    pub at: f32,
    /// Half the angle across the fork: each copy's turn from the axis.
    pub half_angle: f32,
    /// Where the copies lie around the axis (`copy_directions`).
    pub roll: f32,
    /// The stream each copy's own forks from, by its number.
    pub copies: Rng,
}

/// Whether the axis whose stream is `rng` forks, with `fork`'s chance, and if so where and how
/// widely, drawn from a stream of the fork's.
pub(super) fn draw_fork(fork: &Fork, rng: &Rng) -> Option<Drawn> {
    let copies = rng.fork(FORK_STREAM);
    let mut draws = copies.fork(OWN_VALUES_STREAM);
    if draws.unit() >= fork.chance {
        return None;
    }
    let at = draw::<Fork>(&fork.at, "at", &mut draws);
    let half_angle = draw::<Fork>(&fork.angle, "angle", &mut draws) * 0.5;
    let roll = draw::<Fork>(&fork.roll, "roll", &mut draws);
    Some(Drawn {
        at,
        half_angle,
        roll,
        copies,
    })
}

/// Whether a fork `at` metres along an axis, where its radius is `radius`, is at its base, so its
/// copies take its place rather than carrying on a stub no longer than it is thick.
pub(super) fn at_base(at: f32, radius: f32) -> bool {
    at < radius
}

/// The directions of `fork`'s copies leaving `frame`: spread evenly around its tangent, each turned
/// from it by the drawn half angle, the first toward its flank turned up by the drawn roll, so two
/// copies make a Y lying level at no roll and upright at 90°.
pub(super) fn copy_directions(frame: &Frame, fork: &Fork, drawn: &Drawn) -> Vec<Vec3> {
    let (half_sin, half_cos) = drawn.half_angle.sin_cos();
    let upper = upper_side(frame.tangent, frame.side);
    let (roll_sin, roll_cos) = drawn.roll.sin_cos();
    let first = frame.tangent.cross(upper) * roll_cos + upper * roll_sin;
    let count = fork.copies.max(2) as u32;
    (0..count)
        .map(|copy| {
            let around = Quat::from_axis_angle(frame.tangent, TAU * copy as f32 / count as f32);
            frame.tangent * half_cos + around * first * half_sin
        })
        .collect()
}

/// Forks each of the level's axes with the chance its level gives (`fork_axis`), adding its
/// copies as its scions; or, forked at its base, its copies take its place, the first in it and
/// the rest as laterals beside it. Copies do not fork again.
pub(super) fn fork(level: &Level, skeleton: &mut Skeleton) {
    let axes = &mut skeleton.axes;
    for index in 0..axes.len() {
        if axes[index].level != level.number {
            continue;
        }
        let (copies, at_base) = fork_axis(level, index, &mut axes[index]);
        let first = axes.len();
        if !at_base {
            axes[index].scions.extend(first..first + copies.len());
            axes.extend(copies);
            continue;
        }
        let (kind, parent) = (axes[index].kind, axes[index].parent);
        let mut copies = copies.into_iter().map(|copy| Axis {
            kind: AxisKind::Lateral,
            parent,
            ..copy
        });
        let Some(replacement) = copies.next() else {
            continue;
        };
        axes[index] = Axis {
            kind,
            ..replacement
        };
        axes.extend(copies);
        let added = first..axes.len();
        if let Some((parent, _)) = parent {
            axes[parent].scions.extend(added);
        }
    }
}

/// Forks `limb`, the axis at `index`, if its level's fork is on, with the chance it gives,
/// returning the copies carrying it on, or none, and whether the fork is at its base (`at_base`).
/// Unless it is, the limb ends at the fork, at its radius there. The copies leave from there in
/// their directions (`copy_directions`), each starting at the limb's radius at the fork and grown,
/// wandering and tapered as a limb of its level is, reaching what the limb had left to grow past
/// the fork, so a short limb forks short. A copy that would
/// be no longer than it is thick is left out, and with no copies the limb does not fork.
fn fork_axis(level: &Level, index: usize, limb: &mut Axis) -> (Vec<Axis>, bool) {
    let params = level.params;
    let Some(fork) = params.fork.get() else {
        return (Vec::new(), false);
    };
    let Some(drawn) = draw_fork(fork, &limb.rng) else {
        return (Vec::new(), false);
    };
    let frame = limb.frame(drawn.at);
    let fork_at = drawn.at * limb.length();
    // What the limb had left to grow past the fork.
    let reach = (limb.reach - fork_at).max(0.0);
    let copies: Vec<Axis> = (copy_directions(&frame, fork, &drawn).into_iter())
        .enumerate()
        .filter_map(|(copy, direction)| {
            let rng = drawn.copies.fork(copy as u64);
            let start = Ring {
                tangent: direction,
                side: upper_side(direction, frame.side),
                ..frame.ring(0.0)
            };
            let kind = AxisKind::ForkCopy;
            let parent = (index, fork_at);
            let mut copy = Axis::placed(limb.level, kind, parent, start, reach, rng);
            trace(level, &mut copy).then(|| {
                wander_axis(level, &mut copy);
                taper_rings(&mut copy.rings, frame.radius, level.taper);
                copy
            })
        })
        .collect();
    let at_base = at_base(fork_at, frame.radius);
    if !copies.is_empty() && !at_base {
        end_at(&mut limb.rings, frame.ring(fork_at));
    }
    (copies, at_base)
}
