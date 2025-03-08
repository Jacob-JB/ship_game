use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ServerEntity;

/// Message from server to client to initialize a new tank element
#[derive(Serialize, Deserialize)]
pub struct NewTank {
    pub entity: ServerEntity,
    pub translation: Vec3,
    pub rotation: Quat,
    pub state: TankState,
}

/// The state of a tank
#[derive(Serialize, Deserialize)]
pub struct TankState {
    pub enabled: bool,
}

/// Message from server to client to update the state of an existing tank
#[derive(Serialize, Deserialize)]
pub struct UpdateTankState {
    pub entity: ServerEntity,
    pub state: TankState,
}

/// Message from client to server to request to toggle the enabled state of a tank
#[derive(Serialize, Deserialize)]
pub struct RequestToggleTankEnabled {
    pub entity: ServerEntity,
}

/// Message from server to client to update the percentage filled level of a tank
#[derive(Serialize, Deserialize)]
pub struct UpdateTankPercentage {
    pub entity: ServerEntity,
    pub percentage: f32,
}
