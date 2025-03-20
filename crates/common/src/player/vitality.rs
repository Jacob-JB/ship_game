use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const MAX_HEALTH: f32 = 100.0;
pub const MAX_OXYGEN: f32 = 100.0;

/// Component used on the server and client to store vitality stats
#[derive(Clone, Copy, Component, Serialize, Deserialize)]
pub struct PlayerVitality {
    pub health: f32,
    pub oxygen: f32,
}

impl Default for PlayerVitality {
    fn default() -> Self {
        PlayerVitality {
            health: MAX_HEALTH,
            oxygen: MAX_OXYGEN,
        }
    }
}

/// Server -> Client Message to update player vitality of that client
#[derive(Serialize, Deserialize)]
pub struct UpdatePlayerVitality {
    pub vitality: PlayerVitality,
}
