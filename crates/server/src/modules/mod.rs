use bevy::prelude::*;

pub mod atmosphere;
pub mod grid;
pub mod mesh;
pub mod module_types;
pub mod presence;

pub fn build(app: &mut App) {
    grid::build(app);
    module_types::build(app);
    mesh::build(app);
    presence::build(app);
    atmosphere::build(app);
}
