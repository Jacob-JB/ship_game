use avian3d::prelude::*;
use bevy::prelude::*;

use crate::physics::networking::ReplicateBody;

pub mod networking;

pub fn build(app: &mut App) {
    networking::build(app);
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
    rigid_body: RigidBody,
    collider: Collider,
    replicate_body: ReplicateBody,
}

impl PlayerBundle {
    pub fn new(username: String) -> Self {
        PlayerBundle {
            player: Player { username },
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule(0.25, 1.5),
            replicate_body: ReplicateBody,
        }
    }
}
