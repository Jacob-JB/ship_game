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
    mut text_q: Query<(&mut Text, &mut Visibility)>,
) {
    let Ok([(mut health_text, mut health_visibility), (mut oxygen_text, mut oxygen_visibility)]) =
        text_q.get_many_mut([
            ui_elements.health_level_entity,
            ui_elements.oxygen_level_entity,
        ])
    else {
        error!("Couldn't query player health text");
        return;
    };

    let Ok(player_health) = local_player_q.get_single() else {
        *health_visibility = Visibility::Hidden;
        *oxygen_visibility = Visibility::Hidden;
        return;
    };

    *health_visibility = Visibility::Inherited;
    *oxygen_visibility = Visibility::Inherited;

    health_text.0 = format!("Health {:.0}%", player_health.health);
    oxygen_text.0 = format!("Oxygen {:.0}%", player_health.oxygen);
}
