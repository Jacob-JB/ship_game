use bevy::prelude::*;

pub mod atmosphere;
pub mod grid;
pub mod module_types;
pub mod networking;
pub mod presence;

pub fn build(app: &mut App) {
    grid::build(app);
    module_types::build(app);
    networking::build(app);
    presence::build(app);
    atmosphere::build(app);
}
