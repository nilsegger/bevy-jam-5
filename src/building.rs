//! Everything related to placing new buildings

use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{GREEN, RED, WHITE_SMOKE},
    prelude::*,
};
use rand::prelude::*;

use crate::layers::*;

/// The Building component
#[derive(Component)]
struct Building;

/// The Building which will be shown when a possible slot was found
#[derive(Component)]
struct PreviewBuilding {
    /// true if cursor is close enough to put the building anywhere
    visible: bool,
    /// flag if there is a building beneath
    bottom_support: bool,
    /// blocked if a neighbor building is close, but there is something blocking the place...
    blocked: bool,
    /// the size the building will take up
    size: Vec2,
}

/// A sensor that checks that there is another building below the preview building
#[derive(Component)]
struct PreviewBuildingBottomSupportSensor;

/// Epsilon is used to inset PreviewBuilding collider checker
const PREVIEW_BUILDING_EPS: f32 = 0.02;

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
    /// Correct layers
    layers: CollisionLayers,
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
        cursor_builder_layers(),
    ));

    let pb = cmd
        .spawn((
            PreviewBuilding {
                visible: false,
                bottom_support: false,
                blocked: false,
                size: Vec2 { x: 100.0, y: 60.0 },
            },
            TransformBundle::IDENTITY,
            Sensor,
            RigidBody::Kinematic, // NOTE: same again, must it be kinematic??
            Collider::rectangle(100.0 - PREVIEW_BUILDING_EPS, 60.0 - PREVIEW_BUILDING_EPS),
            preview_building_layers(),
        ))
        .id();

    let pb_bottom_support_sensor = cmd
        .spawn((
            PreviewBuildingBottomSupportSensor,
            TransformBundle::from_transform(Transform::from_xyz(0.0, -40.0, 0.0)),
            Sensor,
            RigidBody::Kinematic,
            Collider::rectangle(90.0, 10.0),
            preview_building_layers(), // NOTE: assumptions that same layers dont automatically collide
        ))
        .id();

    cmd.entity(pb).add_child(pb_bottom_support_sensor);

    cmd.spawn(BuildingBundle {
        transform: TransformBundle::IDENTITY,
        building: Building,
        rigidbody: RigidBody::Static,
        collider: Collider::rectangle(100.0, 60.0),
        layers: building_layers(),
    });
}

/// Sets the position of the CursorBuilder to the cursors position
/// Also draws a debug circle
fn update_cursor_builder(
    mut cursor_builders: Query<&mut Transform, With<CursorBuilder>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
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
}

/// Checks if the event was called in a place where there is a possible slot nearby
fn update_preview_building(
    builders: Query<(&GlobalTransform, &CollidingEntities), With<CursorBuilder>>,
    mut preview_buildings: Query<(&mut PreviewBuilding, &mut Transform, &CollidingEntities)>,
    bottom_support_sensors: Query<&CollidingEntities, With<PreviewBuildingBottomSupportSensor>>,
    buildings: Query<(&GlobalTransform, &Collider, &ColliderAabb), With<Building>>,
) {
    if builders.is_empty() || preview_buildings.is_empty() {
        return;
    }

    let (builder_transform, builder_collisions) = builders.single();
    let (mut preview_building, mut pb_transform, pb_colliding_entities) =
        preview_buildings.single_mut();

    let closest_building =
        match find_building_closest_to_cursor(builder_collisions, &buildings, builder_transform) {
            Some(x) => x,
            None => {
                preview_building.visible = false;
                return;
            }
        };

    let builder_pos = builder_transform.translation().xy();

    let (closest_building_transform, _, closest_building_aabb) =
        buildings.get(closest_building).unwrap();

    if builder_pos.y >= closest_building_aabb.min.y && builder_pos.y <= closest_building_aabb.max.y
    {
        preview_building.visible = true;

        // same level
        if builder_pos.x < closest_building_transform.translation().x {
            // left
            pb_transform.translation = Vec2::new(
                closest_building_aabb.min.x - preview_building.size.x / 2.0,
                closest_building_transform.translation().y,
            )
            .extend(0.0);
        } else {
            // right
            pb_transform.translation = Vec2::new(
                closest_building_aabb.max.x + preview_building.size.x / 2.0,
                closest_building_transform.translation().y,
            )
            .extend(0.0);
        }
    } else if builder_pos.y >= closest_building_aabb.max.y {
        preview_building.visible = true;
        pb_transform.translation = Vec2::new(
            builder_transform.translation().x,
            closest_building_aabb.max.y + preview_building.size.y / 2.0,
        )
        .extend(0.0);
    } else {
        preview_building.visible = false;
    }
    // NOTE: no case for putting beneath

    // Checks if found spot is free
    if pb_colliding_entities.is_empty() {
        // Free to place building
        preview_building.blocked = false;
    } else {
        preview_building.blocked = true;
    }

    // Checks if found spot has a building beneath
    let bottom_support_sensor = bottom_support_sensors.single();
    preview_building.bottom_support =
        !bottom_support_sensor.is_empty() || pb_transform.translation.y < 1.0;
}

/// Checks if left mouse button was pressed and preview building is visible
fn maybe_send_place_building_event(
    preview_buildings: Query<&PreviewBuilding>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut events: EventWriter<PlaceBuildingEvent>,
) {
    if preview_buildings.is_empty() {
        return;
    }

    let preview_building = preview_buildings.single();

    if mouse.just_released(MouseButton::Left)
        && preview_building.visible
        && !preview_building.blocked
        && preview_building.bottom_support
    {
        dbg!(events.send(PlaceBuildingEvent));
    }
}

/// realise the preview building into a real building
fn handle_place_building_event(
    mut cmd: Commands,
    mut events: EventReader<PlaceBuildingEvent>,
    mut preview_buildings: Query<(Entity, &mut PreviewBuilding, &GlobalTransform)>,
    pb_bottom_support_sensors: Query<Entity, With<PreviewBuildingBottomSupportSensor>>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();

    let (pb_entity, mut preview_building, transform) = preview_buildings.single_mut();

    cmd.spawn(BuildingBundle {
        building: Building,
        collider: Collider::rectangle(preview_building.size.x, preview_building.size.y),
        rigidbody: RigidBody::Static,
        transform: TransformBundle::from_transform(transform.compute_transform()),
        layers: building_layers(),
    });

    let mut rng = rand::thread_rng();
    preview_building.size.x = rng.gen_range(80..=100) as f32;

    cmd.entity(pb_entity).insert(Collider::rectangle(
        preview_building.size.x - PREVIEW_BUILDING_EPS,
        preview_building.size.y - PREVIEW_BUILDING_EPS,
    ));

    cmd.entity(pb_bottom_support_sensors.single())
        .insert(Collider::rectangle(0.9 * preview_building.size.x, 10.0));
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
        pb.size,
        if pb.blocked { RED } else { GREEN },
    );

    gizmos.arrow_2d(
        transform.translation().xy(),
        transform.translation().xy() - Vec2::Y * 25.0,
        if !pb.bottom_support { RED } else { GREEN },
    );
}

/// Finds building which is clsoest to the cursor
fn find_building_closest_to_cursor(
    builder_collisions: &CollidingEntities,
    building_colliders: &Query<(&GlobalTransform, &Collider, &ColliderAabb), With<Building>>,
    builder_transform: &GlobalTransform,
) -> Option<Entity> {
    let mut same_row = false;
    let mut min_distance = f32::MAX;

    let mut closest_building: Option<Entity> = None;

    for collision_entity in builder_collisions.iter() {
        let (building_transform, building_collider, building_aabb) =
            match building_colliders.get(*collision_entity) {
                Ok(x) => x,
                Err(_) => continue,
            };

        let building_xy = building_transform.translation().xy();

        let distance_to_point = building_collider.distance_to_point(
            building_xy,
            0.0,
            builder_transform.translation().xy(),
            true,
        );

        let building_same_row = building_aabb.max.y >= builder_transform.translation().y
            && building_aabb.min.y <= builder_transform.translation().y;

        // NOTE: setup to favour buildings in the same "row" / "level"
        if !building_same_row && same_row {
            continue;
        }

        if building_same_row && !same_row {
            same_row = true;
            min_distance = distance_to_point;
            closest_building = Some(*collision_entity);
        } else if distance_to_point < min_distance {
            min_distance = distance_to_point;
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
                    (
                        update_preview_building,
                        display_preview_building,
                        maybe_send_place_building_event,
                    )
                        .chain(),
                ),
            )
            .add_systems(FixedUpdate, handle_place_building_event);
    }
}
