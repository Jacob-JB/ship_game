use bevy::{ecs::system::EntityCommands, prelude::*};

use super::grid::{ShipModuleGrid, ShipModuleGridSpaces, ShipModuleTransform};

pub mod command_module;

/// System set where ship module initialization
/// code should be placed during [Update].
///
/// Ship module marker types get inserted before this set.
#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub struct InitShipModules;

pub fn build(app: &mut App) {
    app.add_event::<SpawnShipModule>();

    app.init_resource::<ShipModuleTypes>();

    command_module::build(app);

    app.add_systems(Update, spawn_ship_modules.before(InitShipModules));
}

/// Adds a module type to the app.
pub fn add_ship_module_type<M: Component + Default>(
    app: &mut App,
    description: ShipModuleDescription,
) -> ShipModuleTypeId {
    let mut modules = app.world_mut().resource_mut::<ShipModuleTypes>();

    let id = ShipModuleTypeId(modules.module_descriptions.len());

    modules.module_descriptions.push(ShipModuleType {
        description,
        spawner: Box::new(|mut commands: EntityCommands| {
            commands.insert(M::default());
        }),
    });

    id
}

/// Resource that contains a list of every ship module.
#[derive(Resource, Default)]
pub struct ShipModuleTypes {
    module_descriptions: Vec<ShipModuleType>,
}

pub struct ShipModuleType {
    description: ShipModuleDescription,
    spawner: Box<dyn Fn(EntityCommands) + Send + Sync + 'static>,
}

/// Information needed for spawning in a ship module.
pub struct ShipModuleDescription {
    pub grid_spaces: ShipModuleGridSpaces,
    pub module_name: String,
}

/// Exists on every ship module.
///
/// Contains the [ShipModuleTypeId] for the module.
#[derive(Component)]
pub struct ShipModule {
    pub module_type_id: ShipModuleTypeId,
}

/// Unique id for a type of ship module.
#[derive(Clone, Copy, Debug)]
pub struct ShipModuleTypeId(usize);

/// Fire this event to spawn a ship module.
///
/// Will log an error if the module doesn't fit and do nothing.
#[derive(Event)]
pub struct SpawnShipModule {
    pub module_type_id: ShipModuleTypeId,
    pub transform: ShipModuleTransform,
}

fn spawn_ship_modules(
    mut commands: Commands,
    mut spawn_module_r: EventReader<SpawnShipModule>,
    modules: Res<ShipModuleTypes>,
    mut grid: ResMut<ShipModuleGrid>,
) {
    for &SpawnShipModule {
        module_type_id,
        transform,
    } in spawn_module_r.read()
    {
        let Some(ShipModuleType {
            description,
            spawner,
        }) = modules.module_descriptions.get(module_type_id.0)
        else {
            error!("Ship module type id out of range {:?}", module_type_id);
            continue;
        };

        let true = description
            .grid_spaces
            .fits_in_grid(transform, grid.as_ref())
        else {
            error!("Tried to place ship module but it doesn't fit");
            continue;
        };

        let module_entity = commands
            .spawn((ShipModule { module_type_id }, transform))
            .id();

        spawner(commands.entity(module_entity));

        description
            .grid_spaces
            .insert_into_grid(transform, module_entity, grid.as_mut());

        debug!(
            "Spawned ship module {} \"{}\" at {} {:?}",
            module_entity, description.module_name, transform.translation, transform.rotation
        );
    }
}
