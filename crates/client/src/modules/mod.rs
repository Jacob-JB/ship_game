use bevy::prelude::*;

pub mod mesh;

pub fn build(app: &mut App) {
    mesh::build(app);
}
