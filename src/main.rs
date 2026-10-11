//! An authoring app for low-poly plants. For now, a shell: a parameter panel beside a 3D view.

use bevy::{
    camera::Viewport,
    ecs::system::SystemParam,
    feathers::{
        FeathersPlugins,
        containers::{subpane, subpane_body, subpane_header},
        controls::{FeathersPane, FeathersSlider, FeathersSplitPane, FeathersSplitPaneHandle},
        dark_theme::create_dark_theme,
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
        tokens,
    },
    prelude::*,
    ui::{IsDefaultUiCamera, UiGlobalTransform, UiSystems},
    ui_widgets::{SliderPrecision, SliderStep, split_pane_self_update},
    window::PrimaryWindow,
};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{CapsUiFeathers, slider_value::ImmUiSliderValue},
};
use bevy_remote_driver::RemoteDriverPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "arbor_day".into(),
                    ..default()
                }),
                ..default()
            }),
            FeathersPlugins,
            BevyImmediateAttachPlugin::<CapsUiFeathers, ParamPanel>::new(),
            RemoteDriverPlugin,
        ))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_systems(Startup, (scene.spawn(), shell_ui.spawn()))
        .add_systems(Update, apply_test_shape)
        .add_systems(PostUpdate, follow_viewport_slot.after(UiSystems::Layout))
        .run();
}

/// The camera that draws the 3D view.
#[derive(Component, Default, Clone)]
struct SceneCamera;

/// The UI node whose rectangle the [`SceneCamera`]'s viewport follows.
#[derive(Component, Default, Clone)]
struct ViewportSlot;

/// The root of the immediate-mode parameter panel.
#[derive(Component, Default, Clone)]
struct ParamPanel;

/// The object in the view and what the panel edits, until the generator replaces both.
#[derive(Component, Reflect, Clone, PartialEq)]
#[reflect(Component)]
struct TestShape {
    scale: f32,
}

impl Default for TestShape {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

fn scene() -> impl SceneList {
    bsn_list! {
        Name("SceneCamera")
        SceneCamera
        Camera3d
        Transform::from_xyz(0.0, 2.0, 3.5).looking_at(Vec3::new(0.0, 0.25, 0.0), Vec3::Y)
        --
        // Draws the UI over the whole window, on top of the scene camera.
        Name("UiCamera")
        Camera2d
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
        }
        IsDefaultUiCamera
        --
        Name("Sun")
        DirectionalLight {
            shadow_maps_enabled: true,
        }
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y)
        --
        Name("Ground")
        Mesh3d(asset_value(Circle::new(4.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb(0.25, 0.28, 0.3)))
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
        --
        Name("TestObject")
        TestShape
        Mesh3d(asset_value(Torus::default()))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb(0.3, 0.55, 0.3)))
        Transform::default()
    }
}

fn shell_ui() -> impl Scene {
    bsn! {
        Name("Shell")
        Node {
            width: percent(100),
            height: percent(100),
        }
        Pickable::IGNORE
        Children [
            @FeathersSplitPane
            Node {
                width: percent(100),
                height: percent(100),
            }
            on(split_pane_self_update)
            Children [
                @FeathersPane { @size: 1.0, @min_size: 200.0 }
                Name("Panel")
                ThemeBackgroundColor(tokens::WINDOW_BG)
                Children [
                    ParamPanel
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: px(8),
                        row_gap: px(8),
                    }
                ]
                --
                @FeathersSplitPaneHandle
                --
                @FeathersPane { @size: 3.0, @min_size: 200.0 }
                Name("View")
                ViewportSlot
                // Lets the pointer through to the 3D scene.
                Pickable::IGNORE
            ]
        ]
    }
}

#[derive(SystemParam)]
struct PanelParams<'w, 's> {
    shapes: Query<'w, 's, &'static mut TestShape>,
}

impl ImmediateAttach<CapsUiFeathers> for ParamPanel {
    type Params = PanelParams<'static, 'static>;

    fn construct(ui: &mut Imm<CapsUiFeathers>, params: &mut PanelParams) {
        let Ok(mut shape) = params.shapes.single_mut() else {
            return;
        };
        // Edit a copy: writing through `Mut` every frame would mark the shape changed every frame.
        let mut scale = shape.scale;

        ui.ch().on_spawn_apply_scene(subpane).add(|ui| {
            ui.ch().on_spawn_apply_scene(subpane_header).add(|ui| {
                ui.ch()
                    .on_spawn_apply_scene(|| bsn! { Text("Test object") ThemedText });
            });
            ui.ch().on_spawn_apply_scene(subpane_body).add(|ui| {
                ui.ch()
                    .on_spawn_apply_scene(|| {
                        bsn! {
                            @FeathersSlider { @min: 0.25, @max: 2.0 }
                            SliderStep(0.05)
                            SliderPrecision(2)
                            Name("Scale")
                        }
                    })
                    .slider(&mut scale);
            });
        });

        if shape.scale != scale {
            shape.scale = scale;
        }
    }
}

/// Scales the test object, keeping it on the ground.
fn apply_test_shape(mut shapes: Query<(&TestShape, &mut Transform), Changed<TestShape>>) {
    for (shape, mut transform) in &mut shapes {
        transform.scale = Vec3::splat(shape.scale);
        transform.translation.y = Torus::default().minor_radius * shape.scale;
    }
}

/// Fits the scene camera's viewport to the [`ViewportSlot`] node, so the 3D view fills whatever
/// space the layout gives that node.
fn follow_viewport_slot(
    slot: Single<(&ComputedNode, &UiGlobalTransform), With<ViewportSlot>>,
    mut camera: Single<&mut Camera, With<SceneCamera>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    // The node's center and size are both in physical pixels.
    let (node, transform) = *slot;
    let center = transform.affine().translation;
    let half = node.size() * 0.5;
    let bounds = window.physical_size().as_vec2();
    let min = (center - half).clamp(Vec2::ZERO, bounds).as_uvec2();
    let max = (center + half).clamp(Vec2::ZERO, bounds).as_uvec2();
    let size = max.saturating_sub(min);
    // A zero-sized viewport is a render error; keep the last one until the node has a size.
    if size.x == 0 || size.y == 0 {
        return;
    }
    if camera
        .viewport
        .as_ref()
        .is_some_and(|v| v.physical_position == min && v.physical_size == size)
    {
        return;
    }
    camera.viewport = Some(Viewport {
        physical_position: min,
        physical_size: size,
        ..default()
    });
}
