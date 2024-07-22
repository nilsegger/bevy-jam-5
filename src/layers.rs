//! used as global entry for physics layers

use avian2d::prelude::{CollisionLayers, PhysicsLayer};

/// The physics layers required in the game
#[derive(PhysicsLayer)]
enum Layers {
    /// Any placed building
    Building,
    /// The preview building
    PreviewBuilding,
    /// Used for updating the preview building
    Cursor,
}

/// layers required by Building
pub fn building_layers() -> CollisionLayers {
    CollisionLayers::new(Layers::Building, [Layers::PreviewBuilding, Layers::Cursor])
}

/// layers required by Cursor Builder
pub fn cursor_builder_layers() -> CollisionLayers {
    CollisionLayers::new(Layers::Cursor, [Layers::Building])
}

/// layers required by Preview Building
pub fn preview_building_layers() -> CollisionLayers {
    CollisionLayers::new(Layers::PreviewBuilding, [Layers::Building])
}
