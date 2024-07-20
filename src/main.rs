//! Init and window functions

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::complexity)]
#![warn(clippy::style)]

mod building;

use bevy::{color::palettes::css::*, prelude::*};

use crate::building::BuildingsPlugin;

/// Main
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BuildingsPlugin))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (close_on_esc, draw_cursor_rectangle))
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

/// Draw a "debug" rectangle at cursor position
fn draw_cursor_rectangle(
    mut gizmos: Gizmos,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    // gizmos.grid_2d(
    //     Vec2::ZERO,
    //     0.0,
    //     UVec2::new(100, 100),
    //     Vec2::new(100.0, 60.0),
    //     GREY,
    // );

    let window = windows.single();
    let (camera, camera_transform) = cameras.single();

    let cursor_position = match window.cursor_position() {
        Some(x) => x,
        None => return,
    };

    let cursor_world_position = match camera.viewport_to_world_2d(camera_transform, cursor_position)
    {
        Some(x) => x,
        None => return,
    };

    gizmos.rect_2d(
        cursor_world_position,
        Rot2::IDENTITY,
        Vec2::new(100.0, 60.0),
        BLUE,
    );
}
