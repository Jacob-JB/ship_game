use avian3d::prelude::*;
use bevy::prelude::*;
use common::{player::*, state::JoinRequest};
use controller::PlayerInput;
use nevy::prelude::ReceivedMessages;

use super::{Player, PlayerBundle};
use crate::{networking::prelude::*, state::ReceiveGameUpdates};

pub fn build(app: &mut App) {
    app.add_plugins(MessageQueuePlugin::<PlayerUpdateQueue>::default());

    app.add_systems(
        Update,
        (
            process_join_requests,
            broadcast_new_players,
            init_existing_players,
            receive_state_updates,
        ),
    );
}

/// Marker type for the player update queue
struct PlayerUpdateQueue;

/// exists on a [Player] entity and points to it's client entity
///
/// if a [Player] entity doesn't have this component then there is no
/// connected client
///
/// there is a matching [ConnectedPlayer] on the client entity
#[derive(Component)]
pub struct ConnectedClient {
    client_entity: Entity,
}

/// points to the [Player] entity if this client has one
///
/// there is a matching [ConnectedClient] on the player entity
#[derive(Component)]
pub struct ConnectedPlayer {
    player_entity: Entity,
}

impl ConnectedClient {
    pub fn get(&self) -> Entity {
        self.client_entity
    }
}

impl ConnectedPlayer {
    pub fn get(&self) -> Entity {
        self.player_entity
    }
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
                    PlayerBundle::new(username),
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

/// Responsible for initializing new players for existing clients.
fn broadcast_new_players(
    player_q: Query<(Entity, &Player, &Position), Added<Player>>,
    client_q: Query<(Entity, &ConnectedPlayer), With<ReceiveGameUpdates>>,
    mut messages: QueuedMessageSender<PlayerUpdateQueue>,
    message_id: Res<MessageId<NewPlayer>>,
) {
    for (player_entity, player, &Position(position)) in player_q.iter() {
        for (client_entity, connected_player) in client_q.iter() {
            messages.send(
                *message_id,
                client_entity,
                NewPlayer {
                    server_entity: player_entity.into(),
                    username: player.username.clone(),
                    local_player: (connected_player.get() == player_entity)
                        .then(|| NewLocalPlayer { position }),
                },
            );
        }
    }
}

/// Responsible for initializing players on new clients.
///
/// Doesn't inform the player about their local player
fn init_existing_players(
    client_q: Query<(Entity, Option<&ConnectedPlayer>), Added<ReceiveGameUpdates>>,
    player_q: Query<(Entity, &Player)>,
    mut messages: QueuedMessageSender<PlayerUpdateQueue>,
    message_id: Res<MessageId<NewPlayer>>,
) {
    for (client_entity, connected_player) in client_q.iter() {
        for (player_entity, player) in player_q.iter() {
            if let Some(connected_player) = connected_player {
                if connected_player.get() == player_entity {
                    continue;
                }
            }

            messages.send(
                *message_id,
                client_entity,
                NewPlayer {
                    server_entity: player_entity.into(),
                    username: player.username.clone(),
                    local_player: None,
                },
            );
        }
    }
}

fn receive_state_updates(
    mut client_q: Query<(
        Entity,
        &mut ReceivedMessages<ClientPlayerUpdate>,
        Option<&ConnectedPlayer>,
    )>,
    mut player_q: Query<(&mut Position, &mut LinearVelocity, &mut PlayerInput)>,
) {
    for (client_entity, mut messages, connected_player) in client_q.iter_mut() {
        for ClientPlayerUpdate {
            position,
            linear_velocity,
            input,
        } in messages.drain()
        {
            let Some(connected_player) = connected_player else {
                warn!(
                    "client {} sent a player state update but they don't have a connected player",
                    client_entity
                );
                continue;
            };

            let player_entity = connected_player.get();

            let Ok((mut player_position, mut player_velocity, mut player_input)) =
                player_q.get_mut(player_entity)
            else {
                error!(
                    "couldn't query client {}'s connected player {} to apply a state update",
                    client_entity, player_entity
                );
                continue;
            };

            player_position.0 = position;
            player_velocity.0 = linear_velocity;
            *player_input = input;
        }
    }
}
