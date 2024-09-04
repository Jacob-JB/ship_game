use bevy::prelude::*;
use common::state::JoinRequest;
use nevy::prelude::ReceivedMessages;

use crate::{networking::ClientConnection, state::ReceiveGameUpdates};

pub fn build(app: &mut App) {
    app.add_systems(Update, process_join_requests);
}

/// marker component for a player entity
///
/// the player entity is separate from the
/// client entity and can exist without one
#[derive(Component)]
pub struct Player {
    pub username: String,
}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
}

/// exists on a [Player] entity and points to it's client entity
///
/// if a [Player] entity doesn't have this component then there is no
/// connected client
///
/// there is a matching [ConnectedPlayer] on the client entity
#[derive(Component)]
pub struct ConnectedClient {
    pub client_entity: Entity,
}

/// points to the [Player] entity if this client has one
///
/// there is a matching [ConnectedClient] on the player entity
#[derive(Component)]
pub struct ConnectedPlayer {
    pub player_entity: Entity,
}

fn process_join_requests(
    mut commands: Commands,
    mut client_q: Query<
        (
            Entity,
            &mut ReceivedMessages<JoinRequest>,
            Has<ConnectedPlayer>,
        ),
        With<ClientConnection>,
    >,
) {
    let mut connected_this_tick = Vec::new();

    for (client_entity, mut messages, has_connected_player) in client_q.iter_mut() {
        for JoinRequest { username } in messages.drain() {
            info!(
                "client {} wants to join with username \"{}\"",
                client_entity, username
            );

            if has_connected_player | connected_this_tick.contains(&client_entity) {
                warn!("client {} has already joined", client_entity);
                continue;
            }

            connected_this_tick.push(client_entity);

            let player_entity = commands
                .spawn((
                    PlayerBundle {
                        player: Player { username },
                    },
                    ConnectedClient { client_entity },
                ))
                .id();

            commands
                .entity(client_entity)
                .insert((ConnectedPlayer { player_entity }, ReceiveGameUpdates));

            info!(
                "client {} joined as player {}",
                client_entity, player_entity
            );
        }
    }
}
