use bevy::{input::mouse::MouseMotion, prelude::*};
use common::player::*;
use controller::PlayerInput;

use super::LocalPlayer;

pub fn build(app: &mut App) {
    controller::build_player_controller(app);

    app.insert_resource(MouseSensitivity(Vec2::splat(0.005)));

    app.add_systems(Update, (get_movement_input, get_camera_input));
}

pub const PLAYER_MOVE_SPEED: f32 = 5.;

fn get_movement_input(
    input: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<&mut PlayerInput, With<LocalPlayer>>,
) {
    let Ok(mut player_input) = player_q.get_single_mut() else {
        return;
    };

    let move_forward = input.pressed(KeyCode::KeyW);
    let move_backward = input.pressed(KeyCode::KeyS);
    let move_left = input.pressed(KeyCode::KeyA);
    let move_right = input.pressed(KeyCode::KeyD);

    let input_vector = Vec2 {
        x: match (move_left, move_right) {
            (true, false) => -1.,
            (false, true) => 1.,
            _ => 0.,
        },
        y: match (move_forward, move_backward) {
            (true, false) => -1.,
            (false, true) => 1.,
            _ => 0.,
        },
    }
    .normalize_or_zero();

    player_input.target_velocity = input_vector * PLAYER_MOVE_SPEED;
}

#[derive(Resource)]
pub struct MouseSensitivity(pub Vec2);

fn get_camera_input(
    mut mouse: EventReader<MouseMotion>,
    sensitivity: Res<MouseSensitivity>,
    mut player_q: Query<&mut PlayerInput, With<LocalPlayer>>,
    mut rotation: Local<Vec2>,
) {
    let delta = mouse.read().map(|e| e.delta).sum::<Vec2>() * -sensitivity.0;

    rotation.x += delta.x;
    rotation.y = (rotation.y + delta.y).clamp(
        -std::f32::consts::FRAC_PI_2 * 0.9,
        std::f32::consts::FRAC_PI_2 * 0.9,
    );

    let Ok(mut player_input) = player_q.get_single_mut() else {
        return;
    };

    player_input.look_direction = Dir3::new(
        Quat::from_euler(EulerRot::YXZ, rotation.x, rotation.y, 0.).mul_vec3(Vec3::NEG_Z),
    )
    .unwrap();
}
