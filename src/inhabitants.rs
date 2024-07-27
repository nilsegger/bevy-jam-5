//! Inhabitants will be livings inside of the buildings

use std::time::Duration;

use bevy::{
    color::palettes::css::{DARK_GREEN, GOLD},
    prelude::*,
};
use rand::Rng;

use crate::{building::Building, player::Player};

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

/// Money will travel upwards to the side and then eventually fall down
#[derive(Component)]
struct MoneyVisual {
    /// Direction of travel velocity
    vel: Vec2,
    /// death timer
    death: Timer,
}

/// The timer when the inhabitant pays rent
#[derive(Component)]
struct RentTimer(Timer);

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
                RentTimer(Timer::from_seconds(10.0, TimerMode::Repeating)),
            ))
            .id();

        cmd.entity(*building_entity).add_child(inhabitant);
    }
}

/// ADds money to the player and spawn a money particle
fn handle_rent_timers(
    mut cmd: Commands,
    mut timers: Query<(&GlobalTransform, &mut RentTimer, &Parent)>,
    buildings: Query<(&Building, &GlobalTransform)>,
    time: Res<Time>,
    mut player: ResMut<Player>,
) {
    for (rent_global, mut rent_timer, parent) in timers.iter_mut() {
        let timer = &mut rent_timer.0;
        timer.tick(time.delta());

        if timer.just_finished() {
            let (building, building_global) = match buildings.get(parent.get()) {
                Ok(x) => x,
                Err(_) => continue,
            };

            // should probably be an event
            let rent = building_global.translation().y + building.size.x;
            player.money += rent as i32;

            let mut rng = rand::thread_rng();

            cmd.spawn((
                TransformBundle::from_transform(rent_global.compute_transform()),
                MoneyVisual {
                    vel: Vec2::new(rng.gen_range(-20..20) as f32, rng.gen_range(1..100) as f32)
                        .normalize(),
                    death: Timer::from_seconds(5.0, TimerMode::Once),
                },
            ));
        }
    }
}

/// Updates the money to continually fall more
fn update_money(
    mut cmd: Commands,
    mut moneys: Query<(Entity, &mut Transform, &mut MoneyVisual)>,
    time: Res<Time>,
) {
    for (money_entity, mut money_transform, mut money) in moneys.iter_mut() {
        money.death.tick(time.delta());

        if money.death.just_finished() {
            cmd.entity(money_entity).despawn_recursive();
            continue;
        }

        money.vel.y -= 0.5 * time.delta_seconds() * money.death.elapsed_secs();
        money_transform.rotation = Quat::from_euler(EulerRot::ZXY, money.vel.to_angle(), 0.0, 0.0);
        money_transform.translation += 100.0 * (money.vel * time.delta_seconds()).extend(0.0);
    }
}

/// draws green boxes for the money
fn draw_money(moneys: Query<(&GlobalTransform, &MoneyVisual)>, mut gizmos: Gizmos) {
    for (global, money) in moneys.iter() {
        if money.death.elapsed_secs() < 0.1 {
            // some weird center rendering bug otherwise
            continue;
        }

        gizmos.rect_2d(
            global.translation().xy(),
            global.right().xy().to_angle(),
            Vec2::new(10.0, 5.0),
            DARK_GREEN,
        );
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
            .add_systems(
                Update,
                (
                    move_inside_building,
                    outline_inhabitant,
                    update_money,
                    draw_money,
                ),
            )
            .add_systems(
                FixedUpdate,
                (handle_spawn_new_inhabitant, handle_rent_timers),
            );
    }
}
