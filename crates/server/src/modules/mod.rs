use bevy::prelude::*;

pub mod grid;
pub mod module_types;

pub fn build(app: &mut App) {
    grid::build(app);
    module_types::build(app);
}
