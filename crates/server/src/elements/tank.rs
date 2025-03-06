use bevy::prelude::*;
use common::elements::{NewTank, TankState};

use crate::modules::module_types::InitShipModules;
use crate::networking::prelude::*;
use crate::{modules::atmosphere::AtmosphereTank, state::ReceiveGameUpdates};

use super::ElementUpdateMessageQueue;

pub fn build(app: &mut App) {
    app.add_systems(
        Update,
        (replicate_new_tanks, replicate_existing_tanks).before(InitShipModules),
    );
}

#[derive(Bundle, Default)]
pub struct TankBundle {
    pub atmosphere_tank: AtmosphereTank,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

fn replicate_new_tanks(
    tank_q: Query<(Entity, &AtmosphereTank, &GlobalTransform), Added<AtmosphereTank>>,
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
    tank_q: Query<(Entity, &AtmosphereTank, &GlobalTransform)>,
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
