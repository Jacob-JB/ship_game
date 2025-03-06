use bevy::prelude::*;

pub mod ship_map;
pub mod tank;

pub fn build(app: &mut App) {
    ship_map::build(app);
    tank::build(app);
}
