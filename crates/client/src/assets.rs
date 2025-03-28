use bevy::prelude::*;

pub fn build(app: &mut App) {
    app.init_resource::<GameAssets>();
}

/// contains assets loaded on startup
#[derive(Resource)]
pub struct GameAssets {
    pub valve_handle: Handle<Scene>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        GameAssets {
            valve_handle: world
                .resource::<AssetServer>()
                .load("valve_handle.gltf#Scene0"),
        }
    }
}
