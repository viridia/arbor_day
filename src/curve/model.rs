//! A scalar curve: the points and segments an author edits, the constraints a consumer puts on
//! them, sampling, and applying an edit.

use std::ops::RangeInclusive;

use bevy::{
    curve::{Curve, Interval},
    prelude::*,
};

/// Iterations of the solve for t on a weighted segment: each halves the bracket at least.
const SOLVE_STEPS: u32 = 16;

/// A number that changes along another, drawn as a curve: y over x, through points ordered by x.
/// Two points may share an x, making a step.
#[derive(Reflect, Clone, Debug, PartialEq, Default)]
pub struct ScalarCurve {
    pub points: Vec<CurvePoint>,
    pub extrapolation: CurveExtrapolation,
}

/// A point of a [`ScalarCurve`], and how the curve leaves and enters it.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Default)]
pub struct CurvePoint {
    pub x: f32,
    pub y: f32,
    /// The segment from this point to the next; ignored on the last point.
    pub segment: CurveSegment,
    /// The handle toward the previous point.
    pub left: DirectionHandle,
    /// The handle toward the next point.
    pub right: DirectionHandle,
}

/// How a segment runs from its point to the next.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CurveSegment {
    /// The point's y until the next point, which steps to its own.
    Constant,
    /// A straight line to the next point.
    Linear,
    /// A smooth curve to the next point, shaped by the two points' direction handles.
    #[default]
    Cubic,
}

/// The line from a point to the control point that sets a cubic segment's direction and pull
/// there, as an offset from the point in the curve's units.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Default)]
pub enum DirectionHandle {
    /// Computed from the neighbouring points, so that the curve never overshoots them.
    #[default]
    Auto,
    /// Set by hand. Moving it turns the point's other handle, if also aligned, to stay on the same
    /// line, so the curve stays smooth through the point.
    Aligned(Vec2),
    /// Set by hand, independent of the point's other handle, so the curve can turn sharply.
    Free(Vec2),
}

/// A [`DirectionHandle`]'s kind, without its offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandleMode {
    /// [`DirectionHandle::Auto`].
    Auto,
    /// [`DirectionHandle::Aligned`].
    Aligned,
    /// [`DirectionHandle::Free`].
    Free,
}

impl DirectionHandle {
    /// The handle's kind.
    pub fn mode(&self) -> HandleMode {
        match self {
            Self::Auto => HandleMode::Auto,
            Self::Aligned(_) => HandleMode::Aligned,
            Self::Free(_) => HandleMode::Free,
        }
    }

    fn with_offset(mode: HandleMode, offset: Vec2) -> Self {
        match mode {
            HandleMode::Auto => Self::Auto,
            HandleMode::Aligned => Self::Aligned(offset),
            HandleMode::Free => Self::Free(offset),
        }
    }
}

/// Which of a point's handles: toward the previous point, or the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandleSide {
    /// Toward the previous point: [`CurvePoint::left`].
    Left,
    /// Toward the next point: [`CurvePoint::right`].
    Right,
}

/// What the curve gives past its first and last points.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CurveExtrapolation {
    /// Keeps the end point's value.
    #[default]
    Hold,
    /// Continues along the curve's slope at the end.
    Continue,
    /// Repeats the span between the ends.
    Repeat,
}

impl ScalarCurve {
    /// The curve resolved into one cubic Bézier per segment, for sampling. Points out of order are
    /// taken at the greatest x before them, and handles kept within their segments, so a curve
    /// read from a file cannot break it.
    pub fn prepare(&self) -> PreparedCurve {
        let mut points = self.points.clone();
        for index in 1..points.len() {
            points[index].x = points[index].x.max(points[index - 1].x);
        }
        let slopes = auto_slopes(&points);
        let mut segments = Vec::with_capacity(points.len().saturating_sub(1));
        for (index, pair) in points.windows(2).enumerate() {
            let (a, b) = (pair[0], pair[1]);
            let span = b.x - a.x;
            if span <= 0.0 {
                continue;
            }
            let (start, end) = (Vec2::new(a.x, a.y), Vec2::new(b.x, b.y));
            let third = Vec2::new(span / 3.0, 0.0);
            let controls = match a.segment {
                CurveSegment::Constant => {
                    let level = Vec2::new(b.x, a.y);
                    [start, start + third, start + third * 2.0, level]
                }
                CurveSegment::Linear => [
                    start,
                    start.lerp(end, 1.0 / 3.0),
                    start.lerp(end, 2.0 / 3.0),
                    end,
                ],
                CurveSegment::Cubic => {
                    let out = handle_offset(a.right, HandleSide::Right, slopes[index], span);
                    let into = handle_offset(b.left, HandleSide::Left, slopes[index + 1], span);
                    [start, start + out, end + into, end]
                }
            };
            segments.push(Bezier::new(controls));
        }
        let at = |point: Option<&CurvePoint>| point.map_or(Vec2::ZERO, |p| Vec2::new(p.x, p.y));
        PreparedCurve {
            segments,
            first: at(points.first()),
            last: at(points.last()),
            extrapolation: self.extrapolation,
        }
    }

    /// Applies `op` within `constraints`, and reports what changed. Points are kept within
    /// clamped axes, between their neighbours in x, and at pinned ends. An insert past the most
    /// points, a delete below the fewest, and a segment kind or handle mode the constraints leave
    /// out are refused. An inserted point lands on the curve at the given x. Aligned and free
    /// handles stay within their segments, so the curve stays a function of x, and their control
    /// points within a clamped y axis, so the curve does too.
    ///
    /// A consumer with rules of its own can judge the editor's edits itself instead.
    pub fn apply(&mut self, op: &CurveEditOp, constraints: &CurveConstraints) -> Applied {
        let before = self.clone();
        let inserted = self.apply_op(op, constraints);
        self.clamp_handles(&constraints.y);
        Applied {
            changed: *self != before,
            inserted,
        }
    }

    /// The offset of point `index`'s handle on `side`, if it is set by hand (aligned or free) and
    /// shapes a cubic segment: the handles an editor shows.
    pub fn manual_handle(&self, index: usize, side: HandleSide) -> Option<Vec2> {
        let span = self.span(index, side)?;
        let point = self.points[index];
        let (handle, segment) = match side {
            HandleSide::Left => (point.left, self.points[index - 1].segment),
            HandleSide::Right => (point.right, point.segment),
        };
        if segment != CurveSegment::Cubic || handle.mode() == HandleMode::Auto {
            return None;
        }
        Some(handle_offset(handle, side, 0.0, span))
    }

    /// `apply`'s edit, the clamps aside; returns the new point's index for an insert.
    fn apply_op(&mut self, op: &CurveEditOp, constraints: &CurveConstraints) -> Option<usize> {
        let last = self.points.len().checked_sub(1)?;
        let pinned =
            |index: usize| constraints.pin_ends == PinEnds::XY && (index == 0 || index == last);
        match op {
            CurveEditOp::Move(moves) => {
                let mut moved: Vec<usize> = Vec::with_capacity(moves.len());
                for &(index, at) in moves {
                    if index <= last && !pinned(index) {
                        self.points[index].x = at.x;
                        self.points[index].y = at.y;
                        moved.push(index);
                    }
                }
                moved.sort_unstable();
                for index in moved {
                    self.settle(index, constraints);
                }
            }
            CurveEditOp::SetPoint(index, at) => {
                let index = *index;
                if index <= last && !pinned(index) {
                    self.points[index].x = at.x;
                    self.points[index].y = at.y;
                    self.settle(index, constraints);
                }
            }
            CurveEditOp::MoveHandle(index, side, offset) => {
                self.move_handle(*index, *side, *offset, constraints);
            }
            CurveEditOp::Insert(at) => return self.insert(*at, constraints),
            CurveEditOp::Delete(indices) => {
                let mut indices: Vec<usize> =
                    indices.iter().copied().filter(|&i| i <= last).collect();
                indices.sort_unstable();
                indices.dedup();
                let pinned_end = constraints.pin_ends != PinEnds::None
                    && indices.iter().any(|&i| i == 0 || i == last);
                let left = self.points.len() - indices.len();
                if pinned_end || left < *constraints.points.start() {
                    return None;
                }
                for &index in indices.iter().rev() {
                    self.points.remove(index);
                }
            }
            CurveEditOp::SetSegment(index, segment) => {
                if constraints.segments.contains(segment)
                    && let Some(point) = self.points.get_mut(*index)
                {
                    point.segment = *segment;
                }
            }
            CurveEditOp::SetHandleMode(index, side, mode) => {
                if constraints.handles.contains(mode)
                    && let Some(offset) = self.handle_offset(*index, *side)
                {
                    *self.handle_mut(*index, *side) = DirectionHandle::with_offset(*mode, offset);
                }
            }
            CurveEditOp::Replace(curve) => {
                let allowed = constraints.points.contains(&curve.points.len())
                    && constraints.extrapolation.contains(&curve.extrapolation)
                    && curve.points.iter().all(|point| {
                        constraints.segments.contains(&point.segment)
                            && constraints.handles.contains(&point.left.mode())
                            && constraints.handles.contains(&point.right.mode())
                    });
                if allowed {
                    *self = curve.clone();
                    for index in 0..self.points.len() {
                        self.settle(index, constraints);
                    }
                }
            }
        }
        None
    }

    /// Point `index` clamped to the clamped axes, the pins, and its neighbours' x.
    fn settle(&mut self, index: usize, constraints: &CurveConstraints) {
        let last = self.points.len() - 1;
        let mut x = constraints.x.clamp(self.points[index].x);
        if index > 0 {
            x = x.max(self.points[index - 1].x);
        }
        if index < last {
            x = x.min(self.points[index + 1].x);
        }
        if constraints.pin_ends != PinEnds::None {
            if index == 0 {
                x = constraints.x.min;
            } else if index == last {
                x = constraints.x.max;
            }
        }
        let point = &mut self.points[index];
        point.x = x;
        point.y = constraints.y.clamp(point.y);
    }

    /// Sets a handle by hand: an auto one becomes aligned, or free if the constraints leave out
    /// aligned. An aligned handle turns its opposite, if also aligned, onto its line.
    fn move_handle(
        &mut self,
        index: usize,
        side: HandleSide,
        offset: Vec2,
        constraints: &CurveConstraints,
    ) {
        if index >= self.points.len() || self.handle_offset(index, side).is_none() {
            return;
        }
        let allows = |mode| constraints.handles.contains(&mode);
        let mode = match self.handle_mut(index, side).mode() {
            HandleMode::Auto if allows(HandleMode::Aligned) => HandleMode::Aligned,
            HandleMode::Auto if allows(HandleMode::Free) => HandleMode::Free,
            HandleMode::Auto => return,
            mode => mode,
        };
        let offset = self.clamp_offset(index, side, offset);
        *self.handle_mut(index, side) = DirectionHandle::with_offset(mode, offset);
        let opposite = match side {
            HandleSide::Left => HandleSide::Right,
            HandleSide::Right => HandleSide::Left,
        };
        if mode == HandleMode::Aligned
            && let DirectionHandle::Aligned(other) = *self.handle_mut(index, opposite)
            && let Some(direction) = offset.try_normalize()
        {
            let turned = -direction * other.length();
            let turned = self.clamp_offset(index, opposite, turned);
            *self.handle_mut(index, opposite) = DirectionHandle::Aligned(turned);
        }
    }

    /// Inserts a point on the curve at `at`'s x, clamped as a moved point is, after the points at
    /// or before that x, each coordinate rounded to its axis's precision, as a dragged point's is.
    /// Returns its index.
    fn insert(&mut self, at: Vec2, constraints: &CurveConstraints) -> Option<usize> {
        if self.points.len() >= *constraints.points.end() {
            return None;
        }
        let mut x = constraints.x.clamp(constraints.x.round(at.x));
        if constraints.pin_ends != PinEnds::None
            && let (Some(first), Some(last)) = (self.points.first(), self.points.last())
        {
            x = x.clamp(first.x, last.x);
        }
        let y = constraints
            .y
            .clamp(constraints.y.round(self.prepare().value(x)));
        let index = self.points.partition_point(|point| point.x <= x);
        let segment = (index.checked_sub(1))
            .and_then(|before| self.points.get(before))
            .map(|before| before.segment)
            .filter(|segment| constraints.segments.contains(segment))
            .or_else(|| constraints.segments.first().copied())
            .unwrap_or_default();
        self.points.insert(
            index,
            CurvePoint {
                x,
                y,
                segment,
                ..default()
            },
        );
        Some(index)
    }

    fn handle_mut(&mut self, index: usize, side: HandleSide) -> &mut DirectionHandle {
        let point = &mut self.points[index];
        match side {
            HandleSide::Left => &mut point.left,
            HandleSide::Right => &mut point.right,
        }
    }

    /// The span from point `index` to its neighbour on `side`, if it has one.
    fn span(&self, index: usize, side: HandleSide) -> Option<f32> {
        let point = self.points.get(index)?;
        let neighbour = match side {
            HandleSide::Left => self.points.get(index.checked_sub(1)?)?,
            HandleSide::Right => self.points.get(index + 1)?,
        };
        Some((neighbour.x - point.x).abs())
    }

    /// The offset point `index`'s handle on `side` has, computed if auto; none on a side with no
    /// neighbour.
    fn handle_offset(&self, index: usize, side: HandleSide) -> Option<Vec2> {
        let span = self.span(index, side)?;
        let point = self.points[index];
        let handle = match side {
            HandleSide::Left => point.left,
            HandleSide::Right => point.right,
        };
        Some(handle_offset(
            handle,
            side,
            auto_slopes(&self.points)[index],
            span,
        ))
    }

    /// `offset` kept within the span to the neighbour on `side`.
    fn clamp_offset(&self, index: usize, side: HandleSide, offset: Vec2) -> Vec2 {
        let span = self.span(index, side).unwrap_or(0.0);
        let x = match side {
            HandleSide::Left => offset.x.clamp(-span, 0.0),
            HandleSide::Right => offset.x.clamp(0.0, span),
        };
        Vec2::new(x, offset.y)
    }

    /// Every aligned or free handle kept within its segment and, if `y` is clamped, its control
    /// point within `y`'s bounds.
    fn clamp_handles(&mut self, y: &CurveAxis) {
        for index in 0..self.points.len() {
            let point_y = self.points[index].y;
            for side in [HandleSide::Left, HandleSide::Right] {
                let handle = *self.handle_mut(index, side);
                if let DirectionHandle::Aligned(offset) | DirectionHandle::Free(offset) = handle {
                    let mut offset = self.clamp_offset(index, side, offset);
                    // A segment lies within its control points' hull, so this bounds the curve.
                    offset.y = y.clamp(point_y + offset.y) - point_y;
                    *self.handle_mut(index, side) =
                        DirectionHandle::with_offset(handle.mode(), offset);
                }
            }
        }
    }
}

/// `handle`'s offset on `side` of a point whose auto slope is `slope`, over a segment `span` wide,
/// its x kept within the segment. An auto handle reaches a third of the span.
fn handle_offset(handle: DirectionHandle, side: HandleSide, slope: f32, span: f32) -> Vec2 {
    let sign = match side {
        HandleSide::Left => -1.0,
        HandleSide::Right => 1.0,
    };
    let offset = match handle {
        DirectionHandle::Auto => Vec2::new(span, slope * span) * (sign / 3.0),
        DirectionHandle::Aligned(offset) | DirectionHandle::Free(offset) => offset,
    };
    let x = match side {
        HandleSide::Left => offset.x.clamp(-span, 0.0),
        HandleSide::Right => offset.x.clamp(0.0, span),
    };
    Vec2::new(x, offset.y)
}

/// Each point's slope for auto handles, by Fritsch and Carlson's monotone interpolation: the
/// mean of the secants either side, zero where the curve turns, limited to three times either
/// secant; at an end, the one-sided three-point slope, limited the same way. A step (two points at
/// one x) ends the run of secants on each side of it.
fn auto_slopes(points: &[CurvePoint]) -> Vec<f32> {
    // The secant and span of each segment, none where it has no width.
    let secants: Vec<Option<(f32, f32)>> = (points.windows(2))
        .map(|pair| {
            let span = pair[1].x - pair[0].x;
            (span > 0.0).then(|| ((pair[1].y - pair[0].y) / span, span))
        })
        .collect();
    let secant = |index: Option<usize>| index.and_then(|i| secants.get(i).copied().flatten());
    (0..points.len())
        .map(|index| {
            let before = secant(index.checked_sub(1));
            let after = secant(Some(index));
            match (before, after) {
                (Some((left, _)), Some((right, _))) => {
                    if left * right <= 0.0 {
                        0.0
                    } else {
                        let limit = 3.0 * left.abs().min(right.abs());
                        ((left + right) * 0.5).clamp(-limit, limit)
                    }
                }
                (None, Some(near)) => end_slope(near, secant(Some(index + 1))),
                (Some(near), None) => end_slope(near, secant(index.checked_sub(2))),
                (None, None) => 0.0,
            }
        })
        .collect()
}

/// The slope at an end whose segment has secant and span `near`, and `far` those of the segment
/// after it: the one-sided three-point slope, zero if it points against the near secant, and at
/// most three times it.
fn end_slope(near: (f32, f32), far: Option<(f32, f32)>) -> f32 {
    let (secant, span) = near;
    let Some((far_secant, far_span)) = far else {
        return secant;
    };
    let slope = ((2.0 * span + far_span) * secant - span * far_secant) / (span + far_span);
    if slope * secant <= 0.0 {
        0.0
    } else {
        let limit = 3.0 * secant.abs();
        slope.clamp(-limit, limit)
    }
}

/// A cubic Bézier segment, its four control points in the curve's units.
#[derive(Clone, Copy, Debug)]
pub struct Bezier {
    pub controls: [Vec2; 4],
    /// Its inner controls' x not at thirds of its span, so its y is found by solving for t.
    weighted: bool,
}

impl Bezier {
    fn new(controls: [Vec2; 4]) -> Self {
        let [p0, p1, p2, p3] = controls;
        let span = p3.x - p0.x;
        let tolerance = span * 1e-6;
        let weighted = (p1.x - (p0.x + span / 3.0)).abs() > tolerance
            || (p2.x - (p0.x + span * 2.0 / 3.0)).abs() > tolerance;
        Self { controls, weighted }
    }

    /// The point at parameter `t`.
    pub fn point(&self, t: f32) -> Vec2 {
        let [p0, p1, p2, p3] = self.controls;
        Vec2::new(
            bernstein(t, p0.x, p1.x, p2.x, p3.x),
            bernstein(t, p0.y, p1.y, p2.y, p3.y),
        )
    }

    /// The value and slope at `x`, within the segment's span.
    fn sample(&self, x: f32) -> (f32, f32) {
        let [p0, p1, p2, p3] = self.controls;
        let span = p3.x - p0.x;
        let (t, dx) = if self.weighted {
            let t = self.solve(x);
            (t, derivative(t, p0.x, p1.x, p2.x, p3.x))
        } else {
            (((x - p0.x) / span).clamp(0.0, 1.0), span)
        };
        let y = bernstein(t, p0.y, p1.y, p2.y, p3.y);
        let dy = derivative(t, p0.y, p1.y, p2.y, p3.y);
        (y, dy / dx.max(span * f32::EPSILON))
    }

    /// The t at which the segment reaches `x`: Newton's method, falling back to bisection when a
    /// step would leave the bracket, a fixed number of iterations.
    fn solve(&self, x: f32) -> f32 {
        let [p0, p1, p2, p3] = self.controls;
        let (mut low, mut high) = (0.0, 1.0);
        let mut t = ((x - p0.x) / (p3.x - p0.x)).clamp(0.0, 1.0);
        for _ in 0..SOLVE_STEPS {
            let error = bernstein(t, p0.x, p1.x, p2.x, p3.x) - x;
            if error == 0.0 {
                break;
            }
            if error > 0.0 {
                high = t;
            } else {
                low = t;
            }
            let slope = derivative(t, p0.x, p1.x, p2.x, p3.x);
            let next = t - error / slope;
            t = if slope > 0.0 && next > low && next < high {
                next
            } else {
                (low + high) * 0.5
            };
        }
        t
    }
}

/// The cubic with Bézier controls `a` to `d`, at `t`.
fn bernstein(t: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    // In powers of t, from differences of the controls, so equal controls give exactly their
    // value: a flat curve scales by exactly 1.
    let [c1, c2, c3] = power_coefficients(a, b, c, d);
    a + t * (c1 + t * (c2 + t * c3))
}

/// The derivative in t of [`bernstein`].
fn derivative(t: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    let [c1, c2, c3] = power_coefficients(a, b, c, d);
    c1 + t * (2.0 * c2 + t * 3.0 * c3)
}

/// The coefficients of t, t² and t³ of the cubic with Bézier controls `a` to `d`.
fn power_coefficients(a: f32, b: f32, c: f32, d: f32) -> [f32; 3] {
    [
        3.0 * (b - a),
        3.0 * ((c - b) - (b - a)),
        (d - a) - 3.0 * (c - b),
    ]
}

/// A [`ScalarCurve`] resolved for sampling, from [`ScalarCurve::prepare`]: one cubic Bézier per
/// segment. Sample it anywhere: past the ends it extrapolates, and at a step it takes the later
/// point's value. Prepare once and sample many times; preparing allocates.
#[derive(Clone, Debug)]
pub struct PreparedCurve {
    segments: Vec<Bezier>,
    first: Vec2,
    last: Vec2,
    extrapolation: CurveExtrapolation,
}

impl PreparedCurve {
    /// The segments in order of x, without the zero-width ones at steps.
    pub fn segments(&self) -> &[Bezier] {
        &self.segments
    }

    /// The curve's value at `x`.
    pub fn value(&self, x: f32) -> f32 {
        self.value_and_slope(x).0
    }

    /// The curve's slope at `x`, dy over dx.
    pub fn slope(&self, x: f32) -> f32 {
        self.value_and_slope(x).1
    }

    fn value_and_slope(&self, x: f32) -> (f32, f32) {
        let (first, last) = (self.first, self.last);
        if x >= first.x && x <= last.x {
            return self.within(x);
        }
        match self.extrapolation {
            CurveExtrapolation::Hold if x < first.x => (first.y, 0.0),
            CurveExtrapolation::Hold => (last.y, 0.0),
            CurveExtrapolation::Continue => {
                let end = if x < first.x { first } else { last };
                let slope = self.within(end.x).1;
                (end.y + slope * (x - end.x), slope)
            }
            CurveExtrapolation::Repeat => {
                let span = last.x - first.x;
                if span <= 0.0 {
                    return (if x < first.x { first.y } else { last.y }, 0.0);
                }
                let turns = ((x - first.x) / span).floor();
                self.within(x - turns * span)
            }
        }
    }

    /// The value and slope at `x`, between the ends.
    fn within(&self, x: f32) -> (f32, f32) {
        let after = self
            .segments
            .partition_point(|segment| segment.controls[0].x <= x);
        match after
            .checked_sub(1)
            .and_then(|index| self.segments.get(index))
        {
            Some(segment) => segment.sample(x),
            None => match self.segments.first() {
                Some(segment) => segment.sample(x),
                // Points at one x alone: a step to the last.
                None => (self.last.y, 0.0),
            },
        }
    }
}

impl Curve<f32> for PreparedCurve {
    fn domain(&self) -> Interval {
        Interval::EVERYWHERE
    }

    fn sample_unchecked(&self, t: f32) -> f32 {
        self.value(t)
    }
}

/// The consumer's declaration of what its curve may be: the axes' bounds, the kinds of segment
/// and handle, how many points, and which ends are fixed.
#[derive(Component, Clone, Debug)]
pub struct CurveConstraints {
    pub x: CurveAxis,
    pub y: CurveAxis,
    /// The segment kinds a point may take.
    pub segments: Vec<CurveSegment>,
    /// The handle modes a handle may take.
    pub handles: Vec<HandleMode>,
    /// How many points the curve may have.
    pub points: RangeInclusive<usize>,
    pub pin_ends: PinEnds,
    /// The extrapolations the curve may take.
    pub extrapolation: Vec<CurveExtrapolation>,
}

impl Default for CurveConstraints {
    /// Both axes from 0 to 1 and clamped, every kind and mode, two to 64 points, the ends pinned in
    /// x, and every extrapolation.
    fn default() -> Self {
        Self {
            x: CurveAxis::unit(),
            y: CurveAxis::unit(),
            segments: vec![
                CurveSegment::Cubic,
                CurveSegment::Linear,
                CurveSegment::Constant,
            ],
            handles: vec![HandleMode::Auto, HandleMode::Aligned, HandleMode::Free],
            points: 2..=64,
            pin_ends: PinEnds::X,
            extrapolation: vec![
                CurveExtrapolation::Hold,
                CurveExtrapolation::Continue,
                CurveExtrapolation::Repeat,
            ],
        }
    }
}

/// One axis's bounds, step and precision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurveAxis {
    pub min: f32,
    pub max: f32,
    /// Whether points are kept within the bounds. When false, the bounds only frame the view.
    pub clamped: bool,
    /// The increment for snapping, arrow keys and grid lines.
    pub step: f32,
    /// The finest increment a dragged point lands on, and the inputs show; 0 for any value.
    pub precision: f32,
}

impl CurveAxis {
    /// From 0 to 1, clamped, in steps of 0.05, to hundredths.
    pub fn unit() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            clamped: true,
            step: 0.05,
            precision: 0.01,
        }
    }

    /// `value` to the nearest multiple of the precision. A fine precision divides by its count per
    /// unit rather than multiplying by itself, so 0.35 lands on the `f32` nearest 0.35, which
    /// prints as `0.35`, not one step off it, which prints as `0.35000002`.
    pub fn round(&self, value: f32) -> f32 {
        if self.precision <= 0.0 {
            return value;
        }
        let per_unit = (1.0 / self.precision).round();
        if self.precision < 1.0 && per_unit * self.precision == 1.0 {
            (value * per_unit).round() / per_unit
        } else {
            (value / self.precision).round() * self.precision
        }
    }

    /// `value` within the bounds if they are clamped.
    pub fn clamp(&self, value: f32) -> f32 {
        if self.clamped {
            value.clamp(self.min, self.max)
        } else {
            value
        }
    }
}

/// Which of the first and last points' coordinates are fixed. With either pinned, the ends cannot
/// be deleted and an insert falls between them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinEnds {
    /// The ends move as any point does.
    None,
    /// The ends stay at the x axis's bounds and move only in y.
    X,
    /// The ends never move.
    XY,
}

/// An edit to one curve, its points by index.
#[derive(Clone, Debug, PartialEq)]
pub enum CurveEditOp {
    /// Moves points to proposed positions, as a drag does.
    Move(Vec<(usize, Vec2)>),
    /// Sets a handle's offset from its point, as a drag does.
    MoveHandle(usize, HandleSide, Vec2),
    /// Sets one point's position, as typed into an input.
    SetPoint(usize, Vec2),
    /// Adds a point on the curve at this position's x.
    Insert(Vec2),
    Delete(Vec<usize>),
    /// Sets the kind of the segment from a point to the next.
    SetSegment(usize, CurveSegment),
    /// Sets a handle's mode, keeping its current offset.
    SetHandleMode(usize, HandleSide, HandleMode),
    /// Replaces the whole curve: a preset, a reset or a paste.
    Replace(ScalarCurve),
}

/// What [`ScalarCurve::apply`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Applied {
    /// False if the edit was refused or left the curve as it was.
    pub changed: bool,
    /// The inserted point's index.
    pub inserted: Option<usize>,
}

/// A copy of the curve an editor or thumbnail shows. The consumer writes it; the editor never
/// changes it.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct CurveValue(pub ScalarCurve);

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f32, y: f32) -> CurvePoint {
        CurvePoint { x, y, ..default() }
    }

    fn curve(points: &[(f32, f32)]) -> ScalarCurve {
        ScalarCurve {
            points: points.iter().map(|&(x, y)| point(x, y)).collect(),
            extrapolation: CurveExtrapolation::Hold,
        }
    }

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn two_auto_points_are_a_straight_line() {
        let line = curve(&[(0.0, 1.6), (1.0, 0.4)]).prepare();
        for i in 0..=20 {
            let x = i as f32 / 20.0;
            assert!(near(line.value(x), 1.6 - 1.2 * x), "{x}");
            assert!(near(line.slope(x), -1.2), "{x}");
        }
    }

    #[test]
    fn auto_handles_never_overshoot() {
        let zigzag = curve(&[
            (0.0, 0.0),
            (0.1, 1.0),
            (0.15, 0.2),
            (0.5, 0.25),
            (0.6, 0.9),
            (0.62, 0.9),
            (1.0, 0.0),
        ]);
        let prepared = zigzag.prepare();
        for pair in zigzag.points.windows(2) {
            let (low, high) = (pair[0].y.min(pair[1].y), pair[0].y.max(pair[1].y));
            for i in 0..=100 {
                let x = pair[0].x + (pair[1].x - pair[0].x) * i as f32 / 100.0;
                let y = prepared.value(x);
                assert!(y >= low - 1e-5 && y <= high + 1e-5, "{x}: {y}");
            }
        }
    }

    #[test]
    fn a_step_is_right_continuous() {
        let mut step = curve(&[(0.0, 0.0), (0.5, 0.0), (0.5, 1.0), (1.0, 1.0)]);
        step.points[0].segment = CurveSegment::Linear;
        step.points[2].segment = CurveSegment::Linear;
        let prepared = step.prepare();
        assert_eq!(prepared.segments().len(), 2);
        assert!(near(prepared.value(0.4999), 0.0));
        assert!(near(prepared.value(0.5), 1.0));
    }

    #[test]
    fn a_constant_segment_holds_until_the_next_point() {
        let mut steps = curve(&[(0.0, 0.2), (0.5, 0.8), (1.0, 0.4)]);
        steps.points[0].segment = CurveSegment::Constant;
        let prepared = steps.prepare();
        assert!(near(prepared.value(0.49), 0.2));
        assert!(near(prepared.value(0.5), 0.8));
    }

    #[test]
    fn extrapolations() {
        let mut line = curve(&[(0.0, 0.0), (1.0, 1.0)]);
        assert!(near(line.prepare().value(2.0), 1.0));
        assert!(near(line.prepare().value(-1.0), 0.0));
        line.extrapolation = CurveExtrapolation::Continue;
        assert!(near(line.prepare().value(2.0), 2.0));
        assert!(near(line.prepare().value(-0.5), -0.5));
        line.extrapolation = CurveExtrapolation::Repeat;
        assert!(near(line.prepare().value(1.25), 0.25));
        assert!(near(line.prepare().value(-0.25), 0.75));
    }

    #[test]
    fn a_weighted_and_an_unweighted_form_agree() {
        let prepared = curve(&[(0.0, 0.0), (0.3, 0.8), (1.0, 0.5)]).prepare();
        for segment in prepared.segments() {
            let weighted = Bezier {
                weighted: true,
                ..*segment
            };
            assert!(!segment.weighted);
            let [start, .., end] = segment.controls;
            for i in 0..=10 {
                let x = start.x + (end.x - start.x) * i as f32 / 10.0;
                let (a, b) = (segment.sample(x), weighted.sample(x));
                assert!(
                    near(a.0, b.0) && (a.1 - b.1).abs() < 1e-3,
                    "{x}: {a:?} {b:?}"
                );
            }
        }
    }

    #[test]
    fn a_free_handle_bends_the_curve_and_stays_a_function() {
        let mut bent = curve(&[(0.0, 0.0), (1.0, 0.0)]);
        // Reaching past the next point: kept within the segment.
        bent.points[0].right = DirectionHandle::Free(Vec2::new(2.0, 1.0));
        let prepared = bent.prepare();
        assert!(prepared.segments()[0].weighted);
        assert!(prepared.segments()[0].controls[1].x <= 1.0);
        assert!(prepared.value(0.3) > 0.0);
    }

    #[test]
    fn moves_are_clamped_to_the_axes_neighbours_and_pins() {
        let constraints = CurveConstraints::default();
        let mut moved = curve(&[(0.0, 0.5), (0.5, 0.5), (1.0, 0.5)]);
        let applied = moved.apply(
            &CurveEditOp::Move(vec![(1, Vec2::new(1.4, 1.2)), (0, Vec2::new(0.3, -1.0))]),
            &constraints,
        );
        assert!(applied.changed);
        let at = |c: &ScalarCurve, i: usize| Vec2::new(c.points[i].x, c.points[i].y);
        assert_eq!(at(&moved, 0), Vec2::new(0.0, 0.0));
        assert_eq!(at(&moved, 1), Vec2::new(1.0, 1.0));
        let pinned = CurveConstraints {
            pin_ends: PinEnds::XY,
            ..default()
        };
        let applied = moved.apply(&CurveEditOp::SetPoint(2, Vec2::new(1.0, 0.0)), &pinned);
        assert!(!applied.changed);
    }

    #[test]
    fn an_inserted_point_lands_on_each_axis_precision() {
        let mut inserted = curve(&[(0.0, 0.0), (1.0, 0.7)]);
        let applied = inserted.apply(
            &CurveEditOp::Insert(Vec2::new(0.4567, 0.9)),
            &CurveConstraints::default(),
        );
        assert_eq!(applied.inserted, Some(1));
        let point = inserted.points[1];
        assert_eq!(
            (point.x.to_string(), point.y.to_string()),
            ("0.46".into(), "0.32".into())
        );
    }

    #[test]
    fn a_rounded_value_prints_as_its_precision() {
        let axis = CurveAxis::unit();
        for hundredths in 0..=200 {
            let written = format!("{}", f64::from(hundredths) / 100.0);
            let near = hundredths as f32 * 0.01 + 0.003;
            assert_eq!(axis.round(near).to_string(), written);
        }
    }

    #[test]
    fn inserts_and_deletes_keep_to_the_counts() {
        let constraints = CurveConstraints {
            points: 2..=3,
            ..default()
        };
        let mut line = curve(&[(0.0, 0.0), (1.0, 1.0)]);
        let applied = line.apply(&CurveEditOp::Insert(Vec2::new(0.25, 0.9)), &constraints);
        assert_eq!(applied.inserted, Some(1));
        // On the curve, not at the pointer's y.
        assert!(near(line.points[1].y, 0.25));
        let applied = line.apply(&CurveEditOp::Insert(Vec2::new(0.5, 0.5)), &constraints);
        assert!(!applied.changed);
        assert!(!(line.apply(&CurveEditOp::Delete(vec![0]), &constraints)).changed);
        assert!(
            line.apply(&CurveEditOp::Delete(vec![1]), &constraints)
                .changed
        );
        assert!(!(line.apply(&CurveEditOp::Delete(vec![1]), &constraints)).changed);
    }

    #[test]
    fn kinds_the_constraints_leave_out_are_refused() {
        let constraints = CurveConstraints {
            segments: vec![CurveSegment::Cubic],
            handles: vec![HandleMode::Auto],
            ..default()
        };
        let mut line = curve(&[(0.0, 0.0), (1.0, 1.0)]);
        let segment = CurveEditOp::SetSegment(0, CurveSegment::Constant);
        assert!(!line.apply(&segment, &constraints).changed);
        let handle = CurveEditOp::MoveHandle(0, HandleSide::Right, Vec2::new(0.2, 0.5));
        assert!(!line.apply(&handle, &constraints).changed);
        assert!(line.apply(&segment, &default()).changed);
    }

    #[test]
    fn a_handle_keeps_the_curve_within_a_clamped_axis() {
        let mut curve = curve(&[(0.0, 0.9), (1.0, 0.9)]);
        let constraints = CurveConstraints::default();
        let lift = CurveEditOp::MoveHandle(0, HandleSide::Right, Vec2::new(0.3, 0.8));
        curve.apply(&lift, &constraints);
        let DirectionHandle::Aligned(offset) = curve.points[0].right else {
            panic!("{:?}", curve.points[0].right);
        };
        assert!(offset.abs_diff_eq(Vec2::new(0.3, 0.1), 1e-6), "{offset}");
        let prepared = curve.prepare();
        assert!((0..=100).all(|i| prepared.value(i as f32 / 100.0) <= 1.0 + 1e-6));
    }

    #[test]
    fn a_moved_auto_handle_becomes_aligned_and_turns_its_opposite() {
        let mut bent = curve(&[(0.0, 0.0), (0.5, 0.5), (1.0, 0.0)]);
        let constraints = CurveConstraints::default();
        let left = CurveEditOp::SetHandleMode(1, HandleSide::Left, HandleMode::Aligned);
        bent.apply(&left, &constraints);
        let right = CurveEditOp::MoveHandle(1, HandleSide::Right, Vec2::new(0.1, 0.1));
        bent.apply(&right, &constraints);
        let DirectionHandle::Aligned(left) = bent.points[1].left else {
            panic!("{:?}", bent.points[1].left);
        };
        assert!(left.normalize().dot(Vec2::new(-1.0, -1.0).normalize()) > 0.9999);
    }

    #[test]
    fn only_handles_set_by_hand_on_cubic_segments_are_shown() {
        let mut bent = curve(&[(0.0, 0.0), (0.5, 0.5), (1.0, 0.0)]);
        bent.points[1].left = DirectionHandle::Free(Vec2::new(-0.1, 0.2));
        bent.points[1].right = DirectionHandle::Aligned(Vec2::new(0.1, 0.0));
        bent.points[0].right = DirectionHandle::Free(Vec2::new(0.1, 0.0));
        assert_eq!(
            bent.manual_handle(1, HandleSide::Left),
            Some(Vec2::new(-0.1, 0.2))
        );
        assert_eq!(bent.manual_handle(0, HandleSide::Left), None);
        assert_eq!(bent.manual_handle(2, HandleSide::Left), None);
        bent.points[1].segment = CurveSegment::Linear;
        assert_eq!(bent.manual_handle(1, HandleSide::Right), None);
        assert!(bent.manual_handle(0, HandleSide::Right).is_some());
    }
}
