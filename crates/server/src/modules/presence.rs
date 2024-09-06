use bevy::prelude::*;

use super::grid::{ShipModuleGrid, SHIP_GRID_SCALE};

pub fn build(app: &mut App) {
    app.add_systems(
        PostUpdate,
        set_module_presence.after(bevy::transform::TransformSystem::TransformPropagate),
    );
}

/// When inserted onto an entity with a [GlobalTransform]
/// you can use [ModulePresence::get] to get the module the
/// entity is in.
#[derive(Component, Default)]
pub struct ModulePresence {
    current_module: Option<Entity>,
}

impl ModulePresence {
    pub fn get(&self) -> Option<Entity> {
        self.current_module
    }
}

fn set_module_presence(
    grid: Res<ShipModuleGrid>,
    mut presence_q: Query<(&mut ModulePresence, &GlobalTransform)>,
) {
    for (mut presence, transform) in presence_q.iter_mut() {
        let grid_x = (transform.translation().x / SHIP_GRID_SCALE) as i32;
        let grid_y = (transform.translation().z / SHIP_GRID_SCALE) as i32;

        presence.current_module = grid.get(IVec2::new(grid_x, grid_y));
    }
}
