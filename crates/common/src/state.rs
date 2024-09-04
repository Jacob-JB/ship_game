use serde::{Deserialize, Serialize};

use crate::ServerEntity;

/// message sent from client to server to request to join the current game
#[derive(Serialize, Deserialize, Clone)]
pub struct JoinRequest {
    pub username: String,
}

/// unique id for a player
#[derive(Serialize, Deserialize)]
pub struct PlayerId {
    /// server entity of player
    pub server_entity: ServerEntity,
}
