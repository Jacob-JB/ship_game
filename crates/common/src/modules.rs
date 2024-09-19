use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ServerEntity;

/// Message from server -> client to tell them
/// to load a module mesh and collider.
#[derive(Serialize, Deserialize)]
pub struct LoadModule {
    pub path: String,
    pub server_entity: ServerEntity,
    pub translation: Vec3,
    pub rotation: f32,
    pub map_offset: Vec2,
    pub map_size: Vec2,
}
