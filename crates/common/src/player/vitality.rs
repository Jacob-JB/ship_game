use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const MAX_HEALTH: f32 = 100.0;
pub const MAX_OXYGEN: f32 = 40.0;

/// Component used on the server and client to store vitality stats
#[derive(Clone, Copy, Component, Serialize, Deserialize)]
pub struct PlayerVitality {
    /// Value from 0 to [MAX_HEALTH].
    pub health: f32,
    /// How many seconds of oxygen before the player starts suffocating.
    pub oxygen: f32,
    /// Ratio from 0 to 1
    pub pressure: f32,
}

impl Default for PlayerVitality {
    fn default() -> Self {
        PlayerVitality {
            health: MAX_HEALTH,
            oxygen: MAX_OXYGEN,
            pressure: 0.0,
        }
    }
}

/// Server -> Client Message to update player vitality of that client
#[derive(Serialize, Deserialize)]
pub struct UpdatePlayerVitality {
    pub vitality: PlayerVitality,
}
