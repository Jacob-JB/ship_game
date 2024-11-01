use std::time::Duration;

use avian3d::prelude::*;
use bevy::{
    prelude::*,
    render::{
        camera::{RenderTarget, ScalingMode},
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
        view::RenderLayers,
    },
};
use common::{
    elements::{NewShipMap, ShipMapMoveRequest, ShipMapPositionUpdate},
    GameLayer,
};

use crate::{
    entity_map::{LocalServerEntity, ServerEntityMap, ServerEntityMapper},
    networking::prelude::*,
    player::interaction::{Interactable, InteractionTarget},
    ShipScreenRenderLayer,
};

const SHIP_MAP_MOVE_SPEED: f32 = 5.;
const SHIP_MAP_MOVE_FLUSH_INTERVAL: Duration = Duration::from_millis(100);

pub fn build(app: &mut App) {
    app.init_resource::<ScreenAssets>();

    app.add_systems(
        Update,
        (
            spawn_screens,
            press_ship_map_keys,
            update_ship_map_keys,
            move_ship_map,
            update_ship_maps,
            move_screen_camera,
        ),
    );
}

#[derive(Resource)]
struct ScreenAssets {
    screen_mesh: Handle<Mesh>,
    key_model: Handle<Scene>,
}

impl FromWorld for ScreenAssets {
    fn from_world(world: &mut World) -> Self {
        ScreenAssets {
            screen_mesh: world
                .resource_mut::<Assets<Mesh>>()
                .add(Rectangle::new(0.5, 0.5)),
            key_model: world
                .resource::<AssetServer>()
                .load("arrow_button.gltf#Scene0"),
        }
    }
}

const SCREEN_IMAGE_SIZE: u32 = 128;

#[derive(Component)]
struct ScreenCamera {
    camera_entity: Entity,
}

#[derive(Component)]
struct ScreenTargetPosition {
    pub position: Vec2,
}

#[derive(Component)]
pub struct ShipMapKeys {
    pub left_key_entity: Entity,
    pub right_key_entity: Entity,
    pub up_key_entity: Entity,
    pub down_key_entity: Entity,
    pub move_acc: Vec2,
}

#[derive(Component)]
pub struct ShipMapKey {
    pub pressed: bool,
    pub map_entity: Entity,
    pub offset: Vec3,
}

fn spawn_screens(
    mut commands: Commands,
    mut messages: MessageReceiver<NewShipMap>,
    mut image_assets: ResMut<Assets<Image>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    screen_assets: Res<ScreenAssets>,
    mut map: ServerEntityMapper,
) {
    for NewShipMap {
        entity,
        translation,
        rotation,
        state,
    } in messages.drain()
    {
        info!("new ship map at {}", translation);

        let size = Extent3d {
            width: SCREEN_IMAGE_SIZE,
            height: SCREEN_IMAGE_SIZE,
            ..default()
        };

        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("Screen image"),
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                mag_filter: ImageFilterMode::Nearest,
                min_filter: ImageFilterMode::Nearest,
                ..default()
            }),
            ..default()
        };

        image.resize(size);

        let image = image_assets.add(image);

        let camera_entity = commands
            .spawn((
                Camera2dBundle {
                    camera: Camera {
                        order: -1,
                        target: RenderTarget::Image(image.clone()),
                        clear_color: ClearColorConfig::Custom(Color::BLACK),
                        ..default()
                    },
                    projection: OrthographicProjection {
                        near: -1000.,
                        far: 1000.,
                        scale: state.zoom,
                        scaling_mode: ScalingMode::Fixed {
                            width: 1.,
                            height: 1.,
                        },
                        ..default()
                    },
                    transform: Transform::from_translation(state.position.extend(0.)),
                    ..default()
                },
                RenderLayers::from_layers(&[
                    ShipScreenRenderLayer::Map as usize,
                    ShipScreenRenderLayer::Players as usize,
                ]),
            ))
            .id();

        let material = material_assets.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::WHITE * 10.,
            emissive_texture: Some(image),
            perceptual_roughness: 0.,
            ..default()
        });

        let map_entity = map.get_or_spawn(entity);

        let key_bundle = (
            SceneBundle {
                scene: screen_assets.key_model.clone(),
                ..default()
            },
            Position::default(),
            Collider::cuboid(0.1, 0.05, 0.1),
            CollisionLayers::new([GameLayer::Interaction], 0),
            Interactable,
            DebugRender::default(),
        );

        let left_key_entity = commands
            .spawn((
                key_bundle.clone(),
                ShipMapKey {
                    pressed: false,
                    map_entity,
                    offset: Vec3::new(-0.15, -0.4, 0.),
                },
                Rotation(rotation.mul_quat(Quat::from_euler(
                    EulerRot::XZY,
                    std::f32::consts::FRAC_PI_2,
                    0.,
                    std::f32::consts::FRAC_PI_2,
                ))),
            ))
            .id();

        let right_key_entity = commands
            .spawn((
                key_bundle.clone(),
                ShipMapKey {
                    pressed: false,
                    map_entity,
                    offset: Vec3::new(-0.05, -0.4, 0.),
                },
                Rotation(rotation.mul_quat(Quat::from_euler(
                    EulerRot::XZY,
                    std::f32::consts::FRAC_PI_2,
                    0.,
                    -std::f32::consts::FRAC_PI_2,
                ))),
            ))
            .id();

        let up_key_entity = commands
            .spawn((
                key_bundle.clone(),
                ShipMapKey {
                    pressed: false,
                    map_entity,
                    offset: Vec3::new(0.05, -0.4, 0.),
                },
                Rotation(rotation.mul_quat(Quat::from_euler(
                    EulerRot::XZY,
                    std::f32::consts::FRAC_PI_2,
                    0.,
                    0.,
                ))),
            ))
            .id();

        let down_key_entity = commands
            .spawn((
                key_bundle,
                ShipMapKey {
                    pressed: false,
                    map_entity,
                    offset: Vec3::new(0.15, -0.4, 0.),
                },
                Rotation(rotation.mul_quat(Quat::from_euler(
                    EulerRot::XZY,
                    std::f32::consts::FRAC_PI_2,
                    0.,
                    std::f32::consts::PI,
                ))),
            ))
            .id();

        commands.entity(map_entity).insert((
            MaterialMeshBundle {
                mesh: screen_assets.screen_mesh.clone(),
                material,
                transform: Transform::from_translation(translation).with_rotation(rotation),
                ..default()
            },
            ScreenCamera { camera_entity },
            ScreenTargetPosition {
                position: state.position,
            },
            ShipMapKeys {
                left_key_entity,
                right_key_entity,
                up_key_entity,
                down_key_entity,
                move_acc: Vec2::ZERO,
            },
        ));
    }
}

fn press_ship_map_keys(
    input: Res<ButtonInput<MouseButton>>,
    mut key_q: Query<(Entity, &mut ShipMapKey, Has<InteractionTarget>)>,
    mut current_press: Local<Option<Entity>>,
) {
    if input.pressed(MouseButton::Left) {
        if let None = *current_press {
            for (key_entity, mut key, is_target) in key_q.iter_mut() {
                if is_target {
                    key.pressed = true;
                    *current_press = Some(key_entity);
                }
            }
        }
    } else {
        if let Some(key_entity) = *current_press {
            if let Ok((_, mut key, _)) = key_q.get_mut(key_entity) {
                key.pressed = false;
                *current_press = None;
            }
        }
    }
}

fn update_ship_map_keys(
    mut key_q: Query<(Entity, &ShipMapKey, &mut Transform), Changed<ShipMapKey>>,
    map_q: Query<&GlobalTransform, Without<ShipMapKey>>,
) {
    for (key_entity, key, mut key_transform) in key_q.iter_mut() {
        let Ok(map_transform) = map_q.get(key.map_entity) else {
            error!(
                "couldn't query ship map key {}'s map {}",
                key_entity, key.map_entity
            );
            continue;
        };

        let offset = if key.pressed {
            key.offset + Vec3::new(0., 0., -0.05)
        } else {
            key.offset
        };

        key_transform.translation = map_transform.transform_point(offset);
    }
}

fn move_ship_map(
    mut map_q: Query<(Entity, &LocalServerEntity, &mut ShipMapKeys)>,
    key_q: Query<&ShipMapKey>,
    time: Res<Time>,
    mut last_flush: Local<Duration>,
    mut messages: MessageSender,
    message_id: Res<MessageId<ShipMapMoveRequest>>,
) {
    for (map_entity, _, mut map) in map_q.iter_mut() {
        let mut move_dir = Vec2::ZERO;

        for (key_entity, direction) in [
            (map.left_key_entity, Vec2::NEG_X),
            (map.right_key_entity, Vec2::X),
            (map.down_key_entity, Vec2::NEG_Y),
            (map.up_key_entity, Vec2::Y),
        ] {
            let Ok(key) = key_q.get(key_entity) else {
                error!(
                    "Couldn't query key {} for ship map {}",
                    key_entity, map_entity
                );
                continue;
            };

            if key.pressed {
                move_dir += direction;
            }
        }

        map.move_acc += move_dir.normalize_or_zero() * SHIP_MAP_MOVE_SPEED * time.delta_seconds();
    }

    if time.elapsed() > *last_flush + SHIP_MAP_MOVE_FLUSH_INTERVAL {
        *last_flush = time.elapsed();

        for (_, server_entity, mut map) in map_q.iter_mut() {
            // only reset accumulator if message was sent
            if messages.send(
                *message_id,
                &ShipMapMoveRequest {
                    entity: server_entity.get(),
                    delta: map.move_acc,
                },
            ) {
                map.move_acc = Vec2::ZERO;
            }
        }
    }
}

fn update_ship_maps(
    mut messages: MessageReceiver<ShipMapPositionUpdate>,
    entity_map: Res<ServerEntityMap>,
    mut map_q: Query<(&ScreenCamera, &mut ScreenTargetPosition)>,
    mut camera_q: Query<&mut OrthographicProjection>,
) {
    for ShipMapPositionUpdate { entity, position } in messages.drain() {
        let Some(map_entity) = entity_map.get_client_entity(entity) else {
            warn!(
                "received a ship map position update for a non existent ship map {}",
                entity
            );
            continue;
        };

        let Ok((screen_camera, mut target_position)) = map_q.get_mut(map_entity) else {
            error!(
                "Couldn't query ship map {} to update it's position",
                map_entity
            );
            continue;
        };

        let Ok(mut projection) = camera_q.get_mut(screen_camera.camera_entity) else {
            error!(
                "Couldn't query ship map {}'s camera {}",
                map_entity, screen_camera.camera_entity
            );
            continue;
        };

        // transform.translation = position.position.extend(0.);
        target_position.position = position.position;
        projection.scale = position.zoom;
    }
}

fn move_screen_camera(
    map_q: Query<(Entity, &ScreenCamera, &ScreenTargetPosition)>,
    mut camera_q: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (map_entity, screen_camera, target_position) in map_q.iter() {
        let Ok(mut transform) = camera_q.get_mut(screen_camera.camera_entity) else {
            error!(
                "Couldn't query ship map {}'s camera {}",
                map_entity, screen_camera.camera_entity
            );
            continue;
        };

        // move the ship map towards the target at the move speed
        let move_diff = (target_position.position - transform.translation.xy())
            .clamp_length_max(SHIP_MAP_MOVE_SPEED * time.delta_seconds());

        transform.translation += move_diff.extend(0.);

        // clamp the position to be within 4 flush intervals of the target
        let current_diff = (transform.translation.xy() - target_position.position)
            .clamp_length_max(
                SHIP_MAP_MOVE_SPEED * SHIP_MAP_MOVE_FLUSH_INTERVAL.as_secs_f32() * 4.,
            );

        transform.translation = (target_position.position + current_diff).extend(0.);
    }
}
