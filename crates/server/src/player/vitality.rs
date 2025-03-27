use std::time::Duration;

use bevy::prelude::*;
use common::player::vitality::*;

use crate::{
    modules::{atmosphere::ModuleAtmosphere, grid::ShipGridPresence},
    networking::prelude::*,
};

use super::networking::ConnectedClient;

/// How much of a player's oxygen is refilled per unit of module atmosphere.
const PLAYER_OXYGEN_REFILL_RATIO: f32 = 50.;
/// How much of a player's oxygen is refilled per second.
const PLAYER_OXYGEN_REFILL_RATE: f32 = 10.;

const VITALITY_UPDATE_INTERVAL: Duration = Duration::from_millis(100);

pub fn build(app: &mut App) {
    app.add_systems(Update, (send_vitality_updates, update_oxygen));
}

fn send_vitality_updates(
    player_q: Query<(&PlayerVitality, &ConnectedClient)>,
    mut messages: MessageSender,
    message_id: Res<MessageId<UpdatePlayerVitality>>,
    time: Res<Time>,
    mut last_update: Local<Duration>,
) {
    if time.elapsed() - *last_update > VITALITY_UPDATE_INTERVAL {
        *last_update = time.elapsed();

        for (vitality, client) in player_q.iter() {
            messages.send(
                *message_id,
                client.get(),
                &UpdatePlayerVitality {
                    vitality: *vitality,
                },
            );
        }
    }
}

fn update_oxygen(
    mut player_q: Query<(&mut PlayerVitality, &ShipGridPresence)>,
    mut module_q: Query<&mut ModuleAtmosphere>,
    time: Res<Time>,
) {
    for (mut vitality, grid_presence) in player_q.iter_mut() {
        // Remove oxygen from the player
        vitality.oxygen = vitality.oxygen - time.delta_secs();

        // If the player is in a module refill their tank from that module
        if let Some(module_entity) = grid_presence.current_module {
            let Ok(mut module_atmosphere) = module_q.get_mut(module_entity) else {
                error!("Couldn't query module atmosphere {:?}", module_entity);
                continue;
            };

            vitality.pressure = module_atmosphere.level / module_atmosphere.volume;

            // Find the required amount of oxygen to refill this tick.
            let difference = MAX_OXYGEN - vitality.oxygen;
            let required_tank = difference.min(time.delta_secs() * PLAYER_OXYGEN_REFILL_RATE);
            let required_module = required_tank / PLAYER_OXYGEN_REFILL_RATIO;

            // Find how much oxygen the player can refill from the module.
            let satisfaction = 1.0f32.min(module_atmosphere.level / required_module);

            // Update the player's oxygen level and the module's oxygen level.
            vitality.oxygen += required_tank * satisfaction;
            module_atmosphere.level -= required_module * satisfaction;
        } else {
            vitality.pressure = 0.0;
        }
    }
}
