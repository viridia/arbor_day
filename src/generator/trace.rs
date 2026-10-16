//! Tracing an axis's course (D17): bend toward a direction, kinks about the course, and the rings
//! kept by hinting.

use std::f32::consts::TAU;

use bevy::math::{Mat3, Quat, Vec3};

use super::{
    envelope::{Envelope, inside_envelope},
    skeleton::Ring,
};
use crate::{random::Rng, species::LimbLevel};

/// The distance between kinks on every limb and stem, in crown radii: a limb's zigzag reads by
/// its period, so long limbs kink several times and short twigs once or not at all.
const KINK_SPACING: f32 = 0.15;
/// The most a stretch between kinks is longer or shorter than `KINK_SPACING`, as a share of it,
/// so kinks fall unevenly without two of them running together.
pub(super) const KINK_JITTER: f32 = 0.4;

/// The step in tracing a limb, in metres.
pub(super) const TRACE_STEP: f32 = 0.02;

/// A limb's rings, from `start` for up to `length` along it, kept to `tolerance`
/// (`Trace::finish`). Its course is traced in short steps from its start, turning toward the
/// level's bend, and the limb kinks about its course (`Kinks`), but not in its last `straight`,
/// where a tip's card lies (`foliage`). It stops early where it would leave the envelope, and is
/// then traced again to there, so its last stretch is straight wherever it ends.
pub(super) fn grow_limb(
    envelope: &Envelope,
    params: &LimbLevel,
    start: Ring,
    length: f32,
    kinks: Kinks,
    straight: f32,
    tolerance: f32,
) -> Vec<Ring> {
    let trace = |length: f32, kinks: Kinks| {
        trace_limb(envelope, params, start, length, length - straight, kinks)
    };
    let (mut traced, stopped) = trace(length, kinks.clone());
    if let Some(at) = stopped {
        traced = trace(at, kinks).0;
    }
    traced.finish(tolerance)
}

/// A limb traced as `grow_limb` describes, with no kinks from `end` on, and where it left the
/// envelope, if it did.
fn trace_limb(
    envelope: &Envelope,
    params: &LimbLevel,
    start: Ring,
    length: f32,
    end: f32,
    mut kinks: Kinks,
) -> (Trace, Option<f32>) {
    let (bend_sin, bend_cos) = params.bend_toward.sin_cos();
    // The share of the angle left to the bend that one step closes. Shares compound, so a run of
    // steps closes the same share whatever the step.
    let kept_per_crown = 1.0 - params.bend.clamp(0.0, 0.99);
    let crown = envelope.radius.max(0.01);

    // The course, and a side square to it carried along without twisting.
    let mut course = start.tangent;
    let mut course_side = start.side;
    let mut trace = Trace::new(start);
    let (mut center, mut tangent, mut at) = (start.center, start.tangent, 0.0);
    while at < length {
        let step = kinks.step(at, TRACE_STEP.min(length - at));
        let next = center + tangent * step;
        if !inside_envelope(envelope, next) {
            return (trace, Some(at));
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
        tangent = kinks.direction(at, end, course, course_side);
        trace.push(center, at, tangent);
    }
    (trace, None)
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

/// A stem's rings, kept to `tolerance` (`Trace::finish`). Its course is an arc (`arc`) turned
/// about the vertical to `heading`, which lies in one vertical plane, so a side square to it in
/// that plane never twists: azimuth 0 is toward the heading. The stem kinks about its course as a
/// limb does (`Kinks`).
pub(super) fn stem_rings(
    length: f32,
    splay: f32,
    lean: f32,
    heading: f32,
    mut kinks: Kinks,
    tolerance: f32,
) -> Vec<Ring> {
    let turn = Quat::from_rotation_y(heading);
    let course_at = |at: f32| {
        let (_, tangent) = arc(length, splay, lean, at / length);
        (turn * tangent, turn * tangent.cross(Vec3::Z))
    };
    let (tangent, side) = course_at(0.0);
    let mut trace = Trace::new(Ring {
        center: Vec3::ZERO,
        tangent,
        side,
        at: 0.0,
    });
    let (mut center, mut tangent, mut at) = (Vec3::ZERO, tangent, 0.0);
    while at < length {
        let step = kinks.step(at, TRACE_STEP.min(length - at));
        center += tangent * step;
        at += step;
        let (course, course_side) = course_at(at);
        tangent = kinks.direction(at, length, course, course_side);
        trace.push(center, at, tangent);
    }
    trace.finish(tolerance)
}

/// The distance between kinks on any limb or stem: `KINK_SPACING` crown radii, so a scaled
/// species keeps its form.
pub(super) fn kink_spacing(envelope: &Envelope) -> f32 {
    KINK_SPACING * envelope.radius.max(0.01)
}

/// Where a limb or stem kinks, and which way each stretch between kinks points off its course: up
/// to `wander` from it, about a random axis, held in the course's frame so it turns with the
/// course. Each stretch is `spacing` long on average, give or take `KINK_JITTER` of it, so kinks
/// fall unevenly. The first stretch follows the course. Offsets are from the course, not from the
/// stretch before, so the limb zigzags about its course rather than curling away from it.
#[derive(Clone)]
pub(super) struct Kinks {
    spacing: f32,
    wander: f32,
    next: f32,
    offset: Quat,
    rng: Rng,
}

impl Kinks {
    pub fn new(spacing: f32, wander: f32, rng: Rng) -> Self {
        let mut kinks = Self {
            spacing,
            wander,
            next: 0.0,
            offset: Quat::IDENTITY,
            rng,
        };
        kinks.next = kinks.stretch();
        kinks
    }

    /// The length of the next stretch, drawn.
    fn stretch(&mut self) -> f32 {
        let jitter = (self.rng.unit() * 2.0 - 1.0) * KINK_JITTER;
        (self.spacing * (1.0 + jitter)).max(TRACE_STEP)
    }

    /// `step`, cut short to land on the next kink.
    fn step(&self, at: f32, step: f32) -> f32 {
        step.min(self.next - at)
    }

    /// The direction at `at`, given the course there and a side square to the course. A kink
    /// falling there draws the next offset, unless it falls at or past `end`, from which the
    /// stretch it is on carries on to the tip.
    fn direction(&mut self, at: f32, end: f32, course: Vec3, side: Vec3) -> Vec3 {
        if at >= self.next - 1e-5 {
            if at < end {
                self.next += self.stretch();
                let tilt = self.rng.unit() * self.wander;
                let around = self.rng.unit() * TAU;
                self.offset =
                    Quat::from_axis_angle(Vec3::new(around.cos(), around.sin(), 0.0), tilt);
            } else {
                self.next = f32::INFINITY;
            }
        }
        let frame = Quat::from_mat3(&Mat3::from_cols(side, course.cross(side), course));
        (frame * self.offset * Vec3::Z).normalize()
    }
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

    /// The tube's rings: the points kept by `simplify` at `tolerance`, less any within a step of
    /// the tip (`end_at`). The start ring is kept as it is, and the tip heads as the tube did; each
    /// ring between faces halfway between the chords meeting there, so the tube does not pinch,
    /// with its side carried along from the start's without twisting.
    fn finish(self, tolerance: f32) -> Vec<Ring> {
        let centers: Vec<Vec3> = self.points.iter().map(|(center, _)| *center).collect();
        let mut kept = simplify(&centers, tolerance);
        let tip = self.points.len() - 1;
        let tip_at = self.points[tip].1;
        kept.retain(|&i| i == 0 || i == tip || self.points[i].1 < tip_at - TRACE_STEP);
        let mut rings = vec![self.start];
        for (k, &i) in kept.iter().enumerate().skip(1) {
            let (center, at) = self.points[i];
            let previous = rings[rings.len() - 1];
            let into = (center - previous.center).normalize_or(previous.tangent);
            let tangent = match kept.get(k + 1) {
                Some(&next) => {
                    (into + (self.points[next].0 - center).normalize_or(into)).normalize_or(into)
                }
                None => self.tip_tangent,
            };
            let side = (Quat::from_rotation_arc(previous.tangent, tangent) * previous.side)
                .reject_from_normalized(tangent)
                .normalize_or(previous.side);
            rings.push(Ring {
                center,
                tangent,
                side,
                at,
            });
        }
        rings
    }
}

/// The indices of the points of a polyline that Ramer, Douglas and Peucker keep at `tolerance`:
/// both ends, and between two kept points the one furthest from the chord joining them, if it lies
/// further than `tolerance`, and so on within each half. What is left out lies within `tolerance`
/// of the polyline kept.
pub(super) fn simplify(points: &[Vec3], tolerance: f32) -> Vec<usize> {
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut spans = vec![(0, points.len() - 1)];
    while let Some((a, b)) = spans.pop() {
        let furthest = (a + 1..b)
            .map(|i| (i, distance_to_segment(points[i], points[a], points[b])))
            .max_by(|x, y| x.1.total_cmp(&y.1));
        if let Some((i, distance)) = furthest
            && distance > tolerance
        {
            keep[i] = true;
            spans.push((a, i));
            spans.push((i, b));
        }
    }
    (0..points.len()).filter(|&i| keep[i]).collect()
}

/// How far `point` lies from the segment from `a` to `b`.
pub(super) fn distance_to_segment(point: Vec3, a: Vec3, b: Vec3) -> f32 {
    let along = b - a;
    let t = if along.length_squared() > 0.0 {
        ((point - a).dot(along) / along.length_squared()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (a + along * t - point).length()
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
