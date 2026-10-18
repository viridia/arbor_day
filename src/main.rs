//! An authoring app for low-poly plants. For now, a shell: a parameter panel beside a 3D view.

use bevy::{
    camera::Viewport,
    ecs::system::SystemParam,
    feathers::{
        FeathersPlugins,
        controls::{
            FeathersPane, FeathersScrollbar, FeathersSplitPane, FeathersSplitPaneHandle,
            ScrollbarGutter,
        },
        dark_theme::create_dark_theme,
        theme::{ThemeBackgroundColor, UiTheme},
        tokens,
    },
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    ui::{IsDefaultUiCamera, UiGlobalTransform, UiSystems},
    ui_widgets::{ControlOrientation, ScrollArea, split_pane_self_update},
    window::{ExitCondition, PrimaryWindow},
};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
};
use bevy_remote_driver::RemoteDriverPlugin;
use document::{Document, DocumentPlugin};
use hud::{Ground, HudPlugin, hud};
use keys::KeysPlugin;
use outline::{OutlinePlugin, PlantOutline};
use plant::{Plant, PlantEnvelope, PlantLeaves, PlantPlugin, PlantStats};
use properties::{CapsPanel, HeaderNotes};
use settings::{AppSettingsPlugin, MIN_PANE_WIDTH, save_panel_width};
use species::Species;
use turntable::{
    TurntableCamera, TurntablePlugin, TurntableSubject, orbit_on_drag, zoom_on_scroll,
};

mod document;
mod fuzzy;
mod generator;
mod gltf_file;
mod hud;
mod keys;
mod look;
mod outline;
mod plant;
mod properties;
mod random;
mod settings;
mod shortcut;
mod species;
mod species_file;
mod spread_strip;
mod toggle;
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
            BevyImmediateAttachPlugin::<CapsPanel, ParamPanel>::new(),
            RemoteDriverPlugin,
            TurntablePlugin,
            KeysPlugin,
            PlantPlugin,
            OutlinePlugin,
            HudPlugin,
            look::LookPlugin,
            shortcut::ShortcutPlugin,
            DocumentPlugin,
        ))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_observer(use_game_fov)
        .add_systems(Startup, (scene.spawn(), shell_ui.spawn()))
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

fn scene() -> impl SceneList {
    // Lines take no light, so the envelope reads the same from every side.
    let envelope_material = StandardMaterial {
        base_color: Color::srgba(0.6, 0.85, 0.6, 0.18),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    };
    bsn_list! {
        #SceneCamera
        SceneCamera
        Camera3d
        TurntableCamera
        --
        // Draws the UI over the whole window, on top of the scene camera.
        #UiCamera
        Camera2d
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
        }
        IsDefaultUiCamera
        --
        #Sun
        DirectionalLight {
            shadow_maps_enabled: true,
        }
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y)
        --
        #Ground
        Ground
        Mesh3d(asset_value(Circle::new(4.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb(0.25, 0.28, 0.3)))
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
        --
        #Plant
        Plant
        TurntableSubject
        // White, so the mesh's vertex colors show as they are, and matte, as the leaves are.
        MeshMaterial3d::<StandardMaterial>(asset_value(StandardMaterial {
            perceptual_roughness: 1.0,
            ..default()
        }))
        --
        #Outline
        PlantOutline
        --
        // Not framed: a bare plant has no leaves to bound, and the envelope bounds them.
        #Leaves
        PlantLeaves
        --
        #Envelope
        PlantEnvelope
        // Framed with the plant, so Frame fits the whole crown.
        TurntableSubject
        NotShadowCaster
        NotShadowReceiver
        MeshMaterial3d::<StandardMaterial>(asset_value(envelope_material))
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
        #Shell
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
                #Panel
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
                        #Scrollbar
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
                #View
                ViewportSlot
                // Drags and scrolls over the view move the camera; the panel's stay in the panel.
                on(orbit_on_drag)
                on(zoom_on_scroll)
                Children [ @hud() ]
            ]
        ]
    }
}

#[derive(SystemParam)]
struct PanelParams<'w, 's> {
    species: ResMut<'w, Species>,
    stats: Query<'w, 's, &'static PlantStats>,
    document: Res<'w, Document>,
    commands: Commands<'w, 's>,
}

impl ImmediateAttach<CapsPanel> for ParamPanel {
    type Params = PanelParams<'static, 'static>;

    fn construct(ui: &mut Imm<CapsPanel>, params: &mut PanelParams) {
        if let Some(command) = document::header(ui, &params.document, &params.species) {
            params.commands.trigger(command);
        }
        // Edit a copy: writing through `ResMut` every frame would mark the species changed every
        // frame, and the plant would rebuild every frame.
        let mut edited = params.species.clone();
        // Each limb level's count in its header, so a level some setting has emptied shows.
        let notes: HeaderNotes = params
            .stats
            .single()
            .map(|stats| {
                stats
                    .level_limbs
                    .iter()
                    .enumerate()
                    .map(|(level, count)| (format!("levels[{level}]"), count.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        properties::edit_fields(ui, &mut edited, &notes);
        params.species.set_if_neq(edited);
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
