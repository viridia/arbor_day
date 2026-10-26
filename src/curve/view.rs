//! A curve's view: the mapping between the curve's units and a plot node's pixels, and the curve
//! flattened to a polyline in those pixels.

use bevy::prelude::*;

use super::model::{CurveConstraints, CurveValue, PreparedCurve};

/// The farthest a polyline strays from the curve it stands for, in pixels.
const FLATNESS: f32 = 0.25;

/// How many times a piece of a segment may be halved.
const MAX_DEPTH: u32 = 12;

/// The most points a [`CurvePath`] holds: as many as the path material draws.
pub const MAX_PATH_POINTS: usize = 257;

/// The mapping between a curve's units and a plot's pixels: the axes' bounds stretched over the
/// plot, y up, inset so a stroke at the bounds is not cut off.
#[derive(Clone, Copy, Debug)]
pub struct CurveView {
    /// The plot's size, in logical pixels.
    pub size: Vec2,
    /// The margin kept inside each edge, in logical pixels.
    pub inset: f32,
    /// The curve's units at the plot's bottom left.
    pub min: Vec2,
    /// The curve's units at the plot's top right.
    pub max: Vec2,
}

impl CurveView {
    /// The axes of `constraints` over a plot `size` pixels, inset by `inset`.
    pub fn new(constraints: &CurveConstraints, size: Vec2, inset: f32) -> Self {
        Self {
            size,
            inset,
            min: Vec2::new(constraints.x.min, constraints.y.min),
            max: Vec2::new(constraints.x.max, constraints.y.max),
        }
    }

    /// The pixel, from the plot's top left, at `point` in the curve's units.
    pub fn to_pixels(self, point: Vec2) -> Vec2 {
        let share = (point - self.min) / self.span();
        let inner = self.inner();
        Vec2::new(
            self.inset + share.x * inner.x,
            self.inset + (1.0 - share.y) * inner.y,
        )
    }

    /// The point, in the curve's units, at `pixel` from the plot's top left.
    pub fn to_curve(self, pixel: Vec2) -> Vec2 {
        let share = (pixel - Vec2::splat(self.inset)) / self.inner().max(Vec2::ONE);
        self.min + Vec2::new(share.x, 1.0 - share.y) * self.span()
    }

    /// A movement of `delta` pixels, y down, in the curve's units, y up.
    pub fn delta_to_curve(self, delta: Vec2) -> Vec2 {
        Vec2::new(delta.x, -delta.y) / self.inner().max(Vec2::ONE) * self.span()
    }

    fn inner(self) -> Vec2 {
        (self.size - Vec2::splat(2.0 * self.inset)).max(Vec2::ZERO)
    }

    fn span(self) -> Vec2 {
        (self.max - self.min).max(Vec2::splat(f32::EPSILON))
    }
}

/// The nearest of `entity`'s ancestors that `holds_curve`: the entity holding the curve a plot
/// inside it shows.
pub(super) fn curve_owner(
    entity: Entity,
    parents: &Query<&ChildOf>,
    holds_curve: impl Fn(Entity) -> bool,
) -> Option<Entity> {
    parents
        .iter_ancestors(entity)
        .find(|&ancestor| holds_curve(ancestor))
}

/// On a plot node: the curve as a polyline, in logical pixels from the node's top left, of at most
/// [`MAX_PATH_POINTS`]. A kink is one point and a step two points at one x.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct CurvePath(pub Vec<Vec2>);

/// A node that plots the curve of its nearest ancestor holding a [`CurveValue`] and its
/// [`CurveConstraints`]. Its [`CurvePath`] follows the curve and the node's size.
#[derive(Component, Clone, Default)]
#[require(CurvePath, Node)]
pub struct CurvePlot {
    /// How far the plot keeps the curve from its edges, in logical pixels.
    pub inset: f32,
}

/// `curve` in `view`'s pixels: each segment halved over its own parameter until every piece lies
/// within a quarter pixel of the curve. A curve that would take more than [`MAX_PATH_POINTS`] is
/// drawn coarser, until it fits.
pub fn flatten(curve: &PreparedCurve, view: &CurveView) -> Vec<Vec2> {
    let mut flatness = FLATNESS;
    loop {
        let path = flatten_within(curve, view, flatness);
        // Coarser than the plot, each segment is one piece: a curve of more segments than the
        // budget is cut short.
        if path.len() <= MAX_PATH_POINTS || flatness > view.size.max_element() {
            return path.into_iter().take(MAX_PATH_POINTS).collect();
        }
        flatness *= 2.0;
    }
}

/// [`flatten`] at one `flatness`, in pixels.
fn flatten_within(curve: &PreparedCurve, view: &CurveView, flatness: f32) -> Vec<Vec2> {
    let mut path: Vec<Vec2> = Vec::new();
    for segment in curve.segments() {
        let pixel = |t: f32| view.to_pixels(segment.point(t));
        let start = pixel(0.0);
        if path.last() != Some(&start) {
            path.push(start);
        }
        subdivide(&pixel, flatness, 0.0, 1.0, start, pixel(1.0), 0, &mut path);
    }
    path
}

/// Pushes the piece of a segment from `t0` at `a` to `t1` at `b`, halved while it strays from the
/// chord, `a` already pushed.
fn subdivide(
    pixel: &impl Fn(f32) -> Vec2,
    flatness: f32,
    t0: f32,
    t1: f32,
    a: Vec2,
    b: Vec2,
    depth: u32,
    path: &mut Vec<Vec2>,
) {
    let mid = (t0 + t1) * 0.5;
    let middle = pixel(mid);
    // The quarters too, since an S-shaped piece crosses its chord at the middle.
    let flat = [t0 + (t1 - t0) * 0.25, mid, t0 + (t1 - t0) * 0.75]
        .into_iter()
        .all(|t| distance_to_segment(pixel(t), a, b) <= flatness);
    if depth >= MAX_DEPTH || flat {
        path.push(b);
        return;
    }
    subdivide(pixel, flatness, t0, mid, a, middle, depth + 1, path);
    subdivide(pixel, flatness, mid, t1, middle, b, depth + 1, path);
}

fn distance_to_segment(point: Vec2, a: Vec2, b: Vec2) -> f32 {
    let along = b - a;
    let t = ((point - a).dot(along) / along.length_squared().max(f32::EPSILON)).clamp(0.0, 1.0);
    point.distance(a + along * t)
}

/// Refreshes each plot's path when its curve or the plot's size changes.
pub(super) fn update_curve_paths(
    mut plots: Query<(Entity, &CurvePlot, Ref<ComputedNode>, &mut CurvePath)>,
    parents: Query<&ChildOf>,
    curves: Query<(Ref<CurveValue>, Ref<CurveConstraints>)>,
) {
    for (entity, plot, node, mut path) in &mut plots {
        let owner = curve_owner(entity, &parents, |owner| curves.contains(owner));
        let Some((curve, constraints)) = owner.and_then(|owner| curves.get(owner).ok()) else {
            continue;
        };
        if !(node.is_changed() || curve.is_changed() || constraints.is_changed()) {
            continue;
        }
        let size = node.size() * node.inverse_scale_factor();
        let view = CurveView::new(&constraints, size, plot.inset);
        path.set_if_neq(CurvePath(flatten(&curve.0.prepare(), &view)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::model::{
        CurveExtrapolation, CurvePoint, CurveSegment, DirectionHandle, ScalarCurve,
    };

    fn view() -> CurveView {
        CurveView::new(&CurveConstraints::default(), Vec2::new(300.0, 170.0), 0.0)
    }

    fn curve(points: &[(f32, f32, CurveSegment)]) -> ScalarCurve {
        ScalarCurve {
            points: (points.iter())
                .map(|&(x, y, segment)| CurvePoint {
                    x,
                    y,
                    segment,
                    ..default()
                })
                .collect(),
            extrapolation: CurveExtrapolation::Hold,
        }
    }

    fn distance_to_path(point: Vec2, path: &[Vec2]) -> f32 {
        (path.windows(2))
            .map(|pair| distance_to_segment(point, pair[0], pair[1]))
            .fold(f32::INFINITY, f32::min)
    }

    #[test]
    fn every_piece_lies_within_a_quarter_pixel() {
        use CurveSegment::Cubic;
        let mut steep = curve(&[(0.0, 0.0, Cubic), (0.4, 1.0, Cubic), (1.0, 0.1, Cubic)]);
        steep.points[0].right = DirectionHandle::Free(Vec2::new(0.05, 2.0));
        let prepared = steep.prepare();
        let path = flatten(&prepared, &view());
        for segment in prepared.segments() {
            for i in 0..=1000 {
                let point = view().to_pixels(segment.point(i as f32 / 1000.0));
                assert!(distance_to_path(point, &path) <= FLATNESS + 1e-3, "{point}");
            }
        }
    }

    #[test]
    fn pixels_map_back_to_the_curve() {
        let view = CurveView::new(&CurveConstraints::default(), Vec2::new(300.0, 170.0), 8.0);
        let point = Vec2::new(0.3, 0.8);
        assert!(view.to_curve(view.to_pixels(point)).distance(point) < 1e-5);
        let moved = view.to_pixels(point) + Vec2::new(14.2, -7.7);
        let delta = view.delta_to_curve(Vec2::new(14.2, -7.7));
        assert!((view.to_curve(moved) - (point + delta)).length() < 1e-5);
    }

    #[test]
    fn a_path_too_long_for_the_budget_is_drawn_coarser() {
        let mut wavy = curve(&[]);
        wavy.points = (0..=60)
            .map(|i| CurvePoint {
                x: i as f32 / 60.0,
                y: (i % 2) as f32,
                ..default()
            })
            .collect();
        let big = CurveView::new(&CurveConstraints::default(), Vec2::splat(2000.0), 0.0);
        let prepared = wavy.prepare();
        assert!(flatten_within(&prepared, &big, FLATNESS).len() > MAX_PATH_POINTS);
        assert!(flatten(&prepared, &big).len() <= MAX_PATH_POINTS);
    }

    #[test]
    fn a_kink_is_one_point_and_a_step_two_at_one_x() {
        use CurveSegment::Linear;
        let kink = curve(&[(0.0, 0.0, Linear), (0.5, 1.0, Linear), (1.0, 0.0, Linear)]);
        assert_eq!(flatten(&kink.prepare(), &view()).len(), 3);
        let step = curve(&[
            (0.0, 0.0, Linear),
            (0.5, 0.0, Linear),
            (0.5, 1.0, Linear),
            (1.0, 1.0, Linear),
        ]);
        let path = flatten(&step.prepare(), &view());
        assert_eq!(path.len(), 4);
        assert_eq!(path[1].x, path[2].x);
        assert_ne!(path[1].y, path[2].y);
    }
}
