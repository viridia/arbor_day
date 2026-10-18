//! How the target game renders a plant, so the view shows it as the game will: its tonemapping
//! and its light. Panoply's for now; other games' would be project settings (ledger X6).

use bevy::{prelude::*, render::view::Tonemapping};

pub struct LookPlugin;

impl Plugin for LookPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameLook>()
            .add_systems(Update, apply_look);
    }
}

/// The game's look. Reflected, so it can be tuned at runtime over BRP.
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct GameLook {
    pub tonemapping: Tonemapping,
    /// The sun's color and illuminance in lux, as `DirectionalLight`'s.
    pub sun_color: Color,
    pub sun_illuminance: f32,
    /// Ambient light's color and brightness, as `GlobalAmbientLight`'s.
    pub ambient_color: Color,
    pub ambient_brightness: f32,
}

impl Default for GameLook {
    fn default() -> Self {
        // Panoply's: textures are painted to be seen as they are, so no tonemapping curve, and
        // light kept low enough that little clips (`Sunlight`). Its hemisphere light, sky
        // #c8d1d8 and ground #b97a20, stands as ambient light in their average, as Panoply sets
        // `GlobalAmbientLight` for materials without its shader.
        let sky = LinearRgba::from(Srgba::rgb_u8(0xc8, 0xd1, 0xd8));
        let ground = LinearRgba::from(Srgba::rgb_u8(0xb9, 0x7a, 0x20));
        Self {
            tonemapping: Tonemapping::Linear,
            sun_color: Color::WHITE,
            sun_illuminance: 1750.0,
            ambient_color: ((sky + ground) * 0.5).into(),
            ambient_brightness: 700.0,
        }
    }
}

/// Applies the look to the 3D camera and the sun when it changes, and to cameras and lights as
/// they spawn.
fn apply_look(
    mut commands: Commands,
    look: Res<GameLook>,
    cameras: Query<(Entity, Ref<Camera3d>)>,
    mut suns: Query<&mut DirectionalLight>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    for (entity, camera) in &cameras {
        if look.is_changed() || camera.is_added() {
            commands.entity(entity).insert(look.tonemapping);
        }
    }
    for mut sun in &mut suns {
        if look.is_changed() || sun.is_added() {
            sun.color = look.sun_color;
            sun.illuminance = look.sun_illuminance;
        }
    }
    if look.is_changed() {
        ambient.color = look.ambient_color;
        ambient.brightness = look.ambient_brightness;
    }
}
