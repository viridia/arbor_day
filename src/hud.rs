//! The HUD over the 3D view: controls that change the preview rather than the species, and
//! figures about the plant. Its buttons have fixed colors, not the feathers theme: they sit over
//! the scene, not over a panel, so they would not follow a light or dark theme.

use bevy::{
    feathers::{constants::size, display::label},
    picking::{cursor::EntityCursor, events::PointerDrag, hover::Hovered},
    prelude::*,
    settings::SaveSettingsDeferred,
    ui::Pressed,
    ui_widgets::{Activate, Button},
    window::SystemCursorIcon,
};

use crate::{
    plant::{PlantEnvelope, PlantStats},
    settings::{SAVE_DELAY, ViewSettings},
    turntable::TurntableCamera,
};

/// A HUD button's background, at rest, under the pointer and pressed: a gray that shows over the
/// sky and the ground alike, faint enough not to draw the eye from the plant.
const BUTTON_BG: Color = Color::srgba(0.5, 0.5, 0.5, 0.2);
const BUTTON_BG_HOVER: Color = Color::srgba(0.5, 0.5, 0.5, 0.3);
const BUTTON_BG_PRESSED: Color = Color::srgba(0.5, 0.5, 0.5, 0.4);

/// A toggle's icon while what it shows is shown, and while it is hidden.
const ICON_ON: Color = Color::WHITE;
const ICON_OFF: Color = Color::srgba(1.0, 1.0, 1.0, 0.4);

/// How fast a held camera button moves the camera, per second.
const ORBIT_RATE: f32 = 60.0 * std::f32::consts::PI / 180.0;
const TILT_RATE: f32 = 30.0 * std::f32::consts::PI / 180.0;
/// A factor on the distance, per second.
const ZOOM_RATE: f32 = 2.0;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                style_hud_buttons,
                show_targets,
                show_triangles,
                hold_camera_buttons,
            ),
        );
    }
}

/// The ground disc, which the HUD's ground toggle hides.
#[derive(Component, Default, Clone)]
pub struct Ground;

/// What a HUD toggle shows and hides.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum HudTarget {
    #[default]
    Envelope,
    Ground,
}

impl HudTarget {
    /// Whether the settings show this target.
    fn shown(self, view: &ViewSettings) -> bool {
        match self {
            HudTarget::Envelope => view.show_envelope,
            HudTarget::Ground => view.show_ground,
        }
    }
}

/// A HUD button that shows and hides its target, its icon dimmed while the target is hidden. The
/// state is a [`ViewSettings`] field, so it is saved.
#[derive(Component, Default, Clone)]
pub struct HudToggle(pub HudTarget);

/// How a camera button moves the camera.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraMove {
    #[default]
    Frame,
    OrbitLeft,
    OrbitRight,
    ZoomIn,
    ZoomOut,
    TiltUp,
    TiltDown,
}

/// A HUD button that moves the camera for as long as it is held. Frame instead acts on a click,
/// as a button does.
#[derive(Component, Default, Clone)]
pub struct CameraButton(pub CameraMove);

/// Marks a HUD button, for its colors.
#[derive(Component, Default, Clone)]
struct HudButton;

/// The text showing the plant's triangle count.
#[derive(Component, Default, Clone)]
struct TriangleCount;

/// The HUD, laid over the view as a grid with the panel's spacing. Toggles in the upper left; the
/// camera's buttons in the upper right, an L with orbit along the top, zoom and tilt down the
/// side, and Frame in the corner; status along the bottom, the triangle count at its left. Only
/// its buttons take the pointer, so the view's drags and scrolls pass between them.
pub fn hud() -> impl Scene {
    // Orbit's icon is drawn for one way; mirrored for the other, and turned upright for tilt.
    let mirrored = UiTransform::from_scale(Vec2::new(-1.0, 1.0));
    let up = UiTransform::from_rotation(Rot2::degrees(-90.0));
    let down = UiTransform::from_rotation(Rot2::degrees(90.0));
    bsn! {
        #Hud
        Node {
            display: Display::Grid,
            width: percent(100),
            height: percent(100),
            padding: px(8),
            row_gap: px(8),
            column_gap: px(8),
            grid_template_columns: {vec![
                GridTrack::auto(),
                GridTrack::auto(),
                GridTrack::flex(1.0),
                GridTrack::auto(),
                GridTrack::auto(),
                GridTrack::auto(),
            ]},
            grid_template_rows: {vec![
                GridTrack::auto(),
                GridTrack::auto(),
                GridTrack::auto(),
                GridTrack::auto(),
                GridTrack::auto(),
                GridTrack::flex(1.0),
                GridTrack::auto(),
            ]},
        }
        Pickable::IGNORE
        // A drag that starts on a button would otherwise reach the view and turn the camera.
        on(|mut drag: On<PointerDrag>| drag.propagate(false))
        Children [
            @hud_button("icons/language.png", UiTransform::IDENTITY)
            #envelope
            HudToggle(HudTarget::Envelope)
            on(toggle_target)
            Node { grid_row: GridPlacement::start(1), grid_column: GridPlacement::start(1) }
            --
            @hud_button("icons/landscape.png", UiTransform::IDENTITY)
            #ground
            HudToggle(HudTarget::Ground)
            on(toggle_target)
            Node { grid_row: GridPlacement::start(1), grid_column: GridPlacement::start(2) }
            --
            @hud_button("icons/360.png", mirrored)
            #orbit_left
            CameraButton(CameraMove::OrbitLeft)
            Node { grid_row: GridPlacement::start(1), grid_column: GridPlacement::start(4) }
            --
            @hud_button("icons/360.png", UiTransform::IDENTITY)
            #orbit_right
            CameraButton(CameraMove::OrbitRight)
            Node { grid_row: GridPlacement::start(1), grid_column: GridPlacement::start(5) }
            --
            @hud_button("icons/center_focus_strong.png", UiTransform::IDENTITY)
            #frame
            CameraButton(CameraMove::Frame)
            on(frame_camera)
            Node { grid_row: GridPlacement::start(1), grid_column: GridPlacement::start(6) }
            --
            @hud_button("icons/zoom_in.png", UiTransform::IDENTITY)
            #zoom_in
            CameraButton(CameraMove::ZoomIn)
            Node { grid_row: GridPlacement::start(2), grid_column: GridPlacement::start(6) }
            --
            @hud_button("icons/zoom_out.png", UiTransform::IDENTITY)
            #zoom_out
            CameraButton(CameraMove::ZoomOut)
            Node { grid_row: GridPlacement::start(3), grid_column: GridPlacement::start(6) }
            --
            @hud_button("icons/360.png", up)
            #tilt_up
            CameraButton(CameraMove::TiltUp)
            Node { grid_row: GridPlacement::start(4), grid_column: GridPlacement::start(6) }
            --
            @hud_button("icons/360.png", down)
            #tilt_down
            CameraButton(CameraMove::TiltDown)
            Node { grid_row: GridPlacement::start(5), grid_column: GridPlacement::start(6) }
            --
            @label("")
            #triangles
            TriangleCount
            Pickable::IGNORE
            Node {
                grid_row: GridPlacement::start(7),
                grid_column: GridPlacement::span(3),
                justify_self: JustifySelf::Start,
            }
        ]
    }
}

/// A square button the size of feathers' tool button, holding an icon drawn with `transform`.
fn hud_button(icon: &'static str, transform: UiTransform) -> impl Scene {
    bsn! {
        Node {
            width: size::ROW_HEIGHT,
            height: size::ROW_HEIGHT,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(4)),
        }
        Button
        HudButton
        Hovered
        EntityCursor::System(SystemCursorIcon::Pointer)
        BackgroundColor(BUTTON_BG)
        Children [
            Node { height: px(18) }
            ImageNode { image: icon }
            UiTransform { scale: {transform.scale}, rotation: {transform.rotation} }
            Pickable::IGNORE
        ]
    }
}

fn toggle_target(
    activate: On<Activate>,
    toggles: Query<&HudToggle>,
    mut view: ResMut<ViewSettings>,
    mut commands: Commands,
) {
    let Ok(toggle) = toggles.get(activate.entity) else {
        return;
    };
    let shown = match toggle.0 {
        HudTarget::Envelope => &mut view.show_envelope,
        HudTarget::Ground => &mut view.show_ground,
    };
    *shown = !*shown;
    commands.queue(SaveSettingsDeferred(SAVE_DELAY));
}

/// Shows or hides each target as the settings say, every frame, since the envelope's `Visibility`
/// arrives with its first mesh.
fn show_targets(
    view: Res<ViewSettings>,
    mut envelopes: Query<&mut Visibility, (With<PlantEnvelope>, Without<Ground>)>,
    mut grounds: Query<&mut Visibility, (With<Ground>, Without<PlantEnvelope>)>,
) {
    let visibility = |shown| {
        if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    for mut envelope in &mut envelopes {
        envelope.set_if_neq(visibility(view.show_envelope));
    }
    for mut ground in &mut grounds {
        ground.set_if_neq(visibility(view.show_ground));
    }
}

fn frame_camera(_activate: On<Activate>, mut cameras: Query<&mut TurntableCamera>) {
    if let Ok(mut camera) = cameras.single_mut() {
        camera.frame();
    }
}

/// Moves the camera while a camera button is held.
fn hold_camera_buttons(
    buttons: Query<&CameraButton, With<Pressed>>,
    mut cameras: Query<&mut TurntableCamera>,
    time: Res<Time<Real>>,
) {
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };
    let seconds = time.delta_secs();
    for button in &buttons {
        match button.0 {
            CameraMove::Frame => {}
            CameraMove::OrbitLeft => camera.spin(ORBIT_RATE * seconds),
            CameraMove::OrbitRight => camera.spin(-ORBIT_RATE * seconds),
            CameraMove::ZoomIn => camera.zoom(ZOOM_RATE.powf(-seconds), None),
            CameraMove::ZoomOut => camera.zoom(ZOOM_RATE.powf(seconds), None),
            CameraMove::TiltUp => camera.tilt(TILT_RATE * seconds),
            CameraMove::TiltDown => camera.tilt(-TILT_RATE * seconds),
        }
    }
}

/// Colors each HUD button by the pointer, and dims a toggle's icon while its target is hidden.
fn style_hud_buttons(
    mut buttons: Query<
        (
            &mut BackgroundColor,
            &Hovered,
            Has<Pressed>,
            Option<&HudToggle>,
            &Children,
        ),
        With<HudButton>,
    >,
    mut icons: Query<&mut ImageNode>,
    view: Res<ViewSettings>,
) {
    for (mut background, hovered, pressed, toggle, children) in &mut buttons {
        let color = match (pressed, hovered.0) {
            (true, _) => BUTTON_BG_PRESSED,
            (false, true) => BUTTON_BG_HOVER,
            (false, false) => BUTTON_BG,
        };
        background.set_if_neq(BackgroundColor(color));

        let Some(toggle) = toggle else {
            continue;
        };
        let tint = if toggle.0.shown(&view) {
            ICON_ON
        } else {
            ICON_OFF
        };
        for &child in children {
            if let Ok(mut icon) = icons.get_mut(child)
                && icon.color != tint
            {
                icon.color = tint;
            }
        }
    }
}

fn show_triangles(
    stats: Query<&PlantStats, Changed<PlantStats>>,
    mut texts: Query<&mut Text, With<TriangleCount>>,
) {
    let Ok(stats) = stats.single() else {
        return;
    };
    for mut text in &mut texts {
        text.0 = format!("{} triangles", stats.triangles);
    }
}
