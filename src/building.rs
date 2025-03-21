//! Everything related to placing new buildings

// TODO: buildings should be gradually built, from bottom to top
// TODO: joints should also be placeable?

use avian2d::prelude::*;
use bevy::{
    audio::Volume,
    color::palettes::css::{
        BLACK, DARK_GRAY, DARK_GREY, DARK_SLATE_BLUE, GREEN, ORANGE, PURPLE, RED, WHITE,
        WHITE_SMOKE,
    },
    math::NormedVectorSpace,
    prelude::*,
};

use rand::prelude::*;
use rand::seq::SliceRandom;

use crate::{inhabitants::SpawnNewInhabitant, layers::*, player::Player};

/// The kind of operations supported
enum BuildOps {
    /// Build a new building
    Building,
    /// Build a new joint
    Joint,
}

/// Saves what operation is currently selected
#[derive(Resource)]
struct SelectedBuildOps {
    /// the currently selected operation
    selected: BuildOps,
}

/// The Building component
#[derive(Component)]
pub struct Building {
    /// size of the building
    pub size: Vec2,
}

/// A breakable joint keeping buildings together
#[derive(Component)]
struct BuildingJoint;

/// A breakable joint keeping buildings together, but its preview
#[derive(Component)]
struct BuildingJointPreview {
    /// from entity
    entity_start: Option<Entity>,
    /// local start will be used as anchor
    local_start: Vec2,
    /// current length
    length: f32,
}

/// Types of buildings
#[derive(Debug)]
enum BuildingVariants {
    /// Flat roof
    Default,
    /// building with a chimney and its offset
    Chimney(Vec2),
}

/// Chimney component
#[derive(Component)]
struct Chimney;

/// The Building which will be shown when a possible slot was found
#[derive(Component)]
struct PreviewBuilding {
    /// The variant that will be spawned
    variant: BuildingVariants,
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
    /// building properties
    building: Building,
    /// should be static for now
    rigidbody: RigidBody,
    /// should be Collider::rect
    collider: Collider,
    /// Correct layers
    layers: CollisionLayers,
    /// Sprite bundle, contains transform
    sprite: SpriteBundle,
}

/// The component which will be attached to the cursor,
/// then when a building wants to be placed, its sensor is checked for possible locations
#[derive(Component)]
struct CursorBuilder;

/// Try to place a building
#[derive(Event)]
struct PlaceBuildingEvent;

/// Text that will show if player has enough money
#[derive(Component)]
struct CursorBuildText;

/// Adds the starting building for the tower
/// Adds the CursorBuilder
fn add_default_entities(
    mut cmd: Commands,
    mut inhabitants: EventWriter<SpawnNewInhabitant>,
    asset_server: Res<AssetServer>,
) {
    cmd.spawn((
        CursorBuildText,
        Text2dBundle {
            text: Text::from_section(
                "Build",
                TextStyle {
                    font: asset_server.load("fonts/RobotoSlab.ttf"),
                    font_size: 52.0,
                    color: WHITE.into(),
                },
            )
            .with_justify(JustifyText::Left),
            ..default()
        },
    ));

    cmd.spawn((
        CursorBuildText,
        Text2dBundle {
            text: Text::from_section(
                "Build",
                TextStyle {
                    font: asset_server.load("fonts/RobotoSlab.ttf"),
                    font_size: 50.0,
                    color: PURPLE.into(),
                },
            )
            .with_justify(JustifyText::Left),
            ..default()
        },
    ));

    cmd.spawn((
        CursorBuilder,
        TransformBundle::IDENTITY,
        Sensor,
        Collider::circle(50.0),
        cursor_builder_layers(),
    ));

    let pb = cmd
        .spawn((
            PreviewBuilding {
                variant: BuildingVariants::Default,
                visible: false,
                bottom_support: false,
                blocked: false,
                size: Vec2 { x: 100.0, y: 60.0 },
            },
            TransformBundle::IDENTITY,
            Sensor,
            Collider::rectangle(100.0 - PREVIEW_BUILDING_EPS, 60.0 - PREVIEW_BUILDING_EPS),
            preview_building_layers(),
        ))
        .id();

    let pb_bottom_support_sensor = cmd
        .spawn((
            PreviewBuildingBottomSupportSensor,
            TransformBundle::from_transform(Transform::from_xyz(0.0, -40.0, 0.0)),
            Sensor,
            Collider::rectangle(90.0, 10.0),
            preview_building_layers(), // NOTE: assumptions that same layers dont automatically collide
        ))
        .id();

    cmd.entity(pb).add_child(pb_bottom_support_sensor);

    cmd.spawn(BuildingJointPreview {
        entity_start: None,
        local_start: Vec2::ZERO,
        length: 0.0,
    });

    let building = cmd
        .spawn(BuildingBundle {
            building: Building {
                size: Vec2::new(100.0, 60.0),
            },
            rigidbody: RigidBody::Dynamic,
            collider: Collider::rectangle(100.0, 60.0),
            layers: building_layers(),
            sprite: SpriteBundle {
                texture: asset_server.load("apartmentPINK.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(100.0, 60.0)),
                    ..default()
                },
                ..default()
            },
        })
        .id();

    inhabitants.send(SpawnNewInhabitant(building));

    cmd.spawn((TextBundle::from_section(
        "Build Tool (B)",
        TextStyle {
            font: asset_server.load("fonts/RobotoSlab.ttf"),
            font_size: 42.0,
            color: BLACK.into(),
        },
    )
    .with_text_justify(JustifyText::Center)
    .with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(70.0),
        left: Val::Px(3.0),
        ..default()
    }),));

    cmd.spawn((TextBundle::from_section(
        "Build Tool (B)",
        TextStyle {
            font: asset_server.load("fonts/RobotoSlab.ttf"),
            font_size: 40.0,
            color: WHITE.into(),
        },
    )
    .with_text_justify(JustifyText::Center)
    .with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(70.0),
        left: Val::Px(5.0),
        ..default()
    }),));

    cmd.spawn((TextBundle::from_section(
        "Support Joint Tool (J)",
        TextStyle {
            font: asset_server.load("fonts/RobotoSlab.ttf"),
            font_size: 42.0,
            color: BLACK.into(),
        },
    )
    .with_text_justify(JustifyText::Center)
    .with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(25.0),
        left: Val::Px(1.0),
        ..default()
    }),));

    cmd.spawn((TextBundle::from_section(
        "Support Joint Tool (J)",
        TextStyle {
            font: asset_server.load("fonts/RobotoSlab.ttf"),
            font_size: 40.0,
            color: WHITE.into(),
        },
    )
    .with_text_justify(JustifyText::Center)
    .with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(25.0),
        left: Val::Px(5.0),
        ..default()
    }),));
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

/// update how much money a building costs
fn update_cursor_text(
    mut texts: Query<(&mut Transform, &mut Text), With<CursorBuildText>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    player: Res<Player>,
    ops: Res<SelectedBuildOps>,
    joint_preview: Query<&BuildingJointPreview>,
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

    for (mut transform, mut text) in texts.iter_mut() {
        transform.translation = (cursor_world_position + Vec2::new(60.0, -40.0)).extend(10.0);

        if matches!(ops.selected, BuildOps::Building) {
            if player.money < 500 {
                text.sections[0].value = "Requires 500$".to_string();
            } else {
                text.sections[0].value = "".to_string();
            }
        } else {
            let preview = joint_preview.single();

            if preview.length < 0.1 {
                text.sections[0].value = "".to_string();
            } else if player.money < preview.length as i64 * 3 {
                text.sections[0].value = format!("Requires {}$", preview.length as i32 * 3);
            } else {
                text.sections[0].value = format!("{}$", preview.length as i32 * 3);
            }
        }
    }
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

    let builder_pos = builder_transform.translation().xy();

    let closest_building =
        match find_building_closest_to_cursor(builder_collisions, &buildings, builder_transform) {
            Some(x) => x,
            None => {
                preview_building.visible = false;
                pb_transform.translation = builder_pos.extend(0.0);
                return;
            }
        };

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
    mut inhabitants: EventWriter<SpawnNewInhabitant>,
    mut player: ResMut<Player>,
    assets: Res<AssetServer>,
) {
    if events.is_empty() || player.money < 500 {
        return;
    }
    events.clear();

    player.money -= 500;

    let (pb_entity, mut preview_building, transform) = preview_buildings.single_mut();

    let apartments = [
        "apartmentBLUE.png",
        "apartmentBLUE2.png",
        "apartmentDARK.png",
        "apartmentGREEN.png",
        "apartmentLIGHT.png",
        "apartmentPINK.png",
        "apartmentYELLOW.png",
    ];

    let mut rng = rand::thread_rng();

    let building = cmd
        .spawn(BuildingBundle {
            building: Building {
                size: preview_building.size,
            },
            collider: Collider::rectangle(preview_building.size.x, preview_building.size.y),
            rigidbody: RigidBody::Dynamic,
            layers: building_layers(),
            sprite: SpriteBundle {
                texture: assets.load(apartments.choose(&mut rng).unwrap().to_string()),
                sprite: Sprite {
                    custom_size: Some(preview_building.size),
                    ..default()
                },
                transform: transform.compute_transform(),
                ..default()
            },
        })
        .id();

    cmd.spawn(AudioBundle {
        source: assets.load("build.ogg"),
        settings: PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: Volume::new(1.0),
            ..default()
        },
    });

    match preview_building.variant {
        BuildingVariants::Default => (),
        BuildingVariants::Chimney(offset) => {
            let chimney = cmd
                .spawn((
                    Chimney,
                    Sensor,
                    Collider::rectangle(20.0, 30.0), // HACK: remove hardcoded
                    chimney_layers(),
                    SpriteBundle {
                        texture: assets.load("chimney.png"),
                        transform: Transform::from_translation(offset.extend(0.0)),
                        ..default()
                    },
                ))
                .id();
            cmd.entity(building).add_child(chimney);
        }
    }

    inhabitants.send(SpawnNewInhabitant(building));

    // Create new preview building
    let mut rng = rand::thread_rng();
    preview_building.size.x = rng.gen_range(80..=100) as f32;

    cmd.entity(pb_bottom_support_sensors.single())
        .insert(Collider::rectangle(0.9 * preview_building.size.x, 10.0));

    let variants = [
        BuildingVariants::Default,
        BuildingVariants::Default,
        BuildingVariants::Default,
        BuildingVariants::Default,
        BuildingVariants::Chimney(Vec2::ZERO),
    ];

    let new_variant = variants.choose(&mut rng).unwrap();

    dbg!(new_variant);

    let default_collider = Collider::rectangle(
        preview_building.size.x - PREVIEW_BUILDING_EPS,
        preview_building.size.y - PREVIEW_BUILDING_EPS,
    );

    match new_variant {
        BuildingVariants::Default => {
            preview_building.variant = BuildingVariants::Default;
            cmd.entity(pb_entity).insert(default_collider);
        }
        BuildingVariants::Chimney(_) => {
            let offset = 0.4 * preview_building.size.x;
            let x_offset = rng.gen_range(-offset..=offset) as f32;
            let chimney_collider = Collider::rectangle(20.0, 30.0);
            let chimney_offset = Vec2::new(x_offset, 30.0 + 20.0);
            preview_building.variant = BuildingVariants::Chimney(chimney_offset);
            cmd.entity(pb_entity).insert(Collider::compound(vec![
                (Vec2::ZERO, 0.0, default_collider),
                (chimney_offset, 0.0, chimney_collider),
            ]));
        }
    }
}

/// draws outline of preview building
fn display_preview_building(
    preview_buildings: Query<(&PreviewBuilding, &GlobalTransform)>,
    mut gizmos: Gizmos,
) {
    let (pb, transform) = preview_buildings.single();

    match pb.variant {
        BuildingVariants::Chimney(offset) => {
            gizmos.rect_2d(
                transform.translation().xy() + offset,
                0.0,
                Vec2::new(20.0, 30.0),
                if !pb.visible || pb.blocked {
                    RED
                } else {
                    GREEN
                },
            );
        }
        BuildingVariants::Default => (),
    };

    gizmos.rect_2d(
        transform.translation().xy(),
        0.0,
        pb.size,
        if !pb.visible || pb.blocked {
            RED
        } else {
            GREEN
        },
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

/// Update what build op is selected by keyboard shortcuts
fn update_selected_build_op(
    keys: Res<ButtonInput<KeyCode>>,
    mut build_ops: ResMut<SelectedBuildOps>,
    mut joint_preview: Query<&mut BuildingJointPreview>,
) {
    if keys.just_released(KeyCode::KeyB) {
        build_ops.selected = BuildOps::Building;

        joint_preview.single_mut().entity_start = None;
    } else if keys.just_released(KeyCode::KeyJ) {
        build_ops.selected = BuildOps::Joint;
    }
}

/// Checks if a building was clicked -> then either set it as start point or send event to create
/// new joint
fn check_building_clicked_for_joint(
    mut cmd: Commands,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    spatial_query: SpatialQuery,
    mut previews: Query<&mut BuildingJointPreview>,
    transforms: Query<&GlobalTransform>,
    mut player: ResMut<Player>,
    assets: Res<AssetServer>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

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

    if let Some(projected) = spatial_query.project_point(
        cursor_world_position,
        true,
        SpatialQueryFilter::from_mask(Layers::Building),
    ) {
        let mut preview = previews.single_mut();

        if !projected.is_inside {
            // NOTE: should this clear the previous?
            preview.entity_start = None;
        } else {
            let transform = match transforms.get(projected.entity) {
                Ok(x) => x,
                Err(_) => return, // NOTE: silend failing...
            };

            // NOTE: assuming no rotation....
            // TODO: rotate
            let mut local_offset = projected.point - transform.translation().xy();
            let rotation = Vec2::from_angle(-transform.right().xy().to_angle());
            local_offset = rotation.rotate(local_offset);

            if preview.entity_start.is_none() {
                preview.entity_start = Some(projected.entity);
                preview.local_start = local_offset;
            } else {
                let transform_start = match transforms.get(preview.entity_start.unwrap()) {
                    Ok(x) => x,
                    Err(_) => return, // NOTE: silend failing...
                };

                let cost =
                    (transform_start.translation().xy().distance(projected.point) * 3.0) as i64;
                if cost > player.money {
                    return;
                } else {
                    player.money -= cost;
                }

                // preview.entity_end = Some(projected.entity);
                // preview.local_end = local_offset;
                //
                let start_point = transform_start.translation().xy()
                    + transform_start.right().xy().rotate(preview.local_start);

                // probably should have been done with an event...
                cmd.spawn((
                    BuildingJoint,
                    DistanceJoint::new(preview.entity_start.unwrap(), projected.entity)
                        .with_local_anchor_1(preview.local_start)
                        .with_local_anchor_2(local_offset)
                        .with_rest_length(start_point.distance(projected.point)),
                ));

                cmd.spawn(AudioBundle {
                    source: assets.load("build.ogg"),
                    settings: PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Despawn,
                        volume: Volume::new(0.8),
                        ..default()
                    },
                });

                preview.entity_start = None;
            }
        }
    }
}

/// Debug outline of buildings
fn outline_buildings_system(buildings: Query<(&Building, &GlobalTransform)>, mut gizmos: Gizmos) {
    for (building, transform) in &buildings {
        let dir = transform.right();
        let angle = dir.y.atan2(dir.x);

        gizmos.rect_2d(
            transform.translation().xy(),
            angle,
            building.size,
            DARK_SLATE_BLUE,
        );
    }
}

/// Debug outline of buildings
fn outline_chimneys_system(chimneys: Query<&GlobalTransform, With<Chimney>>, mut gizmos: Gizmos) {
    for transform in &chimneys {
        let dir = transform.right();
        let angle = dir.y.atan2(dir.x);

        gizmos.rect_2d(
            transform.translation().xy(),
            angle,
            Vec2::new(20.0, 30.0),
            DARK_GRAY,
        );
    }
}

/// display preview joints
/// NOTE: this currently also sets the length of the joint...
fn display_preview_joint(
    mut previews: Query<&mut BuildingJointPreview>,
    transforms: Query<&GlobalTransform>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    spatial_query: SpatialQuery,
    mut gizmos: Gizmos,
) {
    let mut preview = previews.single_mut();

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

    if let Some(e) = preview.entity_start {
        let transform = match transforms.get(e) {
            Ok(x) => x,
            Err(_) => return,
        };

        let point =
            transform.translation().xy() + transform.right().xy().rotate(preview.local_start);

        gizmos.circle_2d(point, 8.0, PURPLE);

        gizmos.arrow_2d(point, cursor_world_position, WHITE);

        preview.length = cursor_world_position.distance(point);
    } else {
        preview.length = 0.0;
    }

    let targeting_building = if let Some(projected) = spatial_query.project_point(
        cursor_world_position,
        true,
        SpatialQueryFilter::from_mask(Layers::Building),
    ) {
        projected.is_inside
    } else {
        false
    };

    gizmos.circle_2d(
        cursor_world_position,
        8.0,
        if targeting_building {
            GREEN
        } else if preview.entity_start.is_some() {
            ORANGE
        } else {
            RED
        },
    );
}

/// display joints
fn display_joints(
    joints: Query<&DistanceJoint, With<BuildingJoint>>,
    transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    for joint in &joints {
        let t1 = match transforms.get(joint.entity1) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let t2 = match transforms.get(joint.entity2) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let point1 = t1.translation().xy() + t1.right().xy().rotate(joint.local_anchor_1());
        let point2 = t2.translation().xy() + t2.right().xy().rotate(joint.local_anchor_2());

        gizmos.circle_2d(point1, 8.0, WHITE);
        gizmos.circle_2d(point1, 7.0, PURPLE);
        gizmos.circle_2d(point2, 8.0, WHITE);
        gizmos.circle_2d(point2, 7.0, PURPLE);

        gizmos.line_2d(point1, point2, WHITE);
        gizmos.line_2d(
            point1 + Vec2::new(1.0, -1.0),
            point2 + Vec2::new(1.0, -1.0),
            PURPLE,
        );
    }
}

/// Used to only run systems when currently building op is selected
fn only_for_building_op(build_op: Res<SelectedBuildOps>) -> bool {
    matches!(build_op.selected, BuildOps::Building)
}

/// Used to only run systems when currently building op is selected
fn only_for_joint_op(build_op: Res<SelectedBuildOps>) -> bool {
    matches!(build_op.selected, BuildOps::Joint)
}

/// Plugin for everything related to buildings
pub struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaceBuildingEvent>()
            .insert_resource(SelectedBuildOps {
                selected: BuildOps::Building,
            })
            .add_systems(Startup, add_default_entities)
            .add_systems(
                Update,
                (
                    update_selected_build_op,
                    (
                        update_cursor_builder,
                        (
                            update_preview_building,
                            display_preview_building,
                            maybe_send_place_building_event,
                        )
                            .chain(),
                    )
                        .run_if(only_for_building_op),
                    outline_buildings_system,
                    // outline_chimneys_system,
                    display_preview_joint.run_if(only_for_joint_op),
                    display_joints,
                    update_cursor_text,
                ),
            )
            .add_systems(
                FixedUpdate,
                check_building_clicked_for_joint.run_if(only_for_joint_op),
            )
            .add_systems(FixedUpdate, handle_place_building_event);
    }
}
