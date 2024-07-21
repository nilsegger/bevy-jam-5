//! Everything related to placing new buildings

use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{RED, WHITE_SMOKE},
    prelude::*,
};

/// The Building component
#[derive(Component)]
struct Building;

/// Bundles all important components for a building
#[derive(Bundle)]
struct BuildingBundle {
    /// the transform
    transform: TransformBundle,
    /// building properties
    building: Building,
    /// should be static for now
    rigidbody: RigidBody,
    /// should be Collider::rect
    collider: Collider,
}

/// The component which will be attached to the cursor,
/// then when a building wants to be placed, its sensor is checked for possible locations
#[derive(Component)]
struct CursorBuilder;

/// Try to place a building
#[derive(Event)]
struct PlaceBuildingEvent;

/// Adds the starting building for the tower
/// Adds the CursorBuilder
fn add_default_entities(mut cmd: Commands) {
    cmd.spawn((
        CursorBuilder,
        TransformBundle::IDENTITY,
        RigidBody::Kinematic, // NOTE: should it really by static if it gets moved?
        Sensor,
        Collider::circle(50.0),
    ));

    cmd.spawn(BuildingBundle {
        transform: TransformBundle::IDENTITY,
        building: Building,
        rigidbody: RigidBody::Static,
        collider: Collider::rectangle(100.0, 60.0),
    });
}

/// Sets the position of the CursorBuilder to the cursors position
/// Also draws a debug circle
fn update_cursor_builder(
    mut cursor_builders: Query<&mut Transform, With<CursorBuilder>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut placings: EventWriter<PlaceBuildingEvent>, // mut gizmos: Gizmos,
) {
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

    let mut builder_transform = cursor_builders.single_mut();
    builder_transform.translation = cursor_world_position.extend(0.0);

    if mouse.just_released(MouseButton::Left) {
        placings.send(PlaceBuildingEvent);
    }

    // gizmos.circle_2d(builder_transform.translation.xy(), 50.0, RED);
}

/// Checks if the event was called in a place where there is a possible slot nearby
fn handle_place_building_events(
    events: EventReader<PlaceBuildingEvent>,
    builders: Query<(&GlobalTransform, &CollidingEntities), With<CursorBuilder>>,
    buildings: Query<(&GlobalTransform, &ColliderAabb), With<Building>>,
) {
    // there is no info in the event, so it only matters if there is one
    if events.is_empty() {
        return;
    }

    let (builder_transform, builder_collisions) = builders.single();

    for collision_entity in builder_collisions.iter() {
        // TODO: find the best slot

        let (building_transform, building_aabb) = match buildings.get(*collision_entity) {
            Ok(x) => x,
            Err(_) => continue,
        };

        // TODO: check if on same level as cursor builder
        dbg!(building_transform, building_aabb);

        let cursor_x = builder_transform.translation().x;

        // NOTE: assuming that buildings cant be rotated...
        dbg!((building_aabb.min.x - cursor_x).abs());
        dbg!((building_aabb.max.x - cursor_x).abs());
        if (building_aabb.min.x - cursor_x).abs() < (building_aabb.max.x - cursor_x).abs() {
            // TODO: put building left if it is the correct building!
            dbg!("Left");
        } else {
            dbg!("Right");
        }
    }
}

/// Debug outline of buildings
fn outline_buildings_system(
    buildings: Query<&GlobalTransform, With<Building>>,
    mut gizmos: Gizmos,
) {
    for transform in &buildings {
        gizmos.rect_2d(
            transform.translation().xz(),
            Rot2::IDENTITY,
            Vec2::new(100.0, 60.0),
            WHITE_SMOKE,
        );
    }
}

/// Plugin for everything related to buildings
pub struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaceBuildingEvent>()
            .add_systems(Startup, add_default_entities)
            .add_systems(
                Update,
                (update_cursor_builder, handle_place_building_events),
            );
    }
}
