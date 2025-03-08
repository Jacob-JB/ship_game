use avian3d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::GameLayer;

const PLAYER_ACCELERATION: f32 = 75.;
const MAX_INTEGRATE_ITERATIONS: usize = 20;
const PLAYER_COLLISION_MARGIN: f32 = 0.0005;

pub fn build_player_controller(app: &mut App) {
    app.add_systems(
        PostUpdate,
        insert_missing_position_updates.in_set(PhysicsSet::Prepare),
    );

    app.add_systems(
        PostUpdate,
        (
            (rotate_players, accelerate_players),
            integrate_players,
            update_player_positions,
        )
            .chain()
            .in_set(PhysicsSet::StepSimulation),
    );
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

fn accelerate_players(
    mut player_q: Query<(&PlayerInput, &mut LinearVelocity)>,
    gravity: Res<Gravity>,
    time: Res<Time>,
) {
    for (input, mut velocity) in player_q.iter_mut() {
        let difference = input.target_velocity - velocity.0.xz();
        let max_acceleration = PLAYER_ACCELERATION * time.delta_secs();
        let delta = difference.clamp_length_max(max_acceleration);
        **velocity += Vec3::new(delta.x, 0., delta.y);

        **velocity += gravity.0 * time.delta_secs();
    }
}

#[derive(Component, Default)]
struct CharacterPositionUpdate(Vec3);

fn insert_missing_position_updates(
    mut commands: Commands,
    chacater_q: Query<Entity, (With<PlayerInput>, Without<CharacterPositionUpdate>)>,
) {
    for entity in chacater_q.iter() {
        commands
            .entity(entity)
            .insert(CharacterPositionUpdate::default());
    }
}

/// Integrates kinematic character positions.
/// Performs collision detection and slides characters along obstacles.
fn integrate_players(
    mut character_q: Query<
        (
            Entity,
            &mut LinearVelocity,
            &Position,
            &mut CharacterPositionUpdate,
            &Rotation,
            &Collider,
        ),
        With<PlayerInput>,
    >,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    for (player_entity, mut velocity, position, mut position_update, rotation, collider) in
        character_q.iter_mut()
    {
        let mut position = **position;
        let mut remaining_time = time.delta_secs();

        for _ in 0..MAX_INTEGRATE_ITERATIONS {
            let Ok(direction) = Dir3::new(**velocity) else {
                break;
            };

            let max_distance = remaining_time * velocity.length();

            let hit = spatial_query
                .shape_hits(
                    collider,
                    position,
                    **rotation,
                    direction,
                    u32::MAX,
                    &ShapeCastConfig {
                        max_distance,
                        ..default()
                    },
                    &SpatialQueryFilter::from_mask([GameLayer::World])
                        .with_excluded_entities(std::iter::once(player_entity)),
                )
                .into_iter()
                .filter(|hit| -hit.normal1.dot(direction.into()) > 0.)
                .next();

            let Some(hit) = hit else {
                position += direction * max_distance;
                break;
            };

            let hit_normal = rotation.mul_vec3(-hit.normal2);

            remaining_time -= hit.distance / velocity.length();

            position += direction * hit.distance;
            position += hit_normal * PLAYER_COLLISION_MARGIN;

            **velocity = velocity.reject_from(hit_normal);
        }

        position_update.0 = position;
    }
}

/// Updates character positions after [integrate_characters].
fn update_player_positions(mut character_q: Query<(&mut Position, &CharacterPositionUpdate)>) {
    for (mut position, position_update) in character_q.iter_mut() {
        **position = position_update.0;
    }
}
