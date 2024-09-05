use avian3d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub fn build_player_controller(app: &mut App) {
    app.add_systems(Update, (accelerate_players, rotate_players));
}

#[derive(Component, Serialize, Deserialize, Clone, Copy)]
pub struct PlayerInput {
    pub target_velocity: Vec2,
    pub look_direction: Dir3,
}

impl Default for PlayerInput {
    fn default() -> Self {
        PlayerInput {
            target_velocity: Vec2::ZERO,
            look_direction: Dir3::NEG_Z,
        }
    }
}

const PLAYER_ACCELERATION: f32 = 30.;

fn accelerate_players(mut player_q: Query<(&PlayerInput, &mut LinearVelocity)>, time: Res<Time>) {
    for (input, mut velocity) in player_q.iter_mut() {
        let delta = (input.target_velocity - velocity.0.xz())
            .clamp_length_max(PLAYER_ACCELERATION * time.delta_seconds());
        velocity.0 += Vec3::new(delta.x, 0., delta.y);
    }
}

fn rotate_players(mut player_q: Query<(&PlayerInput, &mut Rotation)>) {
    for (input, mut rotation) in player_q.iter_mut() {
        let face_direction = Vec3 {
            y: 0.,
            ..input.look_direction.into()
        }
        .normalize();

        rotation.0 = Transform::default()
            .looking_to(face_direction, Vec3::Y)
            .rotation;
    }
}
