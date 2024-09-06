use avian3d::prelude::*;
use bevy::prelude::*;
use common::{player::*, GameLayer};
use controller::PlayerInput;

use crate::physics::networking::ReplicateBody;

pub mod networking;

pub fn build(app: &mut App) {
    networking::build(app);

    controller::build_player_controller(app);
}

/// marker component for a player entity
///
/// the player entity is separate from the
/// client entity and can exist without one
#[derive(Component)]
pub struct Player {
    pub username: String,
}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    player_input: PlayerInput,
    rigid_body: RigidBody,
    position: Position,
    collider: Collider,
    replicate_body: ReplicateBody,
    locked_axes: LockedAxes,
    collision_layers: CollisionLayers,
    restitution: Restitution,
}

impl PlayerBundle {
    pub fn new(username: String) -> Self {
        PlayerBundle {
            player: Player { username },
            player_input: PlayerInput::default(),
            rigid_body: RigidBody::Dynamic,
            position: Position(Vec3::new(0., 3., 0.)),
            collider: player_collider(),
            replicate_body: ReplicateBody,
            locked_axes: LockedAxes::default().lock_rotation_x().lock_rotation_z(),
            collision_layers: CollisionLayers::new([GameLayer::Players], [GameLayer::World]),
            restitution: Restitution::PERFECTLY_INELASTIC,
        }
    }
}
