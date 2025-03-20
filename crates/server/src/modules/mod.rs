use bevy::prelude::*;

pub mod atmosphere;
pub mod grid;
pub mod module_types;
pub mod networking;

pub fn build(app: &mut App) {
    grid::build(app);
    module_types::build(app);
    networking::build(app);
    atmosphere::build(app);
}
