//! Everything related to placing new buildings

use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{GREY, RED, WHITE_SMOKE},
    prelude::*,
};

/// The Building component
#[derive(Component)]
struct Building;

/// The Building which will be shown when a possible slot was found
#[derive(Component)]
struct PreviewBuilding {
    /// if slot was found
    visible: bool,
}

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

    cmd.spawn((
        PreviewBuilding { visible: false },
        TransformBundle::IDENTITY,
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
fn update_preview_building(
    builders: Query<(&GlobalTransform, &CollidingEntities), With<CursorBuilder>>,
    mut preview_buildings: Query<(&mut PreviewBuilding, &mut Transform)>,
    buildings: Query<(&GlobalTransform, &ColliderAabb)>,
    building_colliders: Query<&Collider, With<Building>>,
) {
    if builders.is_empty() || preview_buildings.is_empty() {
        return;
    }

    let (builder_transform, builder_collisions) = builders.single();
    let (mut preview_building, mut pb_transform) = preview_buildings.single_mut();

    let closest_building = match find_building_closest_to_cursor(
        builder_collisions,
        building_colliders,
        builder_transform,
    ) {
        Some(x) => x,
        None => {
            preview_building.visible = false;
            return;
        }
    };

    let builder_pos = builder_transform.translation().xy();

    let (closest_building_transform, closest_building_aabb) =
        buildings.get(closest_building).unwrap();

    if builder_pos.y >= closest_building_aabb.min.y && builder_pos.y <= closest_building_aabb.max.y
    {
        // Put left or right
        preview_building.visible = true;
        pb_transform.translation =
            Vec2::new(builder_pos.x, closest_building_transform.translation().y).extend(0.0);
    } else if builder_pos.y >= closest_building_aabb.max.y {
        preview_building.visible = true;
        pb_transform.translation = Vec2::new(
            builder_transform.translation().x,
            closest_building_aabb.max.y + 30.0, // HACK: dont hardcode building height
        )
        .extend(0.0);
    } else {
        preview_building.visible = false;
    }
    // NOTE: no case for putting beneath
}

/// draws outline of preview building
fn display_preview_building(
    preview_buildings: Query<(&PreviewBuilding, &GlobalTransform)>,
    mut gizmos: Gizmos,
) {
    let (pb, transform) = preview_buildings.single();

    if !pb.visible {
        return;
    }
    gizmos.rect_2d(
        transform.translation().xy(),
        0.0,
        Vec2::new(100.0, 60.0),
        GREY,
    );
}

/// Finds building which is clsoest to the cursor
fn find_building_closest_to_cursor(
    builder_collisions: &CollidingEntities,
    building_colliders: Query<&Collider, With<Building>>,
    builder_transform: &GlobalTransform,
) -> Option<Entity> {
    let min_distance = f32::MAX;
    let mut closest_building: Option<Entity> = None;

    for collision_entity in builder_collisions.iter() {
        let building_collider = match building_colliders.get(*collision_entity) {
            Ok(x) => x,
            Err(_) => continue,
        };

        let distance_to_point = building_collider.distance_to_point(
            Vec2::ZERO,
            0.0,
            builder_transform.translation().xy(),
            true,
        );

        if distance_to_point < min_distance {
            closest_building = Some(*collision_entity);
        }
    }
    closest_building
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
                (
                    update_cursor_builder,
                    (update_preview_building, display_preview_building).chain(),
                ),
            );
    }
}
