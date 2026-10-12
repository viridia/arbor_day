//! A turntable camera: it circles a vertical axis through its subject, tilts, and zooms toward the
//! height under the pointer. Framing the subject returns the tilt to its home elevation, so there
//! is one action to get back to a good view. Framing fits the subject once; the camera does not
//! follow it as it changes, so an edit shows at its true size. A drag or scroll moves along one
//! axis at a time, whichever it has recently moved along more, so a spin does not also tilt or
//! zoom.

use bevy::{
    camera::primitives::Aabb,
    input::mouse::MouseScrollUnit,
    picking::{
        events::{PointerDrag, PointerScroll},
        pointer::PointerButton,
    },
    camera::visibility::VisibilitySystems,
    prelude::*,
    transform::TransformSystems,
};

/// A wheel notch, in the pixels a trackpad reports for a similar gesture.
const LINE_PIXELS: f32 = 20.0;
const SPIN_PER_PIXEL: f32 = 0.005;
const ZOOM_PER_PIXEL: f32 = 0.0015;
/// How long a gesture's movement counts toward choosing its axis, in seconds: movement this old
/// weighs 1/e as much as movement now.
const AXIS_MEMORY: f64 = 0.15;
/// Room around the subject when framed, as a factor on its bounding sphere.
const FRAME_MARGIN: f32 = 1.1;
/// Tilt limits, so the camera neither looks up through the ground nor flips over the top.
const MIN_ELEVATION: f32 = 5.0 * std::f32::consts::PI / 180.0;
const MAX_ELEVATION: f32 = 85.0 * std::f32::consts::PI / 180.0;

pub struct TurntablePlugin;

impl Plugin for TurntablePlugin {
    fn build(&self, app: &mut App) {
        // Framing reads the subject's world transform and bounds, so it runs after both are
        // updated; otherwise a change to the subject reaches the camera a frame late and the view
        // jitters.
        app.add_systems(
            PostUpdate,
            (frame_subject, place_camera)
                .chain()
                .after(TransformSystems::Propagate)
                .after(VisibilitySystems::CalculateBounds)
                .before(VisibilitySystems::UpdateFrusta),
        );
    }
}

/// Places its camera on a circle around a vertical axis through `focus`.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
#[require(TurntableGesture)]
pub struct TurntableCamera {
    /// The point looked at, on the axis.
    pub focus: Vec3,
    /// Angle around the axis, in radians; zero puts the camera on the +Z side.
    pub azimuth: f32,
    /// Angle above the horizontal, in radians.
    pub elevation: f32,
    /// The elevation [`TurntableCamera::frame`] returns to.
    pub home_elevation: f32,
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    /// Set by [`TurntableCamera::frame`]: fit every [`TurntableSubject`] on the next update.
    pub frame_pending: bool,
    /// While set, the distance follows the viewport so the sphere last framed still fits.
    /// Zooming clears it; framing sets it. The subject changing does not refit it.
    pub keep_framed: bool,
    /// The radius of the sphere last framed, about `focus`.
    pub framed_radius: f32,
    /// The heights zooming may move `focus` between: the subject's.
    pub height_limits: Option<Vec2>,
}

impl Default for TurntableCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            azimuth: 0.0,
            elevation: std::f32::consts::FRAC_PI_4,
            home_elevation: std::f32::consts::FRAC_PI_4,
            distance: 5.0,
            min_distance: 0.25,
            max_distance: 100.0,
            frame_pending: true,
            keep_framed: false,
            framed_radius: 0.0,
            height_limits: None,
        }
    }
}

impl TurntableCamera {
    pub fn spin(&mut self, radians: f32) {
        self.azimuth = (self.azimuth + radians).rem_euclid(std::f32::consts::TAU);
    }

    /// Raises the camera by `radians`, within limits. A bounding sphere fits the same from any
    /// elevation, so tilting keeps the subject framed.
    pub fn tilt(&mut self, radians: f32) {
        self.elevation = (self.elevation + radians).clamp(MIN_ELEVATION, MAX_ELEVATION);
    }

    /// Scales the distance by `factor`, moving the focus toward `toward_height` by as much, so the
    /// height under the pointer stays under it.
    pub fn zoom(&mut self, factor: f32, toward_height: Option<f32>) {
        let distance = (self.distance * factor).clamp(self.min_distance, self.max_distance);
        if let Some(height) = toward_height {
            self.focus.y += (height - self.focus.y) * (1.0 - distance / self.distance);
            if let Some(limits) = self.height_limits {
                self.focus.y = self.focus.y.clamp(limits.x, limits.y);
            }
        }
        self.distance = distance;
        self.keep_framed = false;
    }

    /// Returns to the home elevation and fits the subject. The spin is left alone: which side to
    /// look from is the author's choice.
    pub fn frame(&mut self) {
        self.elevation = self.home_elevation;
        self.frame_pending = true;
    }

    /// The horizontal direction from the axis toward the camera.
    fn outward(&self) -> Vec3 {
        Vec3::new(self.azimuth.sin(), 0.0, self.azimuth.cos())
    }

    /// Where `ray` crosses the vertical plane through the axis that faces the camera: the height
    /// on the axis that the pointer is over.
    fn height_under(&self, ray: Ray3d) -> Option<f32> {
        let plane = InfinitePlane3d::new(self.outward());
        let distance = ray.intersect_plane(self.focus, plane)?;
        Some(ray.get_point(distance).y)
    }
}

/// Marks what a [`TurntableCamera`] frames.
#[derive(Component, Default, Clone)]
pub struct TurntableSubject;

/// Fits the subject when a frame is pending, and keeps the sphere it framed fitting the viewport
/// until the camera zooms. Writes only on a change, so `place_camera` runs only then.
fn frame_subject(
    mut cameras: Query<(&mut TurntableCamera, &Camera, &Projection)>,
    subjects: Query<(Option<&Aabb>, &GlobalTransform), With<TurntableSubject>>,
) {
    let bounds = world_bounds(&subjects);
    for (mut turntable, camera, projection) in &mut cameras {
        let limits = bounds.map(|(min, max)| Vec2::new(min.y, max.y));
        if turntable.height_limits != limits {
            turntable.height_limits = limits;
        }
        if turntable.frame_pending
            && let Some((min, max)) = bounds
        {
            turntable.frame_pending = false;
            turntable.keep_framed = true;
            turntable.focus = (min + max) * 0.5;
            turntable.framed_radius = (max - min).length() * 0.5;
        }
        if !turntable.keep_framed {
            continue;
        }
        let Some(viewport) = camera.logical_viewport_size() else {
            continue;
        };
        let Projection::Perspective(perspective) = projection else {
            continue;
        };
        // Fit the sphere to the narrower of the two fields of view.
        let half_vertical = perspective.fov * 0.5;
        let half_horizontal = (half_vertical.tan() * viewport.x / viewport.y).atan();
        let half_fov = half_vertical.min(half_horizontal);
        let distance = (turntable.framed_radius * FRAME_MARGIN / half_fov.sin())
            .clamp(turntable.min_distance, turntable.max_distance);
        if (turntable.distance - distance).abs() > 1e-4 {
            turntable.distance = distance;
        }
    }
}

/// The world bounds of every subject, or `None` while any has no bounds yet, so a frame never fits
/// part of the subject.
fn world_bounds(
    subjects: &Query<(Option<&Aabb>, &GlobalTransform), With<TurntableSubject>>,
) -> Option<(Vec3, Vec3)> {
    let mut bounds: Option<(Vec3, Vec3)> = None;
    for (aabb, transform) in subjects {
        let aabb = aabb?;
        let (center, half) = (Vec3::from(aabb.center), Vec3::from(aabb.half_extents));
        for corner in 0..8 {
            let sign = Vec3::new(
                if corner & 1 == 0 { -1.0 } else { 1.0 },
                if corner & 2 == 0 { -1.0 } else { 1.0 },
                if corner & 4 == 0 { -1.0 } else { 1.0 },
            );
            let point = transform.transform_point(center + half * sign);
            bounds = Some(match bounds {
                Some((min, max)) => (min.min(point), max.max(point)),
                None => (point, point),
            });
        }
    }
    bounds
}

/// Runs after transform propagation, so it sets the camera's `GlobalTransform` as well; the camera
/// must have no parent.
fn place_camera(
    mut cameras: Query<
        (&TurntableCamera, &mut Transform, &mut GlobalTransform),
        Changed<TurntableCamera>,
    >,
) {
    for (turntable, mut transform, mut global) in &mut cameras {
        let offset = Vec3::new(
            turntable.elevation.cos() * turntable.azimuth.sin(),
            turntable.elevation.sin(),
            turntable.elevation.cos() * turntable.azimuth.cos(),
        ) * turntable.distance;
        *transform = Transform::from_translation(turntable.focus + offset)
            .looking_at(turntable.focus, Vec3::Y);
        *global = GlobalTransform::from(*transform);
    }
}

/// Chooses the axis a gesture moves along: the one it has recently moved along more, each axis's
/// movement weighed down by its age ([`AXIS_MEMORY`]). A gesture moves at once, with no threshold
/// to cross, keeps to its axis through a brief wobble, and changes axis once it has moved along
/// the other for a while.
#[derive(Clone, Copy, Default, Debug)]
struct DominantAxis {
    /// Each axis's recent movement, in pixels, decayed by age.
    recent: Vec2,
    /// When the last movement came, in seconds.
    last: f64,
}

impl DominantAxis {
    /// Takes a gesture's movement at time `now` and returns the part to apply: along the
    /// dominant axis only.
    fn feed(&mut self, delta: Vec2, now: f64) -> Vec2 {
        let decay = (-(now - self.last).max(0.0) / AXIS_MEMORY).exp() as f32;
        self.recent = self.recent * decay + delta.abs();
        self.last = now;
        if self.recent.x >= self.recent.y {
            Vec2::new(delta.x, 0.0)
        } else {
            Vec2::new(0.0, delta.y)
        }
    }
}

/// A camera's drag and scroll gestures, each moving along one axis at a time.
#[derive(Component, Default)]
pub struct TurntableGesture {
    drag: DominantAxis,
    scroll: DominantAxis,
}

/// Observer for the view's node: dragging sideways spins the subject, as if dragging its near
/// side; dragging down raises the camera to look from above. A drag does one or the other.
pub fn orbit_on_drag(
    mut drag: On<PointerDrag>,
    mut cameras: Query<(&mut TurntableCamera, &mut TurntableGesture)>,
    time: Res<Time<Real>>,
) {
    if drag.button != PointerButton::Primary {
        return;
    }
    drag.propagate(false);
    let Ok((mut turntable, mut gesture)) = cameras.single_mut() else {
        return;
    };
    let delta = gesture.drag.feed(drag.delta, time.elapsed_secs_f64());
    if delta.x != 0.0 {
        turntable.spin(-delta.x * SPIN_PER_PIXEL);
    }
    if delta.y != 0.0 {
        turntable.tilt(delta.y * SPIN_PER_PIXEL);
    }
}

/// Observer for the view's node: horizontal scroll spins, vertical scroll zooms toward the height
/// under the pointer. A scroll does one or the other.
pub fn zoom_on_scroll(
    mut scroll: On<PointerScroll>,
    mut cameras: Query<(
        &mut TurntableCamera,
        &mut TurntableGesture,
        &Camera,
        &GlobalTransform,
    )>,
    time: Res<Time<Real>>,
) {
    scroll.propagate(false);
    let Ok((mut turntable, mut gesture, camera, camera_transform)) = cameras.single_mut() else {
        return;
    };
    let pixels = match scroll.unit {
        MouseScrollUnit::Line => LINE_PIXELS,
        MouseScrollUnit::Pixel => 1.0,
    };
    let delta = gesture
        .scroll
        .feed(Vec2::new(scroll.x, scroll.y) * pixels, time.elapsed_secs_f64());
    if delta.x != 0.0 {
        turntable.spin(-delta.x * SPIN_PER_PIXEL);
    }
    if delta.y != 0.0 {
        let height = camera
            .viewport_to_world(camera_transform, scroll.pointer.position)
            .ok()
            .and_then(|ray| turntable.height_under(ray));
        turntable.zoom((-delta.y * ZOOM_PER_PIXEL).exp(), height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Events 1/60 s apart, as a trackpad sends them.
    const FRAME: f64 = 1.0 / 60.0;

    #[test]
    fn moves_on_the_first_event() {
        let mut axis = DominantAxis::default();
        assert_eq!(axis.feed(Vec2::new(-3.0, 1.0), 0.0), Vec2::new(-3.0, 0.0));
    }

    #[test]
    fn keeps_its_axis_through_a_wobble() {
        let mut axis = DominantAxis::default();
        for i in 0..20 {
            axis.feed(Vec2::new(5.0, 0.0), i as f64 * FRAME);
        }
        assert_eq!(axis.feed(Vec2::new(1.0, 4.0), 20.0 * FRAME), Vec2::new(1.0, 0.0));
    }

    #[test]
    fn changes_axis_when_the_other_persists() {
        let mut axis = DominantAxis::default();
        for i in 0..20 {
            axis.feed(Vec2::new(5.0, 0.0), i as f64 * FRAME);
        }
        let turned = (20..40)
            .map(|i| axis.feed(Vec2::new(0.0, 5.0), i as f64 * FRAME))
            .position(|delta| delta.y != 0.0);
        // Within a fraction of a second.
        assert!(turned.is_some_and(|events| events < 15), "{turned:?}");
    }

    #[test]
    fn a_pause_frees_the_axis() {
        let mut axis = DominantAxis::default();
        for i in 0..20 {
            axis.feed(Vec2::new(5.0, 0.0), i as f64 * FRAME);
        }
        assert_eq!(axis.feed(Vec2::new(0.0, 2.0), 2.0), Vec2::new(0.0, 2.0));
    }
}
