//! Inhabitants will be livings inside of the buildings

use std::time::Duration;

use bevy::{color::palettes::css::GOLD, prelude::*};
use rand::Rng;

use crate::building::Building;

/// Width of a inhabitant
const WIDTH: f32 = 7.5;
/// Height of a inhabitant
const HEIGHT: f32 = 20.0;

/// An inhabitant with its home building marked
#[derive(Component)]
struct Inhabitant {
    /// next point inhabitant is walking to
    target_x: f32,
    /// when to move again
    move_timer: Timer,
}

/// Spawn a new inahbitant inside a building
#[derive(Event)]
pub struct SpawnNewInhabitant(pub Entity);

/// moves inhabitants randomly but if the building is rotated, they start falling down
fn move_inside_building(
    time: Res<Time>,
    mut inhabitants: Query<(&mut Inhabitant, &mut Transform, &Parent)>,
    buildings: Query<(&Building, &GlobalTransform)>,
) {
    // TODO: do nothing during earthquake

    for (mut inhabitant, mut inhabitant_transform, inhabitant_parent) in inhabitants.iter_mut() {
        let (building, building_global) = match buildings.get(inhabitant_parent.get()) {
            Ok(x) => x,
            Err(_) => continue, // NOTE: probably should despawn inhabitant...
        };

        let walk = (inhabitant.target_x - inhabitant_transform.translation.x).signum() * 3.0;

        let falling_velocity = -building_global.right().xy().to_angle();

        let halfsize = building.size.x / 2.0 - WIDTH / 2.0;

        inhabitant_transform.translation = Vec3::new(
            (inhabitant_transform.translation.x
                + (10.0 * falling_velocity + walk) * time.delta_seconds())
            .clamp(-halfsize, halfsize),
            -building.size.y / 2.0 + HEIGHT / 2.0,
            0.0,
        );

        inhabitant.move_timer.tick(time.delta());

        if inhabitant.move_timer.just_finished() {
            let mut rng = rand::thread_rng();
            inhabitant.target_x = rng.gen_range(-halfsize..halfsize);
            inhabitant.move_timer.set_duration(Duration::from_secs(5));
        }
    }
}

/// spawn new inhabitatns in the given building
fn handle_spawn_new_inhabitant(mut cmd: Commands, mut events: EventReader<SpawnNewInhabitant>) {
    for SpawnNewInhabitant(building_entity) in events.read() {
        let inhabitant = cmd
            .spawn((
                TransformBundle::IDENTITY,
                Inhabitant {
                    target_x: 0.0,
                    move_timer: Timer::from_seconds(0.0, TimerMode::Repeating),
                },
            ))
            .id();

        cmd.entity(*building_entity).add_child(inhabitant);
    }
}

/// outline inhabitants
fn outline_inhabitant(inhabitants: Query<&GlobalTransform, With<Inhabitant>>, mut gizmos: Gizmos) {
    for transform in &inhabitants {
        gizmos.rect_2d(
            transform.translation().xy(),
            transform.right().xy().to_angle(),
            Vec2::new(WIDTH, HEIGHT),
            GOLD,
        )
    }
}

/// Functions bundled
pub struct InhabitantPlugin;

impl Plugin for InhabitantPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnNewInhabitant>()
            .add_systems(Update, (move_inside_building, outline_inhabitant))
            .add_systems(FixedUpdate, handle_spawn_new_inhabitant);
    }
}
