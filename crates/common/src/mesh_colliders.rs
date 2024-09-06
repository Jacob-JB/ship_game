use avian3d::prelude::*;
use bevy::{gltf::GltfMesh, prelude::*, render::mesh::Indices};

pub struct GltfColliderPlugin;

impl Plugin for GltfColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, load_mesh_colliders);
    }
}

/// Will insert a [Collider] from a the first gltf mesh of a handle.
///
/// Will log an error and remove this component if there is an error,
/// no collider will be inserted.
///
/// Will use the first primitive of the gltf mesh.
#[derive(Component)]
pub struct GltfCollider {
    pub mesh: Handle<Gltf>,
}

fn load_mesh_colliders(
    mut commands: Commands,
    collider_q: Query<(Entity, &GltfCollider), Without<Collider>>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<GltfMesh>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
) {
    for (collider_entity, collider) in collider_q.iter() {
        let Some(gltf) = gltf_assets.get(&collider.mesh) else {
            // wait until asset is loaded
            continue;
        };

        let Some(gltf_mesh_handle) = gltf.meshes.first() else {
            error!("No meshes in gltf collider for {}", collider_entity);
            commands.entity(collider_entity).remove::<GltfCollider>();
            continue;
        };

        let Some(gltf_mesh) = gltf_mesh_assets.get(gltf_mesh_handle) else {
            error!(
                "Gltf collider asset for {} was loaded but mesh wasn't stored",
                collider_entity
            );
            commands.entity(collider_entity).remove::<GltfCollider>();
            continue;
        };

        let mesh_handle = &gltf_mesh
            .primitives
            .first()
            .expect("Gltf mesh should have at least one primitive")
            .mesh;

        let Some(mesh) = mesh_assets.get_mut(mesh_handle) else {
            error!(
                "First mesh primitive for gltf mesh for {} wasn't stored",
                collider_entity
            );
            commands.entity(collider_entity).remove::<GltfCollider>();
            continue;
        };

        // dbg!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        // dbg!(mesh.indices().is_some());

        // `trimesh_from_mesh` expects indicies
        if let None = mesh.indices() {
            let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
                error!(
                    "Mesh collider for {} has no position attribute",
                    collider_entity
                );
                commands.entity(collider_entity).remove::<GltfCollider>();
                continue;
            };

            mesh.insert_indices(Indices::U32(
                (0..positions.len()).map(|i| i as u32).collect(),
            ));
        }

        let Some(collider_component) = Collider::trimesh_from_mesh(mesh) else {
            error!(
                "Couldn't convert gltf mesh for {} into a collider",
                collider_entity
            );
            commands.entity(collider_entity).remove::<GltfCollider>();
            continue;
        };

        commands.entity(collider_entity).insert(collider_component);
    }
}
