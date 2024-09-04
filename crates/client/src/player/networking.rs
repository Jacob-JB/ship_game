use avian3d::prelude::*;
use bevy::prelude::*;
use common::player::*;

use crate::{
    entity_map::ServerEntityMapper, networking::prelude::*, physics::playout::SnapshotInterpolation,
};

use super::Player;

pub fn build(app: &mut App) {
    app.add_systems(Update, create_new_players);
}

fn create_new_players(
    mut commands: Commands,
    mut messages: MessageReceiver<NewPlayer>,
    mut map: ServerEntityMapper,
) {
    for NewPlayer { server_entity } in messages.drain() {
        let player_entity = map.get_or_spawn(server_entity);

        commands.entity(player_entity).insert((
            Player,
            SnapshotInterpolation::default(),
            Position::default(),
            Rotation::default(),
        ));
    }
}
