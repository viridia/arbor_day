//! Hinting (D17), as font hinting keeps only what a pixel shows: an axis keeps a ring at every
//! step of its trace until its level's last stage, which keeps only those the eye would miss, to
//! the bark's detail threshold. The next level is placed on the rings kept, so scions start on the
//! bark as meshed.

use bevy::math::Vec3;

use super::{
    skeleton::{Ring, Skeleton},
    trace::{TRACE_STEP, halfway},
};

/// Hints the rings of every axis of level `number` (`hint_rings`) to `tolerance`.
pub(super) fn hint(number: usize, tolerance: f32, skeleton: &mut Skeleton) {
    let axes = skeleton.axes.iter_mut();
    for axis in axes.filter(|axis| axis.level == number) {
        axis.rings = hint_rings(&axis.rings, tolerance);
    }
}

/// The rings `simplify` keeps at `tolerance`, less any within a trace step of the tip, since a
/// tube turning between two rings that close folds over itself. The start and the tip are kept as
/// they are, so tubes meet where axes join; each ring between faces `halfway`, its side made
/// square to that.
pub(super) fn hint_rings(rings: &[Ring], tolerance: f32) -> Vec<Ring> {
    let centers: Vec<Vec3> = rings.iter().map(|ring| ring.center).collect();
    let tip = rings.len() - 1;
    let mut kept = simplify(&centers, tolerance);
    kept.retain(|&i| i == 0 || i == tip || rings[i].at < rings[tip].at - TRACE_STEP);
    let mut hinted: Vec<Ring> = kept.iter().map(|&i| rings[i]).collect();
    for k in 1..hinted.len() - 1 {
        let (before, after) = (hinted[k - 1].center, hinted[k + 1].center);
        let ring = &mut hinted[k];
        ring.tangent = halfway(before, ring.center, after, ring.tangent);
        ring.side = (ring.side.reject_from_normalized(ring.tangent))
            .try_normalize()
            .unwrap_or_else(|| ring.tangent.any_orthonormal_vector());
    }
    hinted
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
