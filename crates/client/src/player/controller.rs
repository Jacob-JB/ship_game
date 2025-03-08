use avian3d::prelude::*;
use bevy::{input::mouse::MouseMotion, prelude::*};
use common::{player::*, GameLayer};
use controller::PlayerInput;

use super::LocalPlayer;

pub const PLAYER_MOVE_SPEED: f32 = 5.;
pub const PLAYER_JUMP_SPEED: f32 = 3.;
pub const ON_GROUND_TOLERANCE: f32 = 0.02;
pub const RESET_FLOOR: f32 = -50.0;

pub fn build(app: &mut App) {
    controller::build_player_controller(app);

    app.insert_resource(MouseSensitivity(Vec2::splat(0.002)));

    app.add_systems(
        Update,
        (
            get_movement_input,
            get_camera_input,
            jump_players,
            reset_fallen_player,
        ),
    );
}

fn get_movement_input(
    input: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<(&mut PlayerInput, &Rotation), With<LocalPlayer>>,
) {
    let Ok((mut player_input, &Rotation(rotation))) = player_q.get_single_mut() else {
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

    let rotation = rotation.mul_vec3(Vec3::X).xz();

    player_input.target_velocity = rotation.rotate(input_vector) * PLAYER_MOVE_SPEED;
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

fn jump_players(
    input: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<(&Position, &mut LinearVelocity), With<LocalPlayer>>,
    spatial_query: SpatialQuery,
) {
    let Ok((&Position(position), mut velocity)) = player_q.get_single_mut() else {
        return;
    };

    let mut shape = player_collider();
    shape.set_scale(Vec3::splat(0.99), 10);

    let on_ground = spatial_query
        .cast_shape(
            &shape,
            position,
            Quat::IDENTITY,
            Dir3::NEG_Y,
            &ShapeCastConfig {
                max_distance: ON_GROUND_TOLERANCE,
                ignore_origin_penetration: true,
                ..default()
            },
            &SpatialQueryFilter::from_mask([GameLayer::World]),
        )
        .is_some();

    if on_ground && input.just_pressed(KeyCode::Space) {
        velocity.0 += Vec3::Y * PLAYER_JUMP_SPEED;
    }
}

fn reset_fallen_player(
    mut player_q: Query<(&mut Position, &mut LinearVelocity), With<LocalPlayer>>,
) {
    let Ok((mut position, mut linear_velocity)) = player_q.get_single_mut() else {
        return;
    };

    if position.0.y < RESET_FLOOR {
        position.0 = Vec3::new(0., 1., 0.);
        *linear_velocity = LinearVelocity::default();
    }
}
