use avian3d::prelude::*;
use bevy::prelude::*;
use common::{
    player::{controller::PlayerInput, *},
    GameLayer,
};

use crate::physics::playout::SnapshotInterpolation;

pub mod controller;
pub mod interaction;
pub mod networking;

pub fn build(app: &mut App) {
    networking::build(app);
    controller::build(app);
    interaction::build(app);
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
}

/// Components for a local player
#[derive(Bundle)]
pub struct LocalPlayerBundle {
    player: Player,
    local_player: LocalPlayer,
    player_input: PlayerInput,
    rigid_body: RigidBody,
    position: Position,
    collider: Collider,
    locked_axes: LockedAxes,
    collision_layers: CollisionLayers,
    restitution: Restitution,
}

impl PlayerBundle {
    pub fn new(username: String) -> Self {
        PlayerBundle {
            player: Player { username },
            snapshot_interpolation: SnapshotInterpolation::default(),
            position: Position::default(),
            rotation: Rotation::default(),
            collider: player_collider(),
        }
    }
}

impl LocalPlayerBundle {
    pub fn new(username: String, new_local_player: NewLocalPlayer) -> Self {
        LocalPlayerBundle {
            player: Player { username },
            local_player: LocalPlayer,
            player_input: PlayerInput::default(),
            rigid_body: RigidBody::Dynamic,
            position: Position(new_local_player.position),
            collider: player_collider(),
            locked_axes: LockedAxes::default().lock_rotation_x().lock_rotation_z(),
            collision_layers: CollisionLayers::new([GameLayer::Players], [GameLayer::World]),
            restitution: Restitution::PERFECTLY_INELASTIC,
        }
    }
}
