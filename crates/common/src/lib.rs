use avian3d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub mod networking;
pub mod physics;
pub mod player;
pub mod state;

/// newtype over an entity from the server's ecs world
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash)]
pub struct ServerEntity(Entity);

impl std::fmt::Display for ServerEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Server({})", self)
    }
}

impl From<Entity> for ServerEntity {
    fn from(value: Entity) -> Self {
        ServerEntity(value)
    }
}

impl From<ServerEntity> for Entity {
    fn from(value: ServerEntity) -> Self {
        value.0
    }
}

#[derive(PhysicsLayer)]
pub enum GameLayer {
    World,
    Players,
}
