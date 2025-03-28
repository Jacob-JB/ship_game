use bevy::prelude::*;
use common::mesh_colliders::GltfCollider;

use crate::{
    elements::{room_vent::RoomVent, ship_map::ShipMapBundle, tank::Tank},
    grid_spaces,
    modules::{
        atmosphere::{ModuleAtmosphere, ModuleVent, TankAtmosphere},
        grid::ShipModuleTransform,
        networking::ModuleAssets,
    },
};

use super::{add_ship_module_type, InitShipModules, ShipModuleDescription, SpawnShipModule};

pub fn build(app: &mut App) {
    let module_type_id = add_ship_module_type::<CommandShipModule>(
        app,
        ShipModuleDescription {
            grid_spaces: grid_spaces![
                (-1, -1),
                (0, -1),
                (1, -1),
                (-1, 0),
                (0, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
            ],
            module_name: "Command Module".into(),
        },
    );

    app.add_systems(Update, init_command_modules.in_set(InitShipModules));

    // debug spawn command module
    app.add_systems(Startup, move |mut spawn_w: EventWriter<SpawnShipModule>| {
        spawn_w.send(SpawnShipModule {
            module_type_id,
            transform: ShipModuleTransform {
                translation: IVec2::ZERO,
                rotation: crate::modules::grid::ModuleRotation::East,
            },
        });
    });
}

/// Marker component for the command module
#[derive(Component, Default)]
pub struct CommandShipModule;

fn init_command_modules(
    mut commands: Commands,
    module_q: Query<Entity, Added<CommandShipModule>>,
    assets: Res<AssetServer>,
) {
    for module_entity in module_q.iter() {
        let mesh = assets.load("ship_modules/colliders/command_module.gltf");

        commands.entity(module_entity).insert((
            ModuleAssets {
                path: "command_module".into(),
                map_offset: Vec2::new(0.0, 0.0),
                map_size: Vec2::new(3., 3.),
            },
            GltfCollider { mesh },
            ModuleVent { open: true },
            ModuleAtmosphere {
                volume: 9.,
                level: 9.,
                breached: false,
            },
        ));

        commands
            .spawn(ShipMapBundle {
                transform: Transform::from_xyz(0., 1.2, -2.)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_8)),
                ..default()
            })
            .set_parent(module_entity);

        commands
            .spawn((
                Tank,
                TankAtmosphere {
                    volume: 30.,
                    level: 30.,
                    enabled: false,
                },
                Transform::from_xyz(-2., 1.8, 2.75)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            ))
            .set_parent(module_entity);

        commands
            .spawn((
                RoomVent { module_entity },
                Transform::from_xyz(2., 1.8, 2.75)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            ))
            .set_parent(module_entity);
    }
}
