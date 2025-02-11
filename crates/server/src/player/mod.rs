use avian3d::prelude::*;
use bevy::prelude::*;
use common::{player::*, GameLayer};
use controller::PlayerInput;

use crate::physics::networking::ReplicateBody;

pub mod networking;

pub fn build(app: &mut App) {
    networking::build(app);

    controller::build_player_controller(app);

    app.add_systems(Update, debug_player_positions);
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
    position: Position,
    rotation: Rotation,
    linear_velocity: LinearVelocity,
    collider: Collider,
    replicate_body: ReplicateBody,
    collision_layers: CollisionLayers,
}

impl PlayerBundle {
    pub fn new(username: String) -> Self {
        PlayerBundle {
            player: Player { username },
            player_input: PlayerInput::default(),
            position: Position(Vec3::new(0., 1., 0.)),
            rotation: Rotation::default(),
            linear_velocity: LinearVelocity::default(),
            collider: player_collider(),
            replicate_body: ReplicateBody,
            collision_layers: CollisionLayers::new([GameLayer::Players], 0),
        }
    }
}

fn debug_player_positions(player_q: Query<&Position, With<Player>>) {
    for position in player_q.iter() {
        // debug!("{}", **position)
    }
}
