use std::time::Duration;

use bevy::prelude::*;
use common::elements::tank::{
    NewTank, RequestToggleTankEnabled, TankState, UpdateTankPercentage, UpdateTankState,
};
use nevy::prelude::ReceivedMessages;

use crate::modules::module_types::InitShipModules;
use crate::networking::prelude::*;
use crate::player::networking::ConnectedPlayer;
use crate::{modules::atmosphere::TankAtmosphere, state::ReceiveGameUpdates};

use super::ElementUpdateMessageQueue;

const TANK_PERCENTAGE_UPDATE_INTERVAL: Duration = Duration::from_millis(200);

pub fn build(app: &mut App) {
    app.add_event::<SendTankStateUpdate>();

    app.add_systems(
        Update,
        (
            (replicate_new_tanks, replicate_existing_tanks).before(InitShipModules),
            updated_changed_tank_state,
            receive_tank_toggle_enabled_requests,
            send_tank_percentage_updates,
        ),
    );
}

#[derive(Event)]
struct SendTankStateUpdate {
    tank_entity: Entity,
}

/// Marker component for tank elements.
#[derive(Component, Default)]
#[require(TankAtmosphere, Transform)]
pub struct Tank;

// #[derive(Bundle, Default)]
// pub struct TankBundle {
//     pub tank: Tank,
//     pub atmosphere_tank: TankAtmosphere,
//     pub transform: Transform,
//     pub global_transform: GlobalTransform,
// }

fn replicate_new_tanks(
    tank_q: Query<(Entity, &TankAtmosphere, &GlobalTransform), Added<TankAtmosphere>>,
    client_q: Query<Entity, With<ReceiveGameUpdates>>,
    mut messages: QueuedMessageSender<ElementUpdateMessageQueue>,
    message_id: Res<MessageId<NewTank>>,
) {
    for (tank_entity, tank, transform) in tank_q.iter() {
        for client_entity in client_q.iter() {
            messages.send(
                *message_id,
                client_entity,
                NewTank {
                    entity: tank_entity.into(),
                    translation: transform.translation(),
                    rotation: transform.to_scale_rotation_translation().1,
                    state: TankState {
                        enabled: tank.enabled,
                    },
                },
            );
        }
    }
}

fn replicate_existing_tanks(
    tank_q: Query<(Entity, &TankAtmosphere, &GlobalTransform)>,
    client_q: Query<Entity, Added<ReceiveGameUpdates>>,
    mut messages: QueuedMessageSender<ElementUpdateMessageQueue>,
    message_id: Res<MessageId<NewTank>>,
) {
    for client_entity in client_q.iter() {
        for (tank_entity, tank, transform) in tank_q.iter() {
            messages.send(
                *message_id,
                client_entity,
                NewTank {
                    entity: tank_entity.into(),
                    translation: transform.translation(),
                    rotation: transform.to_scale_rotation_translation().1,
                    state: TankState {
                        enabled: tank.enabled,
                    },
                },
            );
        }
    }
}

fn updated_changed_tank_state(
    mut send_update_r: EventReader<SendTankStateUpdate>,
    tank_q: Query<&TankAtmosphere>,
    client_q: Query<Entity, With<ReceiveGameUpdates>>,
    mut messages: QueuedMessageSender<ElementUpdateMessageQueue>,
    message_id: Res<MessageId<UpdateTankState>>,
) {
    for &SendTankStateUpdate { tank_entity } in send_update_r.read() {
        let Ok(tank) = tank_q.get(tank_entity) else {
            error!("Couldn't query tank {}", tank_entity);
            continue;
        };

        for client_entity in client_q.iter() {
            messages.send(
                *message_id,
                client_entity,
                UpdateTankState {
                    entity: tank_entity.into(),
                    state: TankState {
                        enabled: tank.enabled,
                    },
                },
            );
        }
    }
}

fn receive_tank_toggle_enabled_requests(
    mut client_q: Query<(
        Entity,
        &mut ReceivedMessages<RequestToggleTankEnabled>,
        Has<ConnectedPlayer>,
    )>,
    mut tank_q: Query<&mut TankAtmosphere>,
    mut send_update_w: EventWriter<SendTankStateUpdate>,
) {
    for (client_entity, mut messages, has_player) in client_q.iter_mut() {
        for RequestToggleTankEnabled { entity } in messages.drain() {
            let true = has_player else {
                warn!(
                    "client {} sent a toggle tank enabled request when they weren't a player",
                    client_entity
                );
                continue;
            };

            let tank_entity = entity.into();

            let Ok(mut tank) = tank_q.get_mut(tank_entity) else {
                warn!(
                    "client {} tried to toggle a tank that doesn't exist {}",
                    client_entity, tank_entity
                );
                continue;
            };

            tank.enabled = !tank.enabled;
            send_update_w.send(SendTankStateUpdate { tank_entity });
        }
    }
}

fn send_tank_percentage_updates(
    tank_q: Query<(Entity, &TankAtmosphere), With<Tank>>,
    client_q: Query<Entity, With<ReceiveGameUpdates>>,
    mut message_sender: MessageSender,
    message_id: Res<MessageId<UpdateTankPercentage>>,
    time: Res<Time>,
    mut last_update: Local<Duration>,
) {
    if time.elapsed() - *last_update > TANK_PERCENTAGE_UPDATE_INTERVAL {
        *last_update = time.elapsed();

        for (tank_entity, tank) in tank_q.iter() {
            let message = UpdateTankPercentage {
                entity: tank_entity.into(),
                percentage: tank.level / tank.volume,
            };

            for client_entity in client_q.iter() {
                // send unreliably
                message_sender.send(*message_id, client_entity, &message);
            }
        }
    }
}
