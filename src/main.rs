//! Init and window functions

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::complexity)]
#![warn(clippy::style)]

mod building;
mod layers;

use avian2d::{debug_render::PhysicsDebugPlugin, PhysicsPlugins};
use bevy::prelude::*;
use bevy::{asset::AssetMetaCheck, window::WindowResolution};

use crate::building::BuildingsPlugin;

// TODO: detect window resizes for fullscreen support

/// Main
fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(AssetPlugin {
                // Wasm builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics in web builds on itch.
                // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                meta_check: AssetMetaCheck::Never,
                ..default()
            }), // .set(WindowPlugin {
                //     primary_window: Some(Window {
                //         resolution: WindowResolution::new(640., 360.)
                //             .with_scale_factor_override(1.0),
                //         ..default()
                //     }),
                //     ..default()
                // }),
        )
        .add_plugins((BuildingsPlugin,))
        .add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, close_on_esc)
        .run();
}

/// Close window on esc pressed
pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}

/// Init 2d camera
fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle {
        transform: Transform::from_xyz(0.0, 100.0, 0.0),
        ..default()
    },));
}
