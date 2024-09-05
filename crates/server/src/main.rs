use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};

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

    // expected by [avian3d], not included in [MinimalPlugins]
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Mesh>();
    app.add_plugins(bevy::scene::ScenePlugin);
    app.add_plugins(HierarchyPlugin);

    networking::build(&mut app);
    physics::build(&mut app);
    player::build(&mut app);
    modules::build(&mut app);

    app.add_systems(Startup, spawn_debug_ground);

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

fn spawn_debug_ground(mut commands: Commands) {
    use avian3d::prelude::*;
    use common::GameLayer;

    commands.spawn((
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
        Position(Vec3::new(0., -2., 0.)),
        CollisionLayers::new([GameLayer::World], [GameLayer::Players]),
    ));
}
