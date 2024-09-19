use bevy::prelude::*;
use common::elements::{NewShipMap, ShipMapMoveRequest, ShipMapPosition, ShipMapPositionUpdate};
use nevy::prelude::ReceivedMessages;

use crate::{
    modules::module_types::InitShipModules, networking::prelude::*,
    player::networking::ConnectedPlayer, state::ReceiveGameUpdates,
};

use super::ElementUpdateMessageQueue;

pub fn build(app: &mut App) {
    app.add_systems(
        Update,
        (
            (replicate_new_ship_maps, replicate_existing_ship_maps).before(InitShipModules),
            move_ship_maps,
        ),
    );
}

/// Component with ship map element state
///
/// Must be spawned in or after [InitShipModules] so that
/// the global transform has been computed for network replication the next frame
#[derive(Component)]
pub struct ShipMap {
    pub position: Vec2,
    pub zoom: f32,
}

impl Default for ShipMap {
    fn default() -> Self {
        ShipMap {
            position: Vec2::default(),
            zoom: 15.,
        }
    }
}

#[derive(Bundle, Default)]
pub struct ShipMapBundle {
    pub map: ShipMap,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

fn replicate_new_ship_maps(
    map_q: Query<(Entity, &ShipMap, &GlobalTransform), Added<ShipMap>>,
    client_q: Query<Entity, With<ReceiveGameUpdates>>,
    mut messages: QueuedMessageSender<ElementUpdateMessageQueue>,
    message_id: Res<MessageId<NewShipMap>>,
) {
    for (map_entity, map, transform) in map_q.iter() {
        for client_entity in client_q.iter() {
            messages.send(
                *message_id,
                client_entity,
                NewShipMap {
                    entity: map_entity.into(),
                    translation: transform.translation(),
                    rotation: transform.to_scale_rotation_translation().1,
                    state: ShipMapPosition {
                        position: map.position,
                        zoom: map.zoom,
                    },
                },
            );
        }
    }
}

fn replicate_existing_ship_maps(
    map_q: Query<(Entity, &ShipMap, &GlobalTransform)>,
    client_q: Query<Entity, Added<ReceiveGameUpdates>>,
    mut messages: QueuedMessageSender<ElementUpdateMessageQueue>,
    message_id: Res<MessageId<NewShipMap>>,
) {
    for client_entity in client_q.iter() {
        for (map_entity, map, transform) in map_q.iter() {
            info!("sent ship map");
            messages.send(
                *message_id,
                client_entity,
                NewShipMap {
                    entity: map_entity.into(),
                    translation: transform.translation(),
                    rotation: transform.to_scale_rotation_translation().1,
                    state: ShipMapPosition {
                        position: map.position,
                        zoom: map.zoom,
                    },
                },
            );
        }
    }
}

fn move_ship_maps(
    mut client_q: Query<(
        Entity,
        &mut ReceivedMessages<ShipMapMoveRequest>,
        Has<ConnectedPlayer>,
    )>,
    mut map_q: Query<&mut ShipMap>,

    update_client_q: Query<Entity, With<ReceiveGameUpdates>>,
    mut sender: QueuedMessageSender<ElementUpdateMessageQueue>,
    message_id: Res<MessageId<ShipMapPositionUpdate>>,
) {
    for (client_entity, mut messages, has_player) in client_q.iter_mut() {
        for ShipMapMoveRequest { entity, delta } in messages.drain() {
            let true = has_player else {
                error!(
                    "client {} sent a ship map move request when they weren't a player",
                    client_entity
                );
                continue;
            };

            let map_entity = entity.into();

            let Ok(mut map) = map_q.get_mut(map_entity) else {
                warn!(
                    "client {} tried to move a ship map {} that doesn't exist",
                    client_entity, entity
                );
                continue;
            };

            map.position += delta;

            for client_entity in update_client_q.iter() {
                sender.send(
                    *message_id,
                    client_entity,
                    ShipMapPositionUpdate {
                        entity: map_entity.into(),
                        position: ShipMapPosition {
                            position: map.position,
                            zoom: map.zoom,
                        },
                    },
                )
            }
        }
    }
}
