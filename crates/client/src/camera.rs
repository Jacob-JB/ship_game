use avian3d::prelude::*;
use bevy::{
    core_pipeline::bloom::Bloom,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use common::player::controller::PlayerInput;

use crate::player::LocalPlayer;

pub fn build(app: &mut App) {
    app.add_systems(Startup, spawn_main_camera);
    app.add_systems(Update, toggle_cursor_lock);
    app.add_systems(PostUpdate, move_camera_to_player.in_set(PhysicsSet::Sync));
}

/// Marker component for the main camera.
#[derive(Component)]
pub struct MainCamera;

/// Responsible for spawning the main camera on startup.
fn spawn_main_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Bloom::default(),
    ));
}

const CAMERA_HEIGHT_OFFSET: f32 = 0.8;

fn move_camera_to_player(
    mut camera_q: Query<&mut Transform, With<MainCamera>>,
    player_q: Query<(&Position, &PlayerInput), With<LocalPlayer>>,
) {
    let Ok((&Position(player_position), player_input)) = player_q.get_single() else {
        return;
    };

    let Ok(mut camera_transform) = camera_q.get_single_mut() else {
        return;
    };

    camera_transform.translation = player_position + Vec3::Y * CAMERA_HEIGHT_OFFSET;
    camera_transform.look_to(player_input.look_direction, Vec3::Y);
}

fn toggle_cursor_lock(
    mut window_q: Query<&mut Window, With<PrimaryWindow>>,
    input: Res<ButtonInput<KeyCode>>,
    mut locked: Local<bool>,
) {
    let Ok(mut window) = window_q.get_single_mut() else {
        return;
    };

    if input.just_pressed(KeyCode::AltLeft) {
        *locked = !*locked;

        if *locked {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
        } else {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    }
}
