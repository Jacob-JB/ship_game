use bevy::prelude::*;

pub mod networking;
pub mod playout;

pub fn build(app: &mut App) {
    networking::build(app);
    playout::build(app);
}
