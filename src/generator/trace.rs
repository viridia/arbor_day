//! Tracing (D17). To trace an axis is to walk its course from its start ring in steps of
//! `TRACE_STEP`, recording a point at each: a limb's course turns toward its level's bend, and a
//! stem's is an arc, from its splay and lean. A limb's walk stops at its reach, or before a step
//! that would leave the envelope. Every point is kept as a ring, so the level's later stages work
//! on the path as traced, until it is hinted (`hint`); the wander (`wander`) then displaces it.

use bevy::math::{Quat, Vec3};

use super::{
    envelope::{Envelope, inside_envelope},
    skeleton::{Axis, Level, Ring, Skeleton},
};
use crate::species::LimbLevel;

/// The distance between kinks on every limb and stem, in crown radii: a limb's zigzag reads by
/// its period, so long limbs kink several times and short twigs once or not at all.
const KINK_SPACING: f32 = 0.15;

/// The step in tracing an axis, in metres.
pub(super) const TRACE_STEP: f32 = 0.02;

/// Traces the level's laterals just placed (`trace`), leaving out any that came out no longer
/// than they are thick, and adds the rest to their parents' scions.
pub(super) fn trace_scions(level: &Level, skeleton: &mut Skeleton) {
    let axes = &mut skeleton.axes;
    let first = (axes.iter().position(|axis| axis.level == level.number)).unwrap_or(axes.len());
    for mut axis in axes.split_off(first) {
        if trace(level, &mut axis) {
            let (parent, _) = axis.parent.expect("a limb grows from a parent");
            let id = axes.len();
            axes[parent].scions.push(id);
            axes.push(axis);
        }
    }
}

/// Traces a placed axis along its course for up to its reach (`trace_limb`), replacing its start
/// ring with its rings. Returns whether it came out longer than it is thick at its base, as an
/// axis must to be kept.
pub(super) fn trace(level: &Level, axis: &mut Axis) -> bool {
    let start = axis.rings[0];
    axis.rings = trace_limb(&level.envelope, level.params, start, axis.reach).finish();
    axis.rings.len() >= 2 && axis.length() > start.radius
}

/// A limb's course, traced from `start` for up to `length`, or to where the next step would leave
/// the envelope.
fn trace_limb(envelope: &Envelope, params: &LimbLevel, start: Ring, length: f32) -> Trace {
    let (bend_sin, bend_cos) = params.bend.toward.sin_cos();
    // The share of the angle left to the bend that one step closes. Shares compound, so a run of
    // steps closes the same share whatever the step.
    let kept_per_crown = 1.0 - params.bend.amount.clamp(0.0, 0.99);
    let crown = envelope.radius.max(0.01);

    // The course, and a side square to it carried along without twisting.
    let mut course = start.tangent;
    let mut course_side = start.side;
    let mut trace = Trace::new(start);
    let (mut center, mut at) = (start.center, 0.0);
    while at < length {
        let step = TRACE_STEP.min(length - at);
        let next = center + course * step;
        if !inside_envelope(envelope, next) {
            break;
        }
        center = next;
        at += step;
        // Toward the bend, keeping the compass heading; a course pointing straight up or down
        // takes its side's.
        let heading = Vec3::new(course.x, 0.0, course.z)
            .try_normalize()
            .or_else(|| Vec3::new(course_side.x, 0.0, course_side.z).try_normalize());
        if let Some(heading) = heading {
            let toward = Vec3::Y * bend_cos + heading * bend_sin;
            let closed = 1.0 - kept_per_crown.powf(step / crown);
            let turned = turn_toward(course, toward, course.angle_between(toward) * closed);
            course_side = (Quat::from_rotation_arc(course, turned) * course_side)
                .reject_from_normalized(turned)
                .normalize_or(course_side);
            course = turned;
        }
        trace.push(center, at, course);
    }
    trace
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

/// A stem's course (`Trace::finish`): an arc (`arc`) turned about the vertical to `heading`, which
/// lies in one vertical plane, so a side square to it in that plane never twists: azimuth 0 is
/// toward the heading.
pub(super) fn stem_rings(length: f32, splay: f32, lean: f32, heading: f32) -> Vec<Ring> {
    let turn = Quat::from_rotation_y(heading);
    let tangent = turn * arc(length, splay, lean, 0.0).1;
    let mut trace = Trace::new(Ring {
        center: Vec3::ZERO,
        tangent,
        side: tangent.cross(turn * Vec3::Z),
        at: 0.0,
        radius: 0.0,
    });
    let mut at = 0.0;
    while at < length {
        at += TRACE_STEP.min(length - at);
        let (center, tangent) = arc(length, splay, lean, at / length);
        trace.push(turn * center, at, turn * tangent);
    }
    trace.finish()
}

/// The distance between kinks on any limb or stem: `KINK_SPACING` crown radii, so a scaled
/// species keeps its form.
pub(super) fn kink_spacing(envelope: &Envelope) -> f32 {
    KINK_SPACING * envelope.radius.max(0.01)
}

/// A tube as it is traced: a point every step, from its start ring, and the direction it was
/// heading at the last.
struct Trace {
    start: Ring,
    /// Each point's center and distance along the tube, the start's first.
    points: Vec<(Vec3, f32)>,
    tip_tangent: Vec3,
}

impl Trace {
    fn new(start: Ring) -> Self {
        Self {
            start,
            points: vec![(start.center, 0.0)],
            tip_tangent: start.tangent,
        }
    }

    /// The point a step on, at `center` and `at`, the tube heading `tangent` there.
    fn push(&mut self, center: Vec3, at: f32, tangent: Vec3) {
        self.points.push((center, at));
        self.tip_tangent = tangent;
    }

    fn finish(self) -> Vec<Ring> {
        rings_through(self.start, &self.points, self.tip_tangent)
    }
}

/// A tube's rings, one at each of `points` (a center and a distance along the tube), the first
/// being `start`'s. The start ring is kept as it is, and the tip heads `tip_tangent`; each ring
/// between faces `halfway`, with its side carried along from the start's without twisting.
pub(super) fn rings_through(start: Ring, points: &[(Vec3, f32)], tip_tangent: Vec3) -> Vec<Ring> {
    let mut rings = vec![start];
    for (i, &(center, at)) in points.iter().enumerate().skip(1) {
        let previous = rings[rings.len() - 1];
        let tangent = match points.get(i + 1) {
            Some(&(next, _)) => halfway(previous.center, center, next, previous.tangent),
            None => tip_tangent,
        };
        let side = (Quat::from_rotation_arc(previous.tangent, tangent) * previous.side)
            .reject_from_normalized(tangent)
            .normalize_or(previous.side);
        rings.push(Ring {
            center,
            tangent,
            side,
            at,
            radius: 0.0,
        });
    }
    rings
}

/// The direction at `center` halfway between the chords from `before` and to `after`, so a tube
/// does not pinch there; `fallback` where the chord in has no length.
pub(super) fn halfway(before: Vec3, center: Vec3, after: Vec3, fallback: Vec3) -> Vec3 {
    let into = (center - before).normalize_or(fallback);
    (into + (after - center).normalize_or(into)).normalize_or(into)
}

/// Ends a tube's rings at `tip`, a ring along it. Rings past it go, and so do rings less than a
/// trace step before it, but never the base: a tube turning between two rings that close folds
/// over itself where it is drawn thicker than the gap.
pub(super) fn end_at(rings: &mut Vec<Ring>, tip: Ring) {
    rings.retain(|ring| ring.at == 0.0 || ring.at < tip.at - TRACE_STEP);
    if tip.at > rings[rings.len() - 1].at {
        rings.push(tip);
    }
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
