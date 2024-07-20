//! Everything related to placing new buildings

use bevy::{color::palettes::css::WHITE_SMOKE, prelude::*};

/// The Building component
#[derive(Component)]
struct Building;

/// Try to place a building
#[derive(Event)]
struct PlaceBuildingEvent {
    /// Where to place the building
    position: Vec2,
}

/// Adds the starting building for the tower
fn add_default_building(mut cmd: Commands) {
    cmd.spawn((Building, TransformBundle::IDENTITY));
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
        app.add_systems(Startup, add_default_building)
            .add_systems(Update, outline_buildings_system);
    }
}
