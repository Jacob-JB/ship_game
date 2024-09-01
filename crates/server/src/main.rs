use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};

pub mod networking;

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

    app.run();
}

#[derive(Resource)]
struct ServerConfig {
    pub port: u16,
}

impl ServerConfig {
    fn new() -> Option<Self> {
        let Some(port) = std::env::args().nth(1) else {
            error!("Expected port as first argument");
            return None;
        };

        let Ok(port) = port.parse() else {
            error!("invalid port format \"{}\"", port);
            return None;
        };

        Some(ServerConfig { port })
    }
}
