//! Wander (D17), as a lightning bolt is drawn: an axis traced along its course is displaced by
//! midpoints between two fixed points, the end of a straight run at its base, so it leaves along
//! its course, and the end of its course, or the start of the straight run under a tip's card.
//! The middle of each segment moves square to it, a random amount in a random direction, then each
//! half likewise, down to segments of about one kink spacing. No move is larger than a segment of
//! two spacings would get, so a long axis zigzags evenly about its course rather than turning on
//! one large corner, and its tip stays where the course puts it, at any wander.

use std::f32::consts::TAU;

use bevy::math::Vec3;

use super::{
    envelope::{ENVELOPE_BISECTIONS, Envelope, inside_envelope},
    skeleton::{Axis, Level, Ring, Skeleton, WANDER_STREAM},
    trace::rings_through,
};
use crate::random::Rng;

/// The most a split point moves from a segment's middle, as a share of the segment, either way, so
/// kinks fall unevenly without two of them running together.
pub(super) const KINK_JITTER: f32 = 0.2;
/// A segment is split while it is longer than this many kink spacings, so the segments left
/// average about one.
pub(super) const SPLIT_LENGTH: f32 = 1.5;
/// Halvings of a midpoint's displacement before it is dropped, when either half would point
/// further off the course than the wander.
const CLAMP_HALVINGS: u32 = 6;
/// The most the straight run at an axis's base is longer or shorter than a kink spacing, as a
/// share of it.
const BASE_JITTER: f32 = 0.4;

/// Displaces each of the level's axes by its wander (`wander_axis`).
pub(super) fn wander(level: &Level, skeleton: &mut Skeleton) {
    let axes = skeleton.axes.iter_mut();
    for axis in axes.filter(|axis| axis.level == level.number) {
        wander_axis(level, axis);
    }
}

/// Displaces a limb of `level` by its wander (`displace`), from its own wander stream, short of its
/// straight run and inside the envelope.
pub(super) fn wander_axis(level: &Level, axis: &mut Axis) {
    let end = axis.length() - level.straight;
    let rng = axis.rng.fork(WANDER_STREAM);
    let wander = level.params.wander.angle;
    let envelope = Some(&level.envelope);
    displace(&mut axis.rings, rng, wander, level.spacing, end, envelope);
}

/// Displaces `rings`, traced along a course, from about a kink spacing along them (`BASE_JITTER`)
/// to `end` metres along them, by midpoints (`split`); rings before and after stay on the course. Each stretch points no further off the
/// course than `wander`. Rings displaced outside `envelope`, if given, are pulled back toward the
/// course until inside. The rings are then laid again (`rings_through`) through a point at each
/// ring and each split, each `at` the distance along them.
pub(super) fn displace(
    rings: &mut Vec<Ring>,
    mut rng: Rng,
    wander: f32,
    spacing: f32,
    end: f32,
    envelope: Option<&Envelope>,
) {
    let start = spacing * (1.0 + (rng.unit() * 2.0 - 1.0) * BASE_JITTER);
    if wander <= 0.0 || rings.len() < 2 || end <= start {
        return;
    }
    let course = Course(rings);
    let mut splits = vec![(0.0, Vec3::ZERO), (start, Vec3::ZERO)];
    let bounds = Bounds {
        wander,
        tan: wander.tan(),
        spacing,
    };
    split(
        &course,
        &bounds,
        (start, Vec3::ZERO),
        (end, Vec3::ZERO),
        &mut rng,
        &mut splits,
    );
    splits.push((end, Vec3::ZERO));

    // A point at every ring and every split, so each kink is a corner.
    let mut ats: Vec<f32> = rings.iter().map(|ring| ring.at).collect();
    ats.extend(splits.iter().map(|&(at, _)| at));
    ats.sort_by(f32::total_cmp);
    ats.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
    let mut points: Vec<(Vec3, f32)> = ats
        .iter()
        .map(|&at| {
            let on_course = course.at(at).0;
            let mut center = on_course + offset_at(&splits, at);
            if let Some(envelope) = envelope
                && at > 0.0
                && !inside_envelope(envelope, center)
            {
                let (mut inner, mut outer) = (on_course, center);
                for _ in 0..ENVELOPE_BISECTIONS {
                    let middle = (inner + outer) * 0.5;
                    if inside_envelope(envelope, middle) {
                        inner = middle;
                    } else {
                        outer = middle;
                    }
                }
                center = inner;
            }
            (center, 0.0)
        })
        .collect();
    for i in 1..points.len() {
        points[i].1 = points[i - 1].1 + points[i].0.distance(points[i - 1].0);
    }
    let tip_tangent = rings[rings.len() - 1].tangent;
    *rings = rings_through(rings[0], &points, tip_tangent);
}

/// The traced course, its rings undisplaced.
struct Course<'a>(&'a [Ring]);

impl Course<'_> {
    /// The course's point and side `at` metres along it, between the rings either side.
    fn at(&self, at: f32) -> (Vec3, Vec3) {
        let rings = self.0;
        let next = rings
            .partition_point(|ring| ring.at <= at)
            .clamp(1, rings.len() - 1);
        let (a, b) = (rings[next - 1], rings[next]);
        let along = if b.at > a.at {
            ((at - a.at) / (b.at - a.at)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (a.center.lerp(b.center, along), a.side)
    }
}

/// What every split reads: the wander and its tangent, and the kink spacing.
struct Bounds {
    wander: f32,
    tan: f32,
    spacing: f32,
}

/// Splits the segment from `a` to `b`, each a distance along the course and the offset from it
/// there, if it is longer than `SPLIT_LENGTH` spacings, pushing the split points onto `splits` in
/// order along the course. The split point lies within `KINK_JITTER` of the segment's middle, and
/// moves square to the segment by up to the shorter half's length, or a spacing if less, times the
/// tangent of the wander, in a random direction, halved until neither half points further off the course than
/// the wander.
fn split(
    course: &Course,
    bounds: &Bounds,
    a: (f32, Vec3),
    b: (f32, Vec3),
    rng: &mut Rng,
    splits: &mut Vec<(f32, Vec3)>,
) {
    let length = b.0 - a.0;
    if length <= SPLIT_LENGTH * bounds.spacing {
        return;
    }
    let share = 0.5 + (rng.unit() * 2.0 - 1.0) * KINK_JITTER;
    let around = rng.unit() * TAU;
    let amount = rng.unit();

    let at = a.0 + length * share;
    let (_, side) = course.at(at);
    let point = |(at, offset): (f32, Vec3)| course.at(at).0 + offset;
    let chord = (point(b) - point(a)).normalize_or(Vec3::Y);
    let across = side
        .reject_from_normalized(chord)
        .try_normalize()
        .unwrap_or_else(|| chord.any_orthonormal_vector());
    let direction = across * around.cos() + chord.cross(across) * around.sin();
    let between = a.1.lerp(b.1, share);
    let shorter = (length * share.min(1.0 - share)).min(bounds.spacing);

    // Whether the stretch from `x` to `y` points within the wander of the course between them.
    let within = |x: (f32, Vec3), y: (f32, Vec3)| {
        let along = course.at(y.0).0 - course.at(x.0).0;
        (point(y) - point(x)).angle_between(along) <= bounds.wander + 1e-4
    };
    let mut reach = amount * shorter * bounds.tan;
    let mut middle = (at, between);
    for _ in 0..CLAMP_HALVINGS {
        let moved = (at, between + direction * reach);
        if within(a, moved) && within(moved, b) {
            middle = moved;
            break;
        }
        reach *= 0.5;
    }

    split(course, bounds, a, middle, rng, splits);
    splits.push(middle);
    split(course, bounds, middle, b, rng, splits);
}

/// The offset from the course `at` metres along it, between the split points either side.
fn offset_at(splits: &[(f32, Vec3)], at: f32) -> Vec3 {
    let next = splits
        .partition_point(|&(split, _)| split <= at)
        .clamp(1, splits.len() - 1);
    let ((a, from), (b, to)) = (splits[next - 1], splits[next]);
    if at >= b || b <= a {
        return if at >= b { to } else { from };
    }
    from.lerp(to, (at - a) / (b - a))
}
