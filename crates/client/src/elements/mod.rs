use bevy::prelude::*;

pub mod ship_map;

pub fn build(app: &mut App) {
    ship_map::build(app);
}
