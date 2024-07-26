//! stuff related to the player like keeping track of money

use bevy::prelude::*;

/// Keeps track of the players money
#[derive(Resource)]
pub struct Player {
    /// the money of the player
    pub money: i32,
}

/// The label showing the current money
#[derive(Component)]
struct PlayerMoneyLabel;

/// Adds default entities
fn add_default_entities(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((
        PlayerMoneyLabel,
        TextBundle::from_section(
            "",
            TextStyle {
                font: asset_server.load("fonts/RobotoSlab.ttf"),
                font_size: 100.0,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Right)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        }),
    ));
}

/// Updates the label to the current amount of money
fn update_player_money_label(
    mut labels: Query<&mut Text, With<PlayerMoneyLabel>>,
    player: Res<Player>,
) {
    if labels.is_empty() {
        return;
    }

    let mut label = labels.single_mut();
    label.sections[0].value = player.money.to_string() + "$";
}

/// The player plugin
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Player { money: 10000 })
            .add_systems(Startup, add_default_entities)
            .add_systems(Update, update_player_money_label);
    }
}
