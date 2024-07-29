//! Init and window functions

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::complexity)]
#![warn(clippy::style)]

mod building;
mod earthquake;
mod inhabitants;
mod layers;
mod player;

use avian2d::{debug_render::PhysicsDebugPlugin, PhysicsPlugins};
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::{asset::AssetMetaCheck, window::WindowResolution};
use bevy_screen_diagnostics::{
    ScreenDiagnosticsPlugin, ScreenEntityDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin,
};
use inhabitants::InhabitantPlugin;
use player::PlayerPlugin;

use crate::{building::BuildingsPlugin, earthquake::EarthquakePlugin};

// TODO: detect window resizes for fullscreen support

/// Main
fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics in web builds on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Earthquakes".to_string(), // ToDo
                        // Bind to canvas included in `index.html`
                        canvas: Some("#bevy".to_owned()),
                        fit_canvas_to_parent: true,
                        // Tells wasm not to override default event handling, like F5 and Ctrl+R
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        /* .add_plugins(ScreenDiagnosticsPlugin::default())
        .add_plugins(ScreenFrameDiagnosticsPlugin)
        .add_plugins(ScreenEntityDiagnosticsPlugin) */
        .add_plugins((
            BuildingsPlugin,
            EarthquakePlugin,
            InhabitantPlugin,
            PlayerPlugin,
        ))
        .add_plugins(PhysicsPlugins::default())
        // .add_plugins(PhysicsDebugPlugin::default())
        .add_systems(Startup, (setup_camera, init_level, setup_audio))
        .add_systems(Update, move_camera)
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
/// set it such that the ground is at the bottom of the screen
/// TODO: add resize event
fn setup_camera(mut commands: Commands, windows: Query<&Window>) {
    let window = windows.single();

    let height = window.size().y;

    commands.spawn((Camera2dBundle {
        transform: Transform::from_xyz(0.0, height / 2.0 - 200.0, 0.0),
        ..default()
    },));
}

/// loads background music
fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(AudioBundle {
        source: asset_server.load("bg.ogg"),
        settings: PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::new(0.3),
            ..default()
        },
    });
}

/// move camera with arrow keys and scroll
/// TODO: scroll
fn move_camera(
    mut cameras: Query<&mut Transform, With<Camera>>,
    windows: Query<&Window>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if cameras.is_empty() {
        return;
    }

    let window = windows.single();

    let height = window.size().y;

    let mut camera = cameras.single_mut();
    if keys.pressed(KeyCode::ArrowUp) {
        camera.translation.y += time.delta_seconds() * 300.0;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        camera.translation.y -= time.delta_seconds() * 300.0;
    }

    camera.translation.y = camera.translation.y.max(height / 2.0 - 200.0);
}

/// load sprites etc
fn init_level(mut commands: Commands, asset_server: Res<AssetServer>, windows: Query<&Window>) {
    let window = windows.single();

    // Background
    commands.spawn(SpriteBundle {
        texture: asset_server.load("backgroundheight.png"),
        sprite: Sprite {
            custom_size: Some(window.size()),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, -1.0),
        ..default()
    });
    commands.spawn(SpriteBundle {
        texture: asset_server.load("backgroundheight.png"),
        sprite: Sprite {
            custom_size: Some(window.size()),
            ..default()
        },
        transform: Transform::from_xyz(0.0, window.size().y, -1.0),
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("backgroundheight.png"),
        sprite: Sprite {
            custom_size: Some(window.size()),
            ..default()
        },
        transform: Transform::from_xyz(0.0, window.size().y * 2.0, -1.0),
        ..default()
    });
}
