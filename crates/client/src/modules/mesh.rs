use avian3d::prelude::*;
use bevy::prelude::*;
use common::{mesh_colliders::GltfCollider, modules::LoadModule, GameLayer};

use crate::{entity_map::ServerEntityMapper, networking::prelude::*};

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
    } in messages.drain()
    {
        let scene_entity = mapper.get_or_spawn(server_entity);

        info!("Loading module {} \"{}\"", scene_entity, path);

        let collider_gltf = assets.load(format!("ship_modules/colliders/{}", path));
        let mesh_gltf: Handle<Scene> = assets.load(format!("ship_modules/meshes/{}#Scene0", path));

        commands.entity(scene_entity).insert((
            TransformBundle::from_transform(Transform {
                translation,
                rotation,
                ..default()
            }),
            mesh_gltf,
            VisibilityBundle::default(),
            RigidBody::Static,
            GltfCollider {
                mesh: collider_gltf,
            },
            CollisionLayers::new([GameLayer::World], [GameLayer::Players]),
        ));
    }
}
