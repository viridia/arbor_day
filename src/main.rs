//! An authoring app for low-poly plants. For now, a shell: a parameter panel beside a 3D view.

use bevy::{
    camera::Viewport,
    ecs::system::SystemParam,
    feathers::{
        FeathersPlugins,
        containers::{subpane, subpane_body, subpane_header},
        controls::{
            FeathersButton, FeathersPane, FeathersScrollbar, FeathersSplitPane,
            FeathersSplitPaneHandle, NumberInputUnits, ScrollbarGutter,
        },
        dark_theme::create_dark_theme,
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
        display::label,
        tokens,
    },
    prelude::*,
    ui::{IsDefaultUiCamera, UiGlobalTransform, UiSystems},
    ui_widgets::{ControlOrientation, ScrollArea, split_pane_self_update},
    window::{ExitCondition, PrimaryWindow},
};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{CapsUiFeathers, activated::ImmUiActivated, text::ImmUiText},
};
use bevy_remote_driver::RemoteDriverPlugin;
use fuzzy::{Distribution, Fuzzy};
use keys::KeysPlugin;
use plant::{Plant, PlantPlugin, PlantStats};
use properties::{Caption, Precision, Spinbox, ValueRange, Wrap};
use settings::{AppSettingsPlugin, MIN_PANE_WIDTH, save_panel_width};
use species::Species;
use turntable::{
    TurntableCamera, TurntablePlugin, TurntableSubject, orbit_on_drag, zoom_on_scroll,
};

mod fuzzy;
mod generator;
mod keys;
mod plant;
mod properties;
mod settings;
mod species;
mod spread_strip;
mod turntable;

/// The game camera's vertical field of view (Panoply's `DEFAULT_FOV`, 40°), so plants are seen as
/// players will see them.
const GAME_FOV: f32 = 0.69;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "arbor_day".into(),
                    ..default()
                }),
                // Closing the window exits through `AppSettingsPlugin`, which saves first.
                exit_condition: ExitCondition::DontExit,
                ..default()
            }),
            AppSettingsPlugin,
            FeathersPlugins,
            BevyImmediateAttachPlugin::<CapsUiFeathers, ParamPanel>::new(),
            RemoteDriverPlugin,
            TurntablePlugin,
            KeysPlugin,
            PlantPlugin,
        ))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_observer(use_game_fov)
        .add_systems(Startup, (scene.spawn(), shell_ui.spawn()))
        .add_systems(Update, apply_placeholder)
        .add_systems(PostUpdate, follow_viewport_slot.after(UiSystems::Layout))
        .run();
}

/// The camera that draws the 3D view.
#[derive(Component, Default, Clone)]
struct SceneCamera;

/// The UI node whose rectangle the [`SceneCamera`]'s viewport follows.
#[derive(Component, Default, Clone)]
struct ViewportSlot;

/// The split pane's pane holding the parameter panel.
#[derive(Component, Default, Clone)]
struct PanelPane;

/// The root of the immediate-mode parameter panel.
#[derive(Component, Default, Clone)]
struct ParamPanel;

/// Stands in for the species while the property editor is built: a field of each kind it will
/// edit. `show_ground` hides the ground, so an edit shows in the view. A component on a named
/// entity rather than a resource, so plans can read it.
#[derive(Component, Reflect, Clone, PartialEq)]
#[reflect(Component)]
struct Placeholder {
    #[reflect(@Caption("Show ground"))]
    show_ground: bool,
    smooth: bool,
    #[reflect(@ValueRange::<i32>(0..=10), @Spinbox)]
    count: i32,
    #[reflect(@ValueRange::<f32>(0.0..=1.0), @Precision(2))]
    amount: f32,
    #[reflect(
        @ValueRange::<f32>(-std::f32::consts::PI..=std::f32::consts::PI),
        @NumberInputUnits::new_static("angle_degrees"),
        @Wrap,
    )]
    angle: f32,
    #[reflect(@ValueRange::<f32>(0.0..=2.0), @Precision(2))]
    length: Fuzzy,
    #[reflect(
        @ValueRange::<f32>(-std::f32::consts::FRAC_PI_2..=std::f32::consts::FRAC_PI_2),
        @NumberInputUnits::new_static("angle_degrees"),
    )]
    tilt: Fuzzy,
    #[reflect(
        @ValueRange::<f32>(-std::f32::consts::PI..=std::f32::consts::PI),
        @NumberInputUnits::new_static("angle_degrees"),
        @Wrap,
    )]
    axis: Fuzzy,
    mode: PlaceholderMode,
    nested: PlaceholderNested,
}

impl Default for Placeholder {
    fn default() -> Self {
        Self {
            show_ground: true,
            smooth: false,
            count: 3,
            amount: 0.5,
            angle: 0.0,
            length: Fuzzy::new(1.0, 0.25, Distribution::Uniform),
            // In the units shown, so the input shows no tail of digits.
            tilt: Fuzzy::new(0.0, 15.0_f32.to_radians(), Distribution::Bell),
            // Its spread crosses the seam at 180°, so its strip is two segments.
            axis: Fuzzy::new(
                170.0_f32.to_radians(),
                30.0_f32.to_radians(),
                Distribution::Uniform,
            ),
            mode: PlaceholderMode::First,
            nested: PlaceholderNested::default(),
        }
    }
}

#[derive(Reflect, Clone, Copy, PartialEq, Default)]
enum PlaceholderMode {
    #[default]
    First,
    Second,
}

#[derive(Reflect, Clone, PartialEq, Default)]
struct PlaceholderNested {
    flag: bool,
    size: f32,
}

/// The ground disc, which [`Placeholder::show_ground`] hides.
#[derive(Component, Default, Clone)]
struct Ground;

fn scene() -> impl SceneList {
    bsn_list! {
        Name("SceneCamera")
        SceneCamera
        Camera3d
        TurntableCamera
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
        Ground
        Mesh3d(asset_value(Circle::new(4.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb(0.25, 0.28, 0.3)))
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
        --
        Name("Placeholder")
        Placeholder
        --
        Name("Plant")
        Plant
        TurntableSubject
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb(0.42, 0.33, 0.25)))
    }
}

fn use_game_fov(add: On<Add<SceneCamera>>, mut projections: Query<&mut Projection>) {
    if let Ok(mut projection) = projections.get_mut(add.event_target()) {
        *projection = Projection::Perspective(PerspectiveProjection {
            fov: GAME_FOV,
            ..default()
        });
    }
}

fn shell_ui() -> impl Scene {
    bsn! {
        Name("Shell")
        // `min_height: 0` down the whole chain: a flex item's minimum height is otherwise its
        // content's, so a tall panel would stretch every ancestor past the window, and nothing
        // would scroll.
        Node {
            width: percent(100),
            height: percent(100),
            min_height: px(0),
        }
        Pickable::IGNORE
        Children [
            @FeathersSplitPane
            Node {
                width: percent(100),
                height: percent(100),
                min_height: px(0),
            }
            on(split_pane_self_update)
            on(save_panel_width)
            Children [
                @FeathersPane { @size: 1.0, @min_size: MIN_PANE_WIDTH }
                Name("Panel")
                PanelPane
                ThemeBackgroundColor(tokens::WINDOW_BG)
                Children [
                    // Scrolls as feathers' list view does: a frame with a gutter for the
                    // scrollbar, around a node that scrolls.
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        min_height: px(0),
                        padding: UiRect { right: px(10) },
                    }
                    // Exactly the scrollbar (6 wide, 4 from the edge); the content's own margin
                    // is the gap before it.
                    ScrollbarGutter(px(10))
                    Children [
                        #scrolled
                        Node {
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            min_height: px(0),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        Children [
                            ParamPanel
                            Node {
                                flex_direction: FlexDirection::Column,
                                // The gutter is reserved only while the scrollbar shows, so the
                                // content keeps its own margin on the right.
                                padding: px(8),
                                row_gap: px(8),
                            }
                        ]
                        --
                        @FeathersScrollbar {
                            @target: #scrolled,
                            @orientation: {ControlOrientation::Vertical}
                        }
                        Name("Scrollbar")
                        Node {
                            position_type: PositionType::Absolute,
                            right: px(4),
                            top: px(0),
                            bottom: px(0),
                            width: px(6),
                        }
                    ]
                ]
                --
                @FeathersSplitPaneHandle
                --
                @FeathersPane { @size: 3.0, @min_size: MIN_PANE_WIDTH }
                Name("View")
                ViewportSlot
                // Drags and scrolls over the view move the camera; the panel's stay in the panel.
                on(orbit_on_drag)
                on(zoom_on_scroll)
            ]
        ]
    }
}

#[derive(SystemParam)]
struct PanelParams<'w, 's> {
    species: ResMut<'w, Species>,
    stats: Query<'w, 's, &'static PlantStats>,
    cameras: Query<'w, 's, &'static mut TurntableCamera>,
    placeholders: Query<'w, 's, &'static mut Placeholder>,
}

impl ImmediateAttach<CapsUiFeathers> for ParamPanel {
    type Params = PanelParams<'static, 'static>;

    fn construct(ui: &mut Imm<CapsUiFeathers>, params: &mut PanelParams) {
        // Edit a copy: writing through `ResMut` every frame would mark the species changed every
        // frame, and the plant would rebuild every frame.
        let mut edited = params.species.clone();
        ui.ch().on_spawn_apply_scene(subpane).add(|ui| {
            ui.ch().on_spawn_apply_scene(subpane_header).add(|ui| {
                ui.ch()
                    .on_spawn_apply_scene(|| bsn! { Text("Trunk") ThemedText });
            });
            ui.ch()
                .on_spawn_apply_scene(subpane_body)
                .add(|ui| properties::edit_fields(ui, &mut edited.trunk));
        });
        params.species.set_if_neq(edited);

        let triangles = params.stats.single().map_or(0, |stats| stats.triangles);
        ui.ch()
            .on_spawn_apply_scene(|| label(""))
            .on_spawn_insert(|| Name::new("Triangles"))
            .text(format!("{triangles} triangles"));

        ui.ch().on_spawn_apply_scene(subpane).add(|ui| {
            ui.ch().on_spawn_apply_scene(subpane_header).add(|ui| {
                ui.ch()
                    .on_spawn_apply_scene(|| bsn! { Text("View") ThemedText });
            });
            ui.ch().on_spawn_apply_scene(subpane_body).add(|ui| {
                let framed = ui
                    .ch()
                    .on_spawn_apply_scene(|| {
                        bsn! {
                            @FeathersButton { @caption: bsn! { Text("Frame") ThemedText } }
                            Name("Frame")
                        }
                    })
                    .activated();
                if framed && let Ok(mut camera) = params.cameras.single_mut() {
                    camera.frame();
                }
            });
        });

        let Ok(mut placeholder) = params.placeholders.single_mut() else {
            return;
        };
        let mut edited = placeholder.clone();
        ui.ch().on_spawn_apply_scene(subpane).add(|ui| {
            ui.ch().on_spawn_apply_scene(subpane_header).add(|ui| {
                ui.ch()
                    .on_spawn_apply_scene(|| bsn! { Text("Placeholder") ThemedText });
            });
            ui.ch()
                .on_spawn_apply_scene(subpane_body)
                .add(|ui| properties::edit_fields(ui, &mut edited));
        });
        placeholder.set_if_neq(edited);
    }
}

fn apply_placeholder(
    placeholders: Query<&Placeholder, Changed<Placeholder>>,
    mut grounds: Query<&mut Visibility, With<Ground>>,
) {
    let Ok(placeholder) = placeholders.single() else {
        return;
    };
    for mut visibility in &mut grounds {
        *visibility = if placeholder.show_ground {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
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
