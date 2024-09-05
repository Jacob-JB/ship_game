use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use common::player::*;
use controller::PlayerInput;

use crate::{entity_map::ServerEntityMapper, networking::prelude::*};

use super::{LocalPlayer, LocalPlayerBundle, PlayerBundle};

pub fn build(app: &mut App) {
    app.add_systems(Update, (create_new_players, send_state_updates));
}

fn create_new_players(
    mut commands: Commands,
    mut messages: MessageReceiver<NewPlayer>,
    mut map: ServerEntityMapper,
) {
    for NewPlayer {
        server_entity,
        username,
        local_player,
    } in messages.drain()
    {
        let player_entity = map.get_or_spawn(server_entity);

        if let Some(new_local_player) = local_player {
            commands
                .entity(player_entity)
                .insert(LocalPlayerBundle::new(username, new_local_player));
        } else {
            commands
                .entity(player_entity)
                .insert(PlayerBundle::new(username));
        }
    }
}

const PLAYER_STATE_UPDATE_INTERVAL: Duration = Duration::from_millis(150);

fn send_state_updates(
    player_q: Query<(&Position, &LinearVelocity, &PlayerInput), With<LocalPlayer>>,
    mut messages: MessageSender,
    message_id: Res<MessageId<ClientPlayerUpdate>>,
    mut last_update: Local<Duration>,
    time: Res<Time>,
) {
    if time.elapsed() > *last_update + PLAYER_STATE_UPDATE_INTERVAL {
        *last_update = time.elapsed();

        let Ok((&Position(position), &LinearVelocity(linear_velocity), &input)) =
            player_q.get_single()
        else {
            return;
        };

        messages.send(
            *message_id,
            &ClientPlayerUpdate {
                position,
                linear_velocity,
                input,
            },
        );
    }
}
