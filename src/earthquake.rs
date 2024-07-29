//! Everything related to spawning the plates and randomly moving them

use std::time::Duration;

use avian2d::prelude::*;
use bevy::audio::Volume;
use bevy::color::palettes::css::{BLACK, DARK_RED};
use bevy::{color::palettes::css::BROWN, prelude::*};
use rand::seq::IteratorRandom;

use crate::building::Building;
use crate::layers::{ground_layers, plates_layers};

/// Plates which will create the earthquake
#[derive(Component)]
struct Plate;

/// label which shows next cycle
#[derive(Component)]
struct EarthquakeLabel;

/// the sound that will play for earthquakes
#[derive(Component)]
struct EarthquakeSound;

/// Adds ground and plates
fn add_default_plates(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((
        AudioBundle {
            source: asset_server.load("earthquake.ogg"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: Volume::new(1.0),
                paused: true,
                ..default()
            },
        },
        EarthquakeSound,
    ));

    let width = 50.0;
    let height = 50.0;

    // Ground keeping plates up
    let ground = cmd
        .spawn((
            RigidBody::Static,
            Collider::rectangle(2000.0, 50.0),
            TransformBundle::from_transform(Transform::from_xyz(0.0, -30.0 - 25.0 - 50.0, 0.0)),
            ground_layers(),
        ))
        .id();

    // TODO: add joints
    let mut previous_plate: Option<Entity> = None;

    let mut x_offset: f32 = 0.0;
    for i in 0..20 {
        // Plates

        x_offset = -500.0 + i as f32 * width;
        let plate = cmd
            .spawn((
                Plate,
                RigidBody::Dynamic,
                Collider::rectangle(width, height),
                LockedAxes::ALL_LOCKED.unlock_translation_y(),
                plates_layers(),
                SpriteBundle {
                    texture: asset_server.load("ground.png"),
                    transform: Transform::from_xyz(
                        -500.0 + i as f32 * width,
                        -30.0 - height / 2.0,
                        0.0,
                    ),
                    ..default()
                },
            ))
            .id();

        if let Some(previous_plate) = previous_plate {
            cmd.spawn(DistanceJoint::new(previous_plate, plate).with_limits(width, height));
        } else {
            cmd.spawn(
                DistanceJoint::new(ground, plate)
                    .with_limits(0.0, height / 2.0 + 25.0 + 10.0)
                    .with_local_anchor_1(Vec2::new(x_offset, 0.0)),
            );
        }

        previous_plate = Some(plate);
    }

    cmd.spawn(
        DistanceJoint::new(ground, previous_plate.unwrap())
            .with_limits(0.0, height / 2.0 + 25.0 + 10.0)
            .with_local_anchor_1(Vec2::new(x_offset, 0.0)),
    );

    cmd.spawn((
        EarthquakeLabel,
        TextBundle::from_section(
            "Next Earthquake in: ",
            TextStyle {
                font: asset_server.load("fonts/RobotoSlab.ttf"),
                font_size: 101.0,
                color: BLACK.into(),
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(0.0),
            left: Val::Px(0.0),
            ..default()
        }),
    ));

    cmd.spawn((
        EarthquakeLabel,
        TextBundle::from_section(
            "Next Earthquake in: ",
            TextStyle {
                font: asset_server.load("fonts/RobotoSlab.ttf"),
                font_size: 100.0,
                color: DARK_RED.into(),
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(0.0),
            left: Val::Px(0.0),
            ..default()
        }),
    ));
}

/// Timer for earthquake
#[derive(Resource)]
struct EarthquakeTimer {
    /// counter
    count: i32,
    /// the timer which decides when the next earthquake happens
    next: Timer,
    /// the timer which dictates when the earthquake stops
    stop: Timer,
    /// The timer inbetween "rumbles", aka the small earthquakes
    rumbles: Timer,
}

fn init_timers(mut timers: ResMut<EarthquakeTimer>) {
    timers.next.reset();
    timers.stop.reset();
    timers.rumbles.reset();

    timers.stop.pause();
    timers.rumbles.pause();
}

/// Generates the earthquace by forcing ceratin plates upwards
fn earthquake(
    mut cmd: Commands,
    plates: Query<Entity>,
    delta: Res<Time>,
    mut timers: ResMut<EarthquakeTimer>,
    keys: Res<ButtonInput<KeyCode>>,
    mut sounds: Query<&mut AudioSink, With<EarthquakeSound>>,
) {
    timers.next.tick(delta.delta());

    if sounds.is_empty() {
        return;
    }
    let sound = sounds.single_mut();

    let elapsed = timers.next.elapsed_secs();
    let remaining = timers.next.remaining_secs();
    if remaining < 2.0 {
        sound.play();
        sound.set_volume(1.0 / 2.0 * (2.0 - remaining));
    } else if timers.stop.paused() && elapsed <= 5.0 {
        // an earthquake takes 3s
        sound.set_volume(1.0 / 2.0 * (5.0 - elapsed));
    }

    // if keys.just_pressed(KeyCode::KeyX)  {
    if timers.next.just_finished() {
        sound.set_volume(1.0);
        timers.count += 1;
        let secs = (timers.next.duration().as_secs() - 1).max(5);
        timers.next.set_duration(Duration::from_secs(secs));
        timers.stop.unpause();
        timers.stop.reset();
        timers.rumbles.unpause();
        timers.rumbles.reset();
    }

    timers.stop.tick(delta.delta());
    timers.rumbles.tick(delta.delta());

    if timers.stop.just_finished() {
        timers.stop.pause();
        timers.rumbles.pause();
    }

    if timers.rumbles.just_finished() {
        let mut rng = rand::thread_rng();
        let earthquake_plates = plates.iter().choose_multiple(&mut rng, 6);

        for plate_entity in &earthquake_plates {
            cmd.entity(*plate_entity).insert(
                ExternalForce::new(
                    Vec2::Y * 3000000.0 * (timers.count as f32 + 10.0).log10().powi(4),
                )
                .with_persistence(false),
            );
        }
    }
}

/// Debug outline of plates
fn outline_plates(plates: Query<&GlobalTransform, With<Plate>>, mut gizmos: Gizmos) {
    for transform in &plates {
        let dir = transform.right();
        let angle = dir.y.atan2(dir.x);

        gizmos.rect_2d(
            transform.translation().xy(),
            angle,
            Vec2::new(50.0, 50.0),
            BROWN,
        );
    }
}

/// update earthquake label
fn update_earthquake_text(
    mut texts: Query<&mut Text, With<EarthquakeLabel>>,
    timer: Res<EarthquakeTimer>,
) {
    for mut text in texts.iter_mut() {
        text.sections[0].value = format!(
            "Earthquake #{} in {}s",
            timer.count + 1,
            timer.next.remaining_secs() as i64
        );
    }
}

/// Despawn buildings which are tilted more than X radian
fn remove_tilted_buildings(buildings: Query<&GlobalTransform, With<Building>>) {
    // TODO: check for tilted
}

/// Earthquake logic bundled into plugin
pub struct EarthquakePlugin;

impl Plugin for EarthquakePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (add_default_plates, init_timers))
            .insert_resource(EarthquakeTimer {
                count: 0,
                next: Timer::from_seconds(25.0, TimerMode::Repeating),
                stop: Timer::from_seconds(3.0, TimerMode::Repeating),
                rumbles: Timer::from_seconds(0.1, TimerMode::Repeating),
            })
            .add_systems(FixedUpdate, earthquake)
            .add_systems(Update, update_earthquake_text);
    }
}
