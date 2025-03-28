use bevy::prelude::*;
use common::elements::room_vent::*;
use nevy::prelude::ReceivedMessages;

use crate::{
    modules::atmosphere::ModuleVent, networking::prelude::*, player::networking::ConnectedPlayer,
    state::ReceiveGameUpdates,
};

use super::ElementUpdateMessageQueue;

pub fn build(app: &mut App) {
    app.add_event::<SendRoomVentUpdate>();

    app.add_systems(
        Update,
        (
            replicate_new_room_vents,
            replicate_existing_room_vents,
            receive_room_vent_toggle_enabled_requests,
            send_room_vent_updates,
        ),
    );
}

#[derive(Component)]
#[require(Transform)]
pub struct RoomVent {
    pub module_entity: Entity,
}

fn replicate_new_room_vents(
    room_vent_q: Query<(Entity, &RoomVent, &GlobalTransform), Added<RoomVent>>,
    module_q: Query<&ModuleVent>,
    client_q: Query<Entity, With<ReceiveGameUpdates>>,
    mut messages: QueuedMessageSender<ElementUpdateMessageQueue>,
    message_id: Res<MessageId<NewRoomVent>>,
) {
    for (room_vent_entity, room_vent, transform) in room_vent_q.iter() {
        let Ok(vent) = module_q.get(room_vent.module_entity) else {
            error!(
                "Couldn't query vent {:?} for room vent {:?}",
                room_vent.module_entity, room_vent_entity
            );
            continue;
        };

        for client_entity in client_q.iter() {
            messages.send(
                *message_id,
                client_entity,
                NewRoomVent {
                    entity: room_vent_entity.into(),
                    enabled: vent.open,
                    translation: transform.translation(),
                    rotation: transform.rotation(),
                },
            );
        }
    }
}

fn replicate_existing_room_vents(
    room_vent_q: Query<(Entity, &RoomVent, &GlobalTransform)>,
    module_q: Query<&ModuleVent>,
    client_q: Query<Entity, Added<ReceiveGameUpdates>>,
    mut messages: QueuedMessageSender<ElementUpdateMessageQueue>,
    message_id: Res<MessageId<NewRoomVent>>,
) {
    for client_entity in client_q.iter() {
        for (room_vent_entity, room_vent, transform) in room_vent_q.iter() {
            let Ok(vent) = module_q.get(room_vent.module_entity) else {
                error!(
                    "Couldn't query vent {:?} for room vent {:?}",
                    room_vent.module_entity, room_vent_entity
                );
                continue;
            };

            messages.send(
                *message_id,
                client_entity,
                NewRoomVent {
                    entity: room_vent_entity.into(),
                    enabled: vent.open,
                    translation: transform.translation(),
                    rotation: transform.rotation(),
                },
            );
        }
    }
}

#[derive(Event)]
struct SendRoomVentUpdate {
    room_vent_entity: Entity,
}

fn receive_room_vent_toggle_enabled_requests(
    mut client_q: Query<(
        Entity,
        &mut ReceivedMessages<RequestToggleRoomVentEnabled>,
        Has<ConnectedPlayer>,
    )>,
    room_vent_q: Query<&RoomVent>,
    mut module_vent_q: Query<&mut ModuleVent>,
    mut send_update_w: EventWriter<SendRoomVentUpdate>,
) {
    for (client_entity, mut messages, has_player) in client_q.iter_mut() {
        for RequestToggleRoomVentEnabled { entity } in messages.drain() {
            let true = has_player else {
                warn!(
                    "client {} sent a toggle room vent enabled request when they weren't a player",
                    client_entity
                );
                continue;
            };

            let room_vent_entity = entity.into();

            let Ok(room_vent) = room_vent_q.get(room_vent_entity) else {
                warn!(
                    "client {} tried to toggle a room vent that doesn't exist {}",
                    client_entity, room_vent_entity
                );
                continue;
            };

            let Ok(mut module_vent) = module_vent_q.get_mut(room_vent.module_entity) else {
                error!(
                    "couldn't query module vent {} for room vent {}",
                    room_vent.module_entity, room_vent_entity
                );
                continue;
            };

            module_vent.open = !module_vent.open;

            send_update_w.send(SendRoomVentUpdate { room_vent_entity });
        }
    }
}

fn send_room_vent_updates(
    mut send_update_r: EventReader<SendRoomVentUpdate>,
    room_vent_q: Query<&RoomVent>,
    module_vent_q: Query<&ModuleVent>,
    client_q: Query<Entity, With<ReceiveGameUpdates>>,
    mut messages: QueuedMessageSender<ElementUpdateMessageQueue>,
    message_id: Res<MessageId<UpdateRoomVent>>,
) {
    for &SendRoomVentUpdate { room_vent_entity } in send_update_r.read() {
        let Ok(room_vent) = room_vent_q.get(room_vent_entity) else {
            error!("couldn't query room vent {}", room_vent_entity);
            continue;
        };

        let Ok(module_vent) = module_vent_q.get(room_vent.module_entity) else {
            error!(
                "couldn't query module vent {} for room vent {}",
                room_vent.module_entity, room_vent_entity
            );
            continue;
        };

        for client_entity in client_q.iter() {
            messages.send(
                *message_id,
                client_entity,
                UpdateRoomVent {
                    entity: room_vent_entity.into(),
                    enabled: module_vent.open,
                },
            );
        }
    }
}
