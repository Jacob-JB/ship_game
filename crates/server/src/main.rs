use std::time::{Duration, Instant};

use bevy::{
    gltf::GltfPlugin,
    log::{Level, LogPlugin},
    prelude::*,
};
use common::mesh_colliders::GltfColliderPlugin;

pub mod elements;
pub mod modules;
pub mod networking;
pub mod physics;
pub mod player;
pub mod state;

const SERVER_TICK_INTERVAL: Duration = Duration::from_millis(10);

fn main() {
    let mut app = App::new();

    // start logging first
    app.add_plugins(LogPlugin {
        level: Level::DEBUG,
        filter: "wgpu=error,naga=warn,bevy_app=warn".into(),
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
    app.add_plugins(TransformPlugin);
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.add_plugins(GltfPlugin::default());

    app.add_plugins(GltfColliderPlugin);

    networking::build(&mut app);
    physics::build(&mut app);
    player::build(&mut app);
    modules::build(&mut app);
    elements::build(&mut app);

    app.add_systems(Last, tick_delay);

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

#[derive(Resource)]
struct ServerStart(Instant);

impl Default for ServerStart {
    fn default() -> Self {
        ServerStart(Instant::now())
    }
}

fn tick_delay(server_start: Local<ServerStart>, mut last_tick: Local<Duration>) {
    let elapsed = server_start.0.elapsed().saturating_sub(*last_tick);
    let remaining = SERVER_TICK_INTERVAL.saturating_sub(elapsed);

    std::thread::sleep(remaining);

    *last_tick = server_start.0.elapsed();
}
