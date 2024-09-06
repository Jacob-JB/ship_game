use bevy::prelude::*;

use crate::{
    grid_spaces,
    modules::{grid::ShipModuleTransform, mesh::ModuleAssetPath},
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

fn init_command_modules(mut commands: Commands, module_q: Query<Entity, Added<CommandShipModule>>) {
    for module_entity in module_q.iter() {
        commands.entity(module_entity).insert(ModuleAssetPath {
            path: "command_module.gltf".into(),
        });
    }
}
