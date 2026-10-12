//! A turntable camera: it circles a vertical axis through its subject, tilts, and zooms toward the
//! height under the pointer. Framing the subject returns the tilt to its home elevation, so there
//! is one action to get back to a good view.

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
/// Room around the subject when framed, as a factor on its bounding sphere.
const FRAME_MARGIN: f32 = 1.1;
/// Tilt limits, so the camera neither looks up through the ground nor flips over the top.
const MIN_ELEVATION: f32 = 5.0 * std::f32::consts::PI / 180.0;
const MAX_ELEVATION: f32 = 85.0 * std::f32::consts::PI / 180.0;

pub struct TurntablePlugin;

impl Plugin for TurntablePlugin {
    fn build(&self, app: &mut App) {
        // Framing reads the subject's world transform, so it runs after propagation; otherwise a
        // change to the subject reaches the camera a frame late and the view jitters.
        app.add_systems(
            PostUpdate,
            (keep_subject_framed, place_camera)
                .chain()
                .after(TransformSystems::Propagate)
                .before(VisibilitySystems::UpdateFrusta),
        );
    }
}

/// Places its camera on a circle around a vertical axis through `focus`.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
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
    /// While set, the camera keeps every [`TurntableSubject`] in view as the subject or the
    /// viewport changes. Zooming clears it; [`TurntableCamera::frame`] sets it.
    pub keep_framed: bool,
    /// The heights zooming may move `focus` between: the subject's, once framed.
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
            keep_framed: true,
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

    /// Returns to the home elevation and keeps the subject framed. The spin is left alone: which
    /// side to look from is the author's choice.
    pub fn frame(&mut self) {
        self.elevation = self.home_elevation;
        self.keep_framed = true;
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

fn keep_subject_framed(
    mut cameras: Query<(&mut TurntableCamera, &Camera, &Projection)>,
    subjects: Query<(&Aabb, &GlobalTransform), With<TurntableSubject>>,
) {
    let Some((min, max)) = world_bounds(&subjects) else {
        return;
    };
    for (mut turntable, camera, projection) in &mut cameras {
        if !turntable.keep_framed {
            continue;
        }
        let Some(viewport) = camera.logical_viewport_size() else {
            continue;
        };
        let Projection::Perspective(perspective) = projection else {
            continue;
        };
        // Fit the bounding sphere to the narrower of the two fields of view.
        let half_vertical = perspective.fov * 0.5;
        let half_horizontal = (half_vertical.tan() * viewport.x / viewport.y).atan();
        let half_fov = half_vertical.min(half_horizontal);
        let radius = (max - min).length() * 0.5;
        let focus = (min + max) * 0.5;
        let distance = radius * FRAME_MARGIN / half_fov.sin();
        let limits = Some(Vec2::new(min.y, max.y));

        if turntable.focus.distance(focus) > 1e-4
            || (turntable.distance - distance).abs() > 1e-4
            || turntable.height_limits != limits
        {
            turntable.focus = focus;
            turntable.distance = distance.clamp(turntable.min_distance, turntable.max_distance);
            turntable.height_limits = limits;
        }
    }
}

fn world_bounds(
    subjects: &Query<(&Aabb, &GlobalTransform), With<TurntableSubject>>,
) -> Option<(Vec3, Vec3)> {
    let mut bounds: Option<(Vec3, Vec3)> = None;
    for (aabb, transform) in subjects {
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

/// Observer for the view's node: dragging sideways spins the subject, as if dragging its near
/// side; dragging down raises the camera to look from above.
pub fn orbit_on_drag(mut drag: On<PointerDrag>, mut cameras: Query<&mut TurntableCamera>) {
    if drag.button != PointerButton::Primary {
        return;
    }
    drag.propagate(false);
    if let Ok(mut turntable) = cameras.single_mut() {
        turntable.spin(-drag.delta.x * SPIN_PER_PIXEL);
        turntable.tilt(drag.delta.y * SPIN_PER_PIXEL);
    }
}

/// Observer for the view's node: horizontal scroll spins, vertical scroll zooms toward the height
/// under the pointer.
pub fn zoom_on_scroll(
    mut scroll: On<PointerScroll>,
    mut cameras: Query<(&mut TurntableCamera, &Camera, &GlobalTransform)>,
) {
    scroll.propagate(false);
    let Ok((mut turntable, camera, camera_transform)) = cameras.single_mut() else {
        return;
    };
    let pixels = match scroll.unit {
        MouseScrollUnit::Line => LINE_PIXELS,
        MouseScrollUnit::Pixel => 1.0,
    };
    if scroll.x != 0.0 {
        turntable.spin(-scroll.x * pixels * SPIN_PER_PIXEL);
    }
    if scroll.y != 0.0 {
        let height = camera
            .viewport_to_world(camera_transform, scroll.pointer.position)
            .ok()
            .and_then(|ray| turntable.height_under(ray));
        turntable.zoom((-scroll.y * pixels * ZOOM_PER_PIXEL).exp(), height);
    }
}
