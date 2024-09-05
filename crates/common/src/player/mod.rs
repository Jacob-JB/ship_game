use avian3d::prelude::*;
use bevy::prelude::*;
use controller::PlayerInput;
use serde::{Deserialize, Serialize};

use crate::ServerEntity;

pub mod controller;

/// Message from server to client to initialize a new player.
#[derive(Serialize, Deserialize)]
pub struct NewPlayer {
    pub server_entity: ServerEntity,
    pub username: String,
    pub local_player: Option<NewLocalPlayer>,
}

/// Included in [NewPlayer] if the player is a local player to that client.
///
/// Contains additional state the client needs.
#[derive(Serialize, Deserialize)]
pub struct NewLocalPlayer {
    pub position: Vec3,
}

pub fn player_collider() -> Collider {
    Collider::capsule(0.25, 1.5)
}

/// Message from client to server to update player state.
#[derive(Serialize, Deserialize)]
pub struct ClientPlayerUpdate {
    pub position: Vec3,
    pub linear_velocity: Vec3,
    pub input: PlayerInput,
}
