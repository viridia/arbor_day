//! A sense of scale beside the plant, behind a HUD toggle: a measuring stick ticked and numbered
//! in metres, and a person's silhouette. Both stand just outside the crown on the
//! camera's right and turn to face the camera, so they stay beside the plant as the view spins.
//! Ticks and numbers are sized in pixels, so they read the same at any zoom.

use bevy::{camera::visibility::RenderLayers, light::NotShadowCaster, prelude::*};

use crate::{settings::ViewSettings, species::Species, turntable::TurntableCamera};

/// The render layer the stick is drawn on, which only the 3D camera sees: on the default layer the
/// UI camera would draw it too, a few pixels tall at the window's centre.
const SCALE_LAYER: usize = 1;

/// The stick's height, in metres.
const STICK_HEIGHT: f32 = 5.0;
/// Its lines and numbers: the envelope's pale green, stronger.
const STICK_COLOR: Color = Color::srgba(0.6, 0.85, 0.6, 0.7);
/// The silhouette's fill, opaque, so its overlapping parts do not show.
const FIGURE_COLOR: Color = Color::srgb(0.42, 0.52, 0.42);

/// Tick lengths, in pixels: at each metre, each half metre, and each tenth.
const TICK_LENGTHS: [f32; 3] = [12.0, 8.0, 5.0];
/// The least gap between ticks on screen, in pixels; closer ticks are left out.
const MIN_TICK_GAP: f32 = 6.0;
/// The numbers' size, in pixels. Ticks are numbered while they are twice this far apart.
const NUMBER_SIZE: f32 = 14.0;
/// The stick's gap from the crown, and the silhouette's from the stick, in pixels; the second
/// leaves room for the numbers.
const STICK_GAP: f32 = 12.0;
const FIGURE_GAP: f32 = 56.0;

pub struct ScalePlugin;

impl Plugin for ScalePlugin {
    fn build(&self, app: &mut App) {
        app.insert_gizmo_config(
            ScaleGizmos,
            GizmoConfig {
                render_layers: RenderLayers::layer(SCALE_LAYER),
                ..default()
            },
        )
        .add_observer(add_figure_mesh)
        .add_observer(see_scale_layer)
        .add_systems(Update, draw_scale);
    }
}

/// The stick's gizmos.
#[derive(Default, Reflect, GizmoConfigGroup)]
struct ScaleGizmos;

/// The 3D camera sees the stick's layer as well as the default one.
fn see_scale_layer(add: On<Add<TurntableCamera>>, mut commands: Commands) {
    commands
        .entity(add.event_target())
        .insert(RenderLayers::from_layers(&[0, SCALE_LAYER]));
}

/// The person's silhouette, a flat cutout facing the camera.
#[derive(Component, Default, Clone)]
pub struct ScaleFigure;

/// The silhouette's mesh, 1.75 m tall, in the XY plane with its feet at the origin, and its
/// material, unlit and drawn from both sides.
fn add_figure_mesh(
    add: On<Add<ScaleFigure>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let part = |shape: Mesh, x: f32, y: f32| shape.transformed_by(Transform::from_xyz(x, y, 0.0));
    let parts = [
        part(Circle::new(0.11).into(), 0.0, 1.64),
        part(Capsule2d::new(0.17, 0.4).into(), 0.0, 1.2),
        part(Capsule2d::new(0.05, 0.6).into(), -0.24, 1.12),
        part(Capsule2d::new(0.05, 0.6).into(), 0.24, 1.12),
        part(Capsule2d::new(0.07, 0.75).into(), -0.08, 0.45),
        part(Capsule2d::new(0.07, 0.75).into(), 0.08, 0.45),
    ];
    let mut mesh = parts[0].clone();
    for other in &parts[1..] {
        mesh.merge(other).expect("the parts share their attributes");
    }
    let material = StandardMaterial {
        base_color: FIGURE_COLOR,
        unlit: true,
        cull_mode: None,
        double_sided: true,
        ..default()
    };
    commands.entity(add.event_target()).insert((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        NotShadowCaster,
    ));
}

/// Draws the stick and places the silhouette, every frame while the scale is shown, since both
/// follow the camera.
fn draw_scale(
    view: Res<ViewSettings>,
    species: Res<Species>,
    cameras: Query<(&TurntableCamera, &Camera, &Projection)>,
    mut figures: Query<(&mut Transform, &mut Visibility), With<ScaleFigure>>,
    mut gizmos: Gizmos<ScaleGizmos>,
) {
    let figure_visibility = if view.show_scale {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for (_, mut visibility) in &mut figures {
        visibility.set_if_neq(figure_visibility);
    }
    if !view.show_scale {
        return;
    }
    let Ok((turntable, camera, projection)) = cameras.single() else {
        return;
    };
    let (Projection::Perspective(perspective), Some(viewport)) =
        (projection, camera.logical_viewport_size())
    else {
        return;
    };
    // Metres per pixel at the axis's distance, near enough for the stick beside it.
    let pixel = 2.0 * turntable.distance * (perspective.fov * 0.5).tan() / viewport.y;
    let facing = Quat::from_rotation_y(turntable.azimuth);
    let right = facing * Vec3::X;

    let stick = right * (species.crown.radius + STICK_GAP * pixel);
    gizmos.line(stick, stick + Vec3::Y * STICK_HEIGHT, STICK_COLOR);
    let tenths = (STICK_HEIGHT * 10.0).round() as u32;
    for tenth in 1..=tenths {
        let (level, step) = match tenth {
            t if t % 10 == 0 => (0, 1.0),
            t if t % 5 == 0 => (1, 0.5),
            _ => (2, 0.1),
        };
        let gap = step / pixel;
        if gap < MIN_TICK_GAP {
            continue;
        }
        let height = tenth as f32 * 0.1;
        let base = stick + Vec3::Y * height;
        let tip = base + right * TICK_LENGTHS[level] * pixel;
        gizmos.line(base, tip, STICK_COLOR);
        if gap >= NUMBER_SIZE * 2.0 {
            let number = if level == 0 {
                format!("{height:.0}")
            } else {
                format!("{height:.1}")
            };
            gizmos.text(
                Isometry3d::new(tip + right * 4.0 * pixel, facing),
                &number,
                NUMBER_SIZE * pixel,
                Vec2::new(-0.5, 0.0),
                STICK_COLOR,
            );
        }
    }

    for (mut transform, _) in &mut figures {
        let place = stick + right * (FIGURE_GAP * pixel + 0.3);
        let placed = Transform::from_translation(place).with_rotation(facing);
        transform.set_if_neq(placed);
    }
}
