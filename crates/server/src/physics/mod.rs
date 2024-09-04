use bevy::prelude::*;

pub mod networking;

pub fn build(app: &mut App) {
    networking::build(app);
}
