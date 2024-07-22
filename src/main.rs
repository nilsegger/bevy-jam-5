//! Init and window functions

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::complexity)]
#![warn(clippy::style)]

mod building;
mod layers;

use avian2d::{debug_render::PhysicsDebugPlugin, PhysicsPlugins};
use bevy::prelude::*;

use crate::building::BuildingsPlugin;

/// Main
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BuildingsPlugin))
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
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    },));
}
