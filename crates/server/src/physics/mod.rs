use avian3d::prelude::*;
use bevy::prelude::*;

pub mod networking;

pub fn build(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default());

    networking::build(app);
}
