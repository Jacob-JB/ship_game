use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ServerEntity;

/// Message from server to client to initialize a new ship map element
#[derive(Serialize, Deserialize)]
pub struct NewShipMap {
    pub entity: ServerEntity,
    pub translation: Vec3,
    pub rotation: Quat,
    pub state: ShipMapPosition,
}

/// Message from server to client to update ship map state
#[derive(Serialize, Deserialize)]
pub struct ShipMapPosition {
    pub position: Vec2,
    pub zoom: f32,
}

#[derive(Serialize, Deserialize)]
pub struct ShipMapPositionUpdate {
    pub entity: ServerEntity,
    pub position: ShipMapPosition,
}

/// Message from client to server to request to move a ship map
#[derive(Serialize, Deserialize)]
pub struct ShipMapMoveRequest {
    pub entity: ServerEntity,
    pub delta: Vec2,
}
