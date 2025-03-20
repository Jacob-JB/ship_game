use avian3d::prelude::*;
use bevy::prelude::*;
use common::{
    player::{controller::PlayerInput, vitality::PlayerVitality, *},
    GameLayer,
};

use crate::physics::playout::SnapshotInterpolation;

pub mod controller;
pub mod interaction;
pub mod networking;
pub mod vitality;

pub fn build(app: &mut App) {
    networking::build(app);
    controller::build(app);
    interaction::build(app);
    vitality::build(app);
}

/// Marker component for a player on the client
#[derive(Component)]
pub struct Player {
    pub username: String,
}

/// Marker component for the single local player
#[derive(Component)]
pub struct LocalPlayer;

/// Components for a player
#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    snapshot_interpolation: SnapshotInterpolation,
    position: Position,
    rotation: Rotation,
    collider: Collider,
    collision_layers: CollisionLayers,
}

/// Components for a local player
#[derive(Bundle)]
pub struct LocalPlayerBundle {
    player: Player,
    local_player: LocalPlayer,
    player_input: PlayerInput,
    position: Position,
    rotation: Rotation,
    transform: Transform,
    linear_velocity: LinearVelocity,
    collider: Collider,
    collision_layers: CollisionLayers,
    player_health: PlayerVitality,
}

impl PlayerBundle {
    pub fn new(username: String) -> Self {
        PlayerBundle {
            player: Player { username },
            snapshot_interpolation: SnapshotInterpolation::default(),
            position: Position::default(),
            rotation: Rotation::default(),
            collider: player_collider(),
            collision_layers: CollisionLayers::new([GameLayer::Players], 0),
        }
    }
}

impl LocalPlayerBundle {
    pub fn new(username: String, new_local_player: NewLocalPlayer) -> Self {
        LocalPlayerBundle {
            player: Player { username },
            local_player: LocalPlayer,
            player_input: PlayerInput::default(),
            position: Position(new_local_player.position),
            rotation: Rotation::default(),
            transform: Transform::default(),
            linear_velocity: LinearVelocity::default(),
            collider: player_collider(),
            collision_layers: CollisionLayers::new([GameLayer::Players], 0),
            player_health: PlayerVitality::default(),
        }
    }
}
