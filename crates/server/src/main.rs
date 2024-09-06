use bevy::{
    gltf::GltfPlugin,
    log::{Level, LogPlugin},
    prelude::*,
};
use common::mesh_colliders::GltfColliderPlugin;

pub mod modules;
pub mod networking;
pub mod physics;
pub mod player;
pub mod state;

fn main() {
    let mut app = App::new();

    // start logging first
    app.add_plugins(LogPlugin {
        level: Level::DEBUG,
        ..default()
    });

    // load server config
    let Some(server_config) = ServerConfig::new() else {
        return;
    };
    app.insert_resource(server_config);

    // build rest of app
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: "../../assets".into(),
        ..default()
    });
    app.add_plugins(bevy::scene::ScenePlugin);
    app.add_plugins(HierarchyPlugin);
    app.init_asset::<Mesh>();
    app.add_plugins(GltfPlugin::default());

    app.add_plugins(GltfColliderPlugin);

    networking::build(&mut app);
    physics::build(&mut app);
    player::build(&mut app);
    modules::build(&mut app);

    app.run();
}

#[derive(Resource)]
struct ServerConfig {
    pub port: u16,
}

impl ServerConfig {
    fn new() -> Option<Self> {
        let port = if let Some(port) = std::env::args().nth(1) {
            let Ok(port) = port.parse() else {
                error!("invalid port format \"{}\"", port);
                return None;
            };

            port
        } else {
            27510
        };

        Some(ServerConfig { port })
    }
}
