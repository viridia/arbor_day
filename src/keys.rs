//! The app's keys. They are fixed, so one observer on the window handles them: focused input
//! bubbles up to the window, and a text field stops the keys it types, so these never fire while
//! typing into a field.

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    input_focus::FocusedInput,
    prelude::*,
    window::PrimaryWindow,
};

use crate::turntable::TurntableCamera;

/// Distance factor for one press of +.
const ZOOM_STEP: f32 = 0.8;

pub struct KeysPlugin;

impl Plugin for KeysPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, observe_window);
    }
}

fn observe_window(window: Single<Entity, With<PrimaryWindow>>, mut commands: Commands) {
    commands.entity(*window).observe(on_key);
}

fn on_key(
    input: On<FocusedInput<KeyboardInput>>,
    keys: Res<ButtonInput<Key>>,
    mut cameras: Query<&mut TurntableCamera>,
    mut commands: Commands,
) {
    let event = &input.input;
    if event.state != ButtonState::Pressed {
        return;
    }
    let command = keys.pressed(if cfg!(target_os = "macos") {
        Key::Super
    } else {
        Key::Control
    });
    let Key::Character(character) = &event.logical_key else {
        return;
    };

    if command {
        if character.eq_ignore_ascii_case("q") && !event.repeat {
            commands.write_message(AppExit::Success);
        }
        return;
    }
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };
    match character.as_str() {
        "f" | "F" => camera.frame(),
        // `=` is + without Shift on many layouts.
        "+" | "=" => camera.zoom(ZOOM_STEP, None),
        "-" | "_" => camera.zoom(1.0 / ZOOM_STEP, None),
        _ => {}
    }
}
