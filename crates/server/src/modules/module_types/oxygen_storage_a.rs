use bevy::prelude::*;
use common::mesh_colliders::GltfCollider;

use crate::{
    elements::tank::Tank,
    grid_spaces,
    modules::{atmosphere::TankAtmosphere, grid::ShipModuleTransform, networking::ModuleAssets},
};

use super::{add_ship_module_type, InitShipModules, ShipModuleDescription, SpawnShipModule};

pub fn build(app: &mut App) {
    let module_type_id = add_ship_module_type::<OxygenStorageAModule>(
        app,
        ShipModuleDescription {
            module_name: "Oxygen Storage A".into(),
            grid_spaces: grid_spaces![(0, 0), (0, 1),],
        },
    );

    app.add_systems(
        Update,
        init_oxygen_storage_a_modules.in_set(InitShipModules),
    );

    // debug spawn oxygen storage module
    app.add_systems(Startup, move |mut spawn_w: EventWriter<SpawnShipModule>| {
        spawn_w.send(SpawnShipModule {
            module_type_id,
            transform: ShipModuleTransform {
                translation: IVec2::new(0, 2),
                rotation: crate::modules::grid::ModuleRotation::East,
            },
        });
    });
}

/// Marker component for the oxygen storage A module
#[derive(Component, Default)]
pub struct OxygenStorageAModule;

fn init_oxygen_storage_a_modules(
    mut commands: Commands,
    module_q: Query<Entity, Added<OxygenStorageAModule>>,
    assets: Res<AssetServer>,
) {
    for module_entity in module_q.iter() {
        let mesh = assets.load("ship_modules/colliders/oxygen_storage_a.gltf");

        commands.entity(module_entity).insert((
            ModuleAssets {
                path: "oxygen_storage_a".into(),
                map_offset: Vec2::new(0.0, 0.0),
                map_size: Vec2::new(3., 3.),
            },
            GltfCollider { mesh },
        ));

        commands
            .spawn((
                Tank,
                TankAtmosphere {
                    volume: 60.,
                    level: 60.,
                    enabled: false,
                },
                Transform::from_xyz(-0.9, 1.8, 0.)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
            ))
            .set_parent(module_entity);
    }
}
