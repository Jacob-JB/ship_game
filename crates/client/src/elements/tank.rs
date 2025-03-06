use avian3d::prelude::*;
use bevy::prelude::*;
use common::{
    elements::tank::{NewTank, RequestToggleTankEnabled, UpdateTankState},
    GameLayer,
};

use crate::{
    entity_map::{LocalServerEntity, ServerEntityMap, ServerEntityMapper},
    networking::prelude::*,
    player::interaction::{Interactable, InteractionTarget},
    screens::{RenderLayerAllocater, Screens},
};

use super::ModuleMessages;

const SCREEN_IMAGE_SIZE: UVec2 = UVec2::new(24, 48);

pub fn build(app: &mut App) {
    app.init_resource::<ValveHandleScene>();

    app.add_systems(
        Update,
        (
            spawn_tanks,
            update_tank_state,
            send_tank_toggle_enabled_requests,
            move_tank_handles,
        ),
    );
}

#[derive(Resource)]
struct ValveHandleScene(Handle<Scene>);

impl FromWorld for ValveHandleScene {
    fn from_world(world: &mut World) -> Self {
        ValveHandleScene(
            world
                .resource::<AssetServer>()
                .load("valve_handle.gltf#Scene0"),
        )
    }
}

#[derive(Component)]
pub struct Tank {
    enable_handle_mesh_entity: Entity,
    enable_handle_collider_entity: Entity,
    enabled: bool,
}

const ENABLE_HANDLE_MESH_OFFSET: Vec3 = Vec3::new(0., -0.35, 0.);
const ENABLE_HANDLE_COLLIDER_OFFSET: Vec3 = Vec3::new(0., -0.35, 0.025);

fn spawn_tanks(
    mut commands: Commands,
    mut messages: MessageReceiver<NewTank>,
    mut mapper: ServerEntityMapper,
    mut screens: Screens,
    mut layers: ResMut<RenderLayerAllocater>,
    valve_handle_scene: Res<ValveHandleScene>,
) {
    for NewTank {
        entity,
        translation,
        rotation,
        state,
    } in messages.drain()
    {
        let tank_entity = mapper.get_or_spawn(entity);

        let render_layer = layers.next();

        let tank_transform = Transform {
            translation,
            rotation,
            ..default()
        };

        screens.create_screen(
            tank_entity,
            SCREEN_IMAGE_SIZE,
            Transform {
                scale: Vec3::new(0.5, 1., 1.),
                ..tank_transform
            },
            1.,
            default(),
            &[render_layer],
        );

        let enable_handle_mesh_entity = commands
            .spawn((
                SceneRoot(valve_handle_scene.0.clone()),
                Transform::default(),
            ))
            .id();

        let collider_transform = tank_transform.mul_transform(Transform {
            translation: ENABLE_HANDLE_COLLIDER_OFFSET,
            ..default()
        });

        let enable_handle_collider_entity = commands
            .spawn((
                Collider::cuboid(0.1, 0.1, 0.05),
                Position(collider_transform.translation),
                Rotation(collider_transform.rotation),
                CollisionLayers::new([GameLayer::Interaction], 0),
                Interactable,
                DebugRender::default(),
            ))
            .id();

        commands.entity(tank_entity).insert(Tank {
            enable_handle_mesh_entity,
            enable_handle_collider_entity,
            enabled: state.enabled,
        });
    }
}

fn update_tank_state(
    mut messages: MessageReceiver<UpdateTankState>,
    map: Res<ServerEntityMap>,
    mut tank_q: Query<&mut Tank>,
) {
    for UpdateTankState { entity, state } in messages.drain() {
        let Some(tank_entity) = map.get_client_entity(entity) else {
            error!(
                "Received tank state update for a tank that doesn't exist {}",
                entity
            );
            continue;
        };

        let Ok(mut tank) = tank_q.get_mut(tank_entity) else {
            error!("Couldn't query tank {}", tank_entity);
            continue;
        };

        tank.enabled = state.enabled;
    }
}

fn send_tank_toggle_enabled_requests(
    input: Res<ButtonInput<MouseButton>>,
    tank_q: Query<(&Tank, &LocalServerEntity)>,
    interaction_target_q: Query<(), With<InteractionTarget>>,
    mut messages: QueuedMessageSender<ModuleMessages>,
    message_id: Res<MessageId<RequestToggleTankEnabled>>,
) {
    let true = input.just_pressed(MouseButton::Left) else {
        return;
    };

    for (tank, tank_server_entity) in tank_q.iter() {
        let true = interaction_target_q.contains(tank.enable_handle_collider_entity) else {
            continue;
        };

        messages.send(
            *message_id,
            RequestToggleTankEnabled {
                entity: tank_server_entity.get(),
            },
        );

        // Only one handle can be the interaction target
        return;
    }
}

fn move_tank_handles(
    tank_q: Query<(Entity, &Tank, &GlobalTransform)>,
    mut tank_handle_q: Query<&mut Transform>,
) {
    for (tank_entity, tank, tank_transform) in tank_q.iter() {
        let Ok(mut handle_transform) = tank_handle_q.get_mut(tank.enable_handle_mesh_entity) else {
            error!(
                "Couldn't query tank {}'s handle mesh {}",
                tank_entity, tank.enable_handle_mesh_entity
            );
            continue;
        };

        let handle_angle = if tank.enabled {
            0.0
        } else {
            -std::f32::consts::FRAC_PI_2
        };

        *handle_transform = tank_transform.compute_transform().mul_transform(Transform {
            scale: Vec3::splat(0.25),
            translation: ENABLE_HANDLE_MESH_OFFSET,
            rotation: Quat::from_euler(
                EulerRot::XZY,
                std::f32::consts::FRAC_PI_2,
                0.,
                handle_angle,
            ),
        });
    }
}
