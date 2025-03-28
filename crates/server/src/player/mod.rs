use avian3d::prelude::*;
use bevy::prelude::*;
use common::{
    player::{
        controller::{build_player_controller, PlayerInput},
        player_collider,
        vitality::PlayerVitality,
    },
    GameLayer,
};

use crate::{modules::grid::ShipGridPresence, physics::networking::ReplicateBody};

pub mod networking;
pub mod vitality;

pub fn build(app: &mut App) {
    networking::build(app);
    vitality::build(app);

    build_player_controller(app);

    app.add_systems(PostUpdate, update_player_transform);
}

/// component for a player entity
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
    transform: Transform,
    linear_velocity: LinearVelocity,
    collider: Collider,
    replicate_body: ReplicateBody,
    collision_layers: CollisionLayers,
    grid_presence: ShipGridPresence,
    vitality: PlayerVitality,
}

impl PlayerBundle {
    pub fn new(username: String) -> Self {
        PlayerBundle {
            player: Player { username },
            player_input: PlayerInput::default(),
            position: Position(Vec3::new(0., 1., 0.)),
            rotation: Rotation::default(),
            transform: Transform::default(),
            linear_velocity: LinearVelocity::default(),
            collider: player_collider(),
            replicate_body: ReplicateBody,
            collision_layers: CollisionLayers::new([GameLayer::Players], 0),
            grid_presence: ShipGridPresence::default(),
            vitality: PlayerVitality::default(),
        }
    }
}

fn update_player_transform(mut player_q: Query<(&Position, &mut Transform), With<Player>>) {
    for (position, mut transform) in player_q.iter_mut() {
        transform.translation = position.0;
    }
}
