//! App state saved between runs with `bevy_settings`: the window, the panel's width, what the
//! view shows, and feathers' color picker preferences.

use std::time::Duration;

use bevy::{
    ecs::system::NonSendMarker,
    feathers::controls::ColorInputSettings,
    prelude::*,
    settings::{
        ReflectSettingsGroup, SaveSettingsDeferred, SaveSettingsSync, SettingsGroup, SettingsPlugin,
    },
    ui_widgets::{Pane, ValueChange},
    window::{PrimaryWindow, WindowCloseRequested, WindowMode, WindowResized, WindowResolution},
    winit::WINIT_WINDOWS,
};

use crate::{PanelPane, ViewportSlot};

/// Narrowest either pane may be, in logical pixels; the shell's panes use it as `min_size`.
pub const MIN_PANE_WIDTH: f32 = 200.0;

pub const SAVE_DELAY: Duration = Duration::from_millis(500);

/// The window's position, size and fullscreen state.
#[derive(Resource, SettingsGroup, Reflect, Default, Clone, PartialEq, Debug)]
#[reflect(Resource, SettingsGroup, Default)]
#[settings_group(group = "window")]
pub struct WindowSettings {
    pub fullscreen: bool,
    /// Physical pixels, as winit reports them.
    pub position: Option<IVec2>,
    /// Logical pixels.
    pub size: Option<UVec2>,
}

/// The panel's width, in logical pixels.
#[derive(Resource, SettingsGroup, Reflect, Clone, PartialEq, Debug)]
#[reflect(Resource, SettingsGroup, Default)]
#[settings_group(group = "layout")]
pub struct LayoutSettings {
    pub panel_width: f32,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self { panel_width: 320.0 }
    }
}

/// What the view shows besides the plant, toggled on the HUD.
#[derive(Resource, SettingsGroup, Reflect, Clone, PartialEq, Debug)]
#[reflect(Resource, SettingsGroup, Default)]
#[settings_group(group = "view")]
pub struct ViewSettings {
    pub show_envelope: bool,
    pub show_ground: bool,
    pub show_wireframe: bool,
    pub show_scale: bool,
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self {
            show_envelope: true,
            show_ground: true,
            show_wireframe: false,
            show_scale: true,
        }
    }
}

/// Loads the settings, applies them, and saves changes. The window must exist when this is added,
/// so add it after `DefaultPlugins`, with `WindowPlugin::exit_condition` set to `DontExit` so that
/// closing the window saves before exiting.
pub struct AppSettingsPlugin;

impl Plugin for AppSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SettingsPlugin::new("org.viridia.arbor_day"));
        apply_window_settings(app);
        app.add_systems(
            Update,
            (
                update_window_settings,
                exit_on_window_close,
                fit_panel_width,
                save_color_input_settings,
            ),
        )
        .add_systems(Last, save_on_exit);
    }
}

fn apply_window_settings(app: &mut App) {
    let world = app.world_mut();
    let Some(settings) = world.get_resource::<WindowSettings>().cloned() else {
        return;
    };
    let Ok(mut window) = world
        .query_filtered::<&mut Window, With<PrimaryWindow>>()
        .single_mut(world)
    else {
        warn!("Primary window not found");
        return;
    };

    if let Some(position) = settings.position {
        window.position = WindowPosition::new(position);
    }
    // Before the window is created its scale factor is 1, and winit creates it at the logical
    // size, so the saved logical size goes in as the "physical" one.
    if let Some(size) = settings.size {
        window.resolution = WindowResolution::new(size.x, size.y);
    }
    if settings.fullscreen {
        window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
    }
}

/// Runs on the main thread (`NonSendMarker`) because it reads `WINIT_WINDOWS`.
fn update_window_settings(
    mut moves: MessageReader<WindowMoved>,
    mut resizes: MessageReader<WindowResized>,
    window: Single<(Entity, &Window), With<PrimaryWindow>>,
    mut settings: ResMut<WindowSettings>,
    mut commands: Commands,
    _main_thread: NonSendMarker,
) {
    // Drain both readers, so stale messages don't carry over.
    let moved = moves.read().count() > 0;
    let resized = resizes.read().count() > 0;
    if !moved && !resized {
        return;
    }
    let (entity, window) = *window;
    let changed = settings.set_if_neq(WindowSettings {
        fullscreen: window.mode != WindowMode::Windowed,
        position: match window.position {
            WindowPosition::At(position) => Some(position),
            // A window never moved is still `Automatic` or `Centered`, so ask winit where it is;
            // otherwise a session that only resized would forget the position.
            _ => WINIT_WINDOWS.with_borrow(|windows| {
                let position = windows.get_window(entity)?.outer_position().ok()?;
                Some(IVec2::new(position.x, position.y))
            }),
        },
        size: Some(UVec2::new(
            window.resolution.width() as u32,
            window.resolution.height() as u32,
        )),
    });
    if changed {
        commands.queue(SaveSettingsDeferred(SAVE_DELAY));
    }
}

/// Keeps the panel at its saved width. Pane sizes are flex weights, so with weights in pixels
/// that sum to the space the two panes share, the panel is exactly `panel_width` wide. The weights
/// are reset when that space changes (the window resized) or the width does.
fn fit_panel_width(
    layout: Res<LayoutSettings>,
    mut panel: Single<(&mut Pane, &ComputedNode), (With<PanelPane>, Without<ViewportSlot>)>,
    mut view: Single<(&mut Pane, &ComputedNode), With<ViewportSlot>>,
    mut last_total: Local<f32>,
) {
    let logical_width = |node: &ComputedNode| node.size().x * node.inverse_scale_factor();
    let total = logical_width(panel.1) + logical_width(view.1);
    if total < 2.0 * MIN_PANE_WIDTH {
        return;
    }
    if (total - *last_total).abs() < 0.5 && !layout.is_changed() {
        return;
    }
    *last_total = total;

    let width = layout
        .panel_width
        .clamp(MIN_PANE_WIDTH, total - MIN_PANE_WIDTH);
    if (panel.0.size - width).abs() >= 0.5 {
        panel.0.size = width;
    }
    if (view.0.size - (total - width)).abs() >= 0.5 {
        view.0.size = total - width;
    }
}

/// Observer on the split pane: a finished drag proposes the panes' sizes, and with weights in
/// pixels (see [`fit_panel_width`]) the first is the panel's new width.
pub fn save_panel_width(
    change: On<ValueChange<Vec<f32>>>,
    mut layout: ResMut<LayoutSettings>,
    mut commands: Commands,
) {
    if !change.is_final {
        return;
    }
    let Some(&width) = change.value.first() else {
        return;
    };
    if layout.set_if_neq(LayoutSettings {
        panel_width: width.round(),
    }) {
        commands.queue(SaveSettingsDeferred(SAVE_DELAY));
    }
}

/// Saves feathers' color picker preferences (its mode and recent colors) when they change. Feathers
/// only marks them changed, and Cmd+Q on macOS quits before an exit save could run.
fn save_color_input_settings(settings: Res<ColorInputSettings>, mut commands: Commands) {
    if settings.is_changed() && !settings.is_added() {
        commands.queue(SaveSettingsDeferred(SAVE_DELAY));
    }
}

fn exit_on_window_close(mut close: MessageReader<WindowCloseRequested>, mut commands: Commands) {
    if close.read().next().is_some() {
        commands.write_message(AppExit::Success);
    }
}

/// Saves whatever changed before the app exits, however the exit was asked for: closing the
/// window, or an `AppExit` from elsewhere (the remote driver ends a plan with one).
fn save_on_exit(mut exits: MessageReader<AppExit>, mut commands: Commands) {
    if exits.read().next().is_some() {
        commands.queue(SaveSettingsSync::IfChanged);
    }
}
