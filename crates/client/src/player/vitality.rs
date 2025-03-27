use bevy::prelude::*;
use common::player::vitality::*;

use crate::{networking::prelude::*, ui::UiElements};

use super::LocalPlayer;

pub fn build(app: &mut App) {
    app.add_systems(Update, (update_health_ui, receive_player_vitality_updates));
}

fn receive_player_vitality_updates(
    mut messages: MessageReceiver<UpdatePlayerVitality>,
    mut player_vitality: Query<&mut PlayerVitality, With<LocalPlayer>>,
) {
    for UpdatePlayerVitality { vitality } in messages.drain() {
        let Ok(mut player_vitality) = player_vitality.get_single_mut() else {
            continue;
        };

        *player_vitality = vitality;
    }
}

fn update_health_ui(
    local_player_q: Query<&PlayerVitality, With<LocalPlayer>>,
    ui_elements: Res<UiElements>,
    mut visibility_q: Query<&mut Visibility>,
    mut text_q: Query<&mut Text>,
) {
    let Ok(mut visibility) = visibility_q.get_mut(ui_elements.vitality.vitality_node_entity) else {
        error!("Couldn't query player vitality visibility");
        return;
    };

    let Ok([mut pressure_text, mut health_text, mut oxygen_text]) = text_q.get_many_mut([
        ui_elements.vitality.pressure_level_entity,
        ui_elements.vitality.health_level_entity,
        ui_elements.vitality.oxygen_level_entity,
    ]) else {
        error!("Couldn't query player health text");
        return;
    };

    let Ok(player_health) = local_player_q.get_single() else {
        *visibility = Visibility::Hidden;
        return;
    };

    *visibility = Visibility::Inherited;

    pressure_text.0 = format!("Pressure {:.0}%", player_health.pressure * 100.);
    health_text.0 = format!("Health {:.0}", player_health.health);
    oxygen_text.0 = format!("Oxygen {:.0}", player_health.oxygen);
}
