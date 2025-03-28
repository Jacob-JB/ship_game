use avian3d::prelude::*;
use bevy::{
    color::palettes::css::{GREEN, RED},
    prelude::*,
};
use common::{elements::room_vent::*, GameLayer};

use crate::{
    assets::GameAssets,
    entity_map::{LocalServerEntity, ServerEntityMap, ServerEntityMapper},
    networking::prelude::*,
    player::interaction::{Interactable, InteractionTarget},
    screens::{RenderLayerAllocater, Screens},
};

use super::ModuleMessages;

const VENT_SCREEN_RESOLUTION: UVec2 = UVec2::new(48, 24);

pub fn build(app: &mut App) {
    app.add_systems(
        Update,
        (
            spawn_room_vents,
            update_vent_ui,
            move_vent_handles,
            send_room_vent_toggle_enabled_requests,
            receive_room_vent_updates,
        ),
    );
}

#[derive(Component)]
pub struct RoomVent {
    enabled: bool,
    screen_entity: Entity,
    screen_camera_entity: Entity,
    vent_ui_root_entity: Entity,
    vent_open_text_entity: Entity,
    enable_handle_mesh_entity: Entity,
    enable_handle_collider_entity: Entity,
}

const ENABLE_HANDLE_MESH_OFFSET: Vec3 = Vec3::new(0., -0.2, 0.);
const ENABLE_HANDLE_COLLIDER_OFFSET: Vec3 = Vec3::new(0., -0.2, 0.025);

fn spawn_room_vents(
    mut commands: Commands,
    mut messages: MessageReceiver<NewRoomVent>,
    mut mapper: ServerEntityMapper,
    mut layers: ResMut<RenderLayerAllocater>,
    mut screens: Screens,
    assets: Res<GameAssets>,
) {
    for NewRoomVent {
        entity,
        enabled,
        translation,
        rotation,
    } in messages.drain()
    {
        let room_vent_entity = mapper.get_or_spawn(entity);

        // screen

        let render_layer = layers.next();

        let screen_entity = commands.spawn_empty().set_parent(room_vent_entity).id();

        let screen_camera_entity = screens.create_screen(
            screen_entity,
            VENT_SCREEN_RESOLUTION,
            Transform::from_scale(Vec3::new(0.5, 0.25, 1.)),
            1.,
            default(),
            &[render_layer],
        );

        let vent_ui_root_entity = commands
            .spawn((
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                TargetCamera(screen_camera_entity),
            ))
            .id();

        let vent_open_text_entity = commands
            .spawn((
                Text::default(),
                TextFont {
                    font_size: 16.,
                    ..default()
                },
            ))
            .set_parent(vent_ui_root_entity)
            .id();

        // enable handle

        let room_vent_transform = Transform {
            translation,
            rotation,
            ..default()
        };

        let enable_handle_mesh_entity = commands
            .spawn((SceneRoot(assets.valve_handle.clone()), Transform::default()))
            .id();

        let collider_transform = room_vent_transform.mul_transform(Transform {
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

        commands.entity(room_vent_entity).insert((
            RoomVent {
                enabled,
                screen_entity,
                screen_camera_entity,
                vent_ui_root_entity,
                vent_open_text_entity,
                enable_handle_collider_entity,
                enable_handle_mesh_entity,
            },
            Transform {
                translation,
                rotation,
                ..default()
            },
        ));
    }
}

fn update_vent_ui(room_vent_q: Query<&RoomVent>, mut text_q: Query<(&mut Text, &mut TextColor)>) {
    for room_vent in room_vent_q.iter() {
        let Ok((mut text, mut text_color)) = text_q.get_mut(room_vent.vent_open_text_entity) else {
            continue;
        };

        text.0 = if room_vent.enabled { "Open" } else { "Shut" }.into();

        text_color.0 = if room_vent.enabled {
            GREEN.into()
        } else {
            RED.into()
        };
    }
}

fn move_vent_handles(
    room_vent_q: Query<(Entity, &RoomVent, &GlobalTransform)>,
    mut room_vent_handle_q: Query<&mut Transform>,
) {
    for (room_vent_entity, room_vent, room_vent_transform) in room_vent_q.iter() {
        let Ok(mut handle_transform) =
            room_vent_handle_q.get_mut(room_vent.enable_handle_mesh_entity)
        else {
            error!(
                "Couldn't query room vent {}'s handle mesh {}",
                room_vent_entity, room_vent.enable_handle_mesh_entity
            );
            continue;
        };

        let handle_angle = if room_vent.enabled {
            -std::f32::consts::FRAC_PI_2
        } else {
            0.0
        };

        *handle_transform = room_vent_transform
            .compute_transform()
            .mul_transform(Transform {
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

fn send_room_vent_toggle_enabled_requests(
    input: Res<ButtonInput<MouseButton>>,
    room_vent_q: Query<(&RoomVent, &LocalServerEntity)>,
    interaction_target_q: Query<(), With<InteractionTarget>>,
    mut messages: QueuedMessageSender<ModuleMessages>,
    message_id: Res<MessageId<RequestToggleRoomVentEnabled>>,
) {
    let true = input.just_pressed(MouseButton::Left) else {
        return;
    };

    for (room_vent, room_vent_server_entity) in room_vent_q.iter() {
        let true = interaction_target_q.contains(room_vent.enable_handle_collider_entity) else {
            continue;
        };

        messages.send(
            *message_id,
            RequestToggleRoomVentEnabled {
                entity: room_vent_server_entity.get(),
            },
        );

        // Only one handle can be the interaction target
        return;
    }
}

fn receive_room_vent_updates(
    mut messages: MessageReceiver<UpdateRoomVent>,
    map: Res<ServerEntityMap>,
    mut room_vent_q: Query<&mut RoomVent>,
) {
    for UpdateRoomVent { entity, enabled } in messages.drain() {
        let Some(room_vent_entity) = map.get_client_entity(entity) else {
            warn!("Received room vent update for unknown entity {}", entity);
            continue;
        };

        let Ok(mut room_vent) = room_vent_q.get_mut(room_vent_entity) else {
            error!("Couldn't query room vent {}", entity);
            continue;
        };

        room_vent.enabled = enabled;
    }
}
