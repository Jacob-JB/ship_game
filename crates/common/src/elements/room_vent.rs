use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ServerEntity;

#[derive(Serialize, Deserialize)]
pub struct NewRoomVent {
    pub entity: ServerEntity,
    pub enabled: bool,
    pub translation: Vec3,
    pub rotation: Quat,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateRoomVent {
    pub entity: ServerEntity,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RequestToggleRoomVentEnabled {
    pub entity: ServerEntity,
}
