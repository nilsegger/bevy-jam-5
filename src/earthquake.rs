//! Everything related to spawning the plates and randomly moving them

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::seq::IteratorRandom;

use crate::layers::{ground_layers, plates_layers};

/// Plates which will create the earthquake
#[derive(Component)]
struct Plate;

/// Adds ground and plates
fn add_default_plates(mut cmd: Commands) {
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
                TransformBundle::from_transform(Transform::from_xyz(
                    -500.0 + i as f32 * width,
                    -30.0 - height / 2.0,
                    0.0,
                )),
                plates_layers(),
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
}

/// Timer for earthquake
#[derive(Resource)]
struct EarthquakeTimer {
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
) {
    timers.next.tick(delta.delta());

    if keys.just_pressed(KeyCode::KeyX) {
        // if timers.next.just_finished() {
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
        let earthquake_plates = plates.iter().choose_multiple(&mut rng, 5);

        for plate_entity in &earthquake_plates {
            cmd.entity(*plate_entity)
                .insert(ExternalForce::new(Vec2::Y * 2000000.0).with_persistence(false));
        }
    }
}

/// Earthquake logic bundled into plugin
pub struct EarthquakePlugin;

impl Plugin for EarthquakePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (add_default_plates, init_timers))
            .insert_resource(EarthquakeTimer {
                next: Timer::from_seconds(10.0, TimerMode::Repeating),
                stop: Timer::from_seconds(3.0, TimerMode::Repeating),
                rumbles: Timer::from_seconds(0.1, TimerMode::Repeating),
            })
            .add_systems(FixedUpdate, earthquake);
    }
}
