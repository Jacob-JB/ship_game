use avian3d::prelude::*;
use bevy::{prelude::*, render::view::RenderLayers};
use common::{mesh_colliders::GltfCollider, modules::LoadModule, GameLayer};

use crate::{
    entity_map::ServerEntityMapper, modules::ModuleMapSprite, networking::prelude::*,
    ShipScreenRenderLayer,
};

pub fn build(app: &mut App) {
    app.add_systems(Update, load_static_scenes);
}

fn load_static_scenes(
    mut commands: Commands,
    mut messages: MessageReceiver<LoadModule>,
    mut mapper: ServerEntityMapper,
    assets: Res<AssetServer>,
) {
    for LoadModule {
        path,
        server_entity,
        translation,
        rotation,
        map_offset,
        map_size,
    } in messages.drain()
    {
        let scene_entity = mapper.get_or_spawn(server_entity);

        info!("Loading module {} \"{}\"", scene_entity, path);

        let collider_gltf = assets.load(format!("ship_modules/colliders/{}.gltf", path));
        let mesh_gltf: Handle<Scene> =
            assets.load(format!("ship_modules/meshes/{}.gltf#Scene0", path));
        let map_image = assets.load(format!("ship_modules/map/{}.png", path));

        let map_translation =
            translation + Vec2::from_angle(rotation).rotate(map_offset).extend(0.);

        let map_entity = commands
            .spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(map_size),
                        ..default()
                    },
                    texture: map_image,
                    transform: Transform {
                        translation: map_translation,
                        rotation: Quat::from_rotation_y(rotation),
                        ..default()
                    },
                    ..default()
                },
                RenderLayers::from_layers(&[ShipScreenRenderLayer::Map as usize]),
            ))
            .id();

        commands.entity(scene_entity).insert((
            TransformBundle::from_transform(Transform {
                translation,
                rotation: Quat::from_rotation_y(rotation),
                ..default()
            }),
            mesh_gltf,
            VisibilityBundle::default(),
            RigidBody::Static,
            GltfCollider {
                mesh: collider_gltf,
            },
            CollisionLayers::new([GameLayer::World], [GameLayer::Players]),
            ModuleMapSprite { entity: map_entity },
        ));
    }
}
