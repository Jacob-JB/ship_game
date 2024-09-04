use avian3d::prelude::*;
use bevy::prelude::*;

pub mod networking;

pub fn build(app: &mut App) {
    app.add_systems(Update, debug_players);

    networking::build(app);
}

/// Marker component for a player on the client
#[derive(Component)]
pub struct Player;

fn debug_players(player_q: Query<&Position, With<Player>>) {
    for position in player_q.iter() {
        debug!("{:?}", position);
    }
}
