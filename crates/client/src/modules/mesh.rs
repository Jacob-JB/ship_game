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

        info!("Loading new module {} \"{}\"", scene_entity, path);

        let collider = assets.load(format!("ship_modules/colliders/{}", path));

        commands.entity(scene_entity).insert((
            TransformBundle::from_transform(Transform {
                translation,
                rotation,
                ..default()
            }),
            VisibilityBundle::default(),
            RigidBody::Static,
            GltfCollider { mesh: collider },
            CollisionLayers::new([GameLayer::World], [GameLayer::Players]),
        ));
    }
}
