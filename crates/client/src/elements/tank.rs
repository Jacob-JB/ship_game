use avian3d::prelude::*;
use bevy::{
    color::palettes::css::{GREEN, RED},
    prelude::*,
};
use common::{elements::tank::*, GameLayer};

use crate::{
    assets::GameAssets,
    entity_map::{LocalServerEntity, ServerEntityMap, ServerEntityMapper},
    networking::prelude::*,
    player::interaction::{Interactable, InteractionTarget},
    screens::{RenderLayerAllocater, Screens},
};

use super::ModuleMessages;

const TANK_SCREEN_RESOLUTION: UVec2 = UVec2::new(48, 48);

pub fn build(app: &mut App) {
    app.add_systems(
        Update,
        (
            spawn_tanks,
            update_tank_state,
            send_tank_toggle_enabled_requests,
            move_tank_handles,
            update_enabled_text,
            receive_tank_percentage_updates,
        ),
    );
}

#[derive(Component)]
pub struct Tank {
    screen_camera_entity: Entity,
    tank_ui_root: Entity,
    level_text_entity: Entity,
    enabled_text_entity: Entity,
    enable_handle_mesh_entity: Entity,
    enable_handle_collider_entity: Entity,
    enabled: bool,
}

const ENABLE_HANDLE_MESH_OFFSET: Vec3 = Vec3::new(0., -0.2, 0.);
const ENABLE_HANDLE_COLLIDER_OFFSET: Vec3 = Vec3::new(0., -0.2, 0.025);

fn spawn_tanks(
    mut commands: Commands,
    mut messages: MessageReceiver<NewTank>,
    mut mapper: ServerEntityMapper,
    mut screens: Screens,
    mut layers: ResMut<RenderLayerAllocater>,
    assets: Res<GameAssets>,
) {
    for NewTank {
        entity,
        translation,
        rotation,
        state,
    } in messages.drain()
    {
        let tank_entity = mapper.get_or_spawn(entity);

        // screen

        let render_layer = layers.next();

        let screen_entity = commands.spawn_empty().set_parent(tank_entity).id();

        let screen_camera_entity = screens.create_screen(
            screen_entity,
            TANK_SCREEN_RESOLUTION,
            Transform::from_scale(Vec3::new(0.5, 0.5, 1.)),
            1.,
            default(),
            &[render_layer],
        );

        // ui

        let tank_ui_root = commands
            .spawn((
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                TargetCamera(screen_camera_entity),
            ))
            .id();

        let level_text_entity = commands
            .spawn((
                Text::default(),
                TextFont {
                    font_size: 16.,
                    ..default()
                },
                Node {
                    margin: UiRect::all(Val::Px(1.)),
                    ..default()
                },
            ))
            .set_parent(tank_ui_root)
            .id();

        let enabled_text_entity = commands
            .spawn((
                Text::default(),
                TextFont {
                    font_size: 16.,
                    ..default()
                },
                TextColor(GREEN.into()),
                Node {
                    margin: UiRect::all(Val::Px(1.)),
                    ..default()
                },
            ))
            .set_parent(tank_ui_root)
            .id();

        // enable handle

        let tank_transform = Transform {
            translation,
            rotation,
            ..default()
        };

        let enable_handle_mesh_entity = commands
            .spawn((SceneRoot(assets.valve_handle.clone()), Transform::default()))
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

        commands.entity(tank_entity).insert((
            Tank {
                screen_camera_entity,
                tank_ui_root,
                level_text_entity,
                enabled_text_entity,
                enable_handle_mesh_entity,
                enable_handle_collider_entity,
                enabled: state.enabled,
            },
            tank_transform,
        ));
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
            -std::f32::consts::FRAC_PI_2
        } else {
            0.0
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

fn update_enabled_text(
    tank_q: Query<(Entity, &Tank)>,
    mut text_q: Query<(&mut Text, &mut TextColor)>,
) {
    for (tank_entity, tank) in tank_q.iter() {
        let Ok((mut enabled_text, mut enabled_color)) = text_q.get_mut(tank.enabled_text_entity)
        else {
            error!(
                "Couldn't query tank {}'s enabled text {}",
                tank_entity, tank.enabled_text_entity
            );
            continue;
        };

        enabled_text.0 = if tank.enabled {
            "Open".to_string()
        } else {
            "Shut".to_string()
        };

        enabled_color.0 = if tank.enabled {
            GREEN.into()
        } else {
            RED.into()
        };
    }
}

fn receive_tank_percentage_updates(
    mut messages: MessageReceiver<UpdateTankPercentage>,
    map: Res<ServerEntityMap>,
    tank_q: Query<&Tank>,
    mut text_q: Query<&mut Text>,
) {
    for UpdateTankPercentage { entity, percentage } in messages.drain() {
        let Some(tank_entity) = map.get_client_entity(entity) else {
            // percentage updates are unordered, it's ok if the tank doesn't exist
            continue;
        };

        let Ok(tank) = tank_q.get(tank_entity) else {
            error!("Couldn't query tank {}", tank_entity);
            continue;
        };

        let Ok(mut level_text) = text_q.get_mut(tank.level_text_entity) else {
            error!(
                "Couldn't query tank {}'s level text {}",
                tank_entity, tank.level_text_entity
            );
            continue;
        };

        level_text.0 = format!("{:.0}%", percentage * 100.);
    }
}
