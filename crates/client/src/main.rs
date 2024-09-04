use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use common::state::JoinRequest;
use state::ConnectToServer;

pub mod entity_map;
pub mod networking;
pub mod physics;
pub mod player;
pub mod state;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(LogPlugin {
        level: Level::DEBUG,
        ..default()
    }));

    networking::build(&mut app);
    entity_map::build(&mut app);
    state::build(&mut app);
    physics::build(&mut app);
    player::build(&mut app);

    app.add_systems(Startup, debug_join_server);

    app.run();
}

fn debug_join_server(mut connect_w: EventWriter<ConnectToServer>) {
    connect_w.send(ConnectToServer {
        server_addr: std::env::args()
            .nth(1)
            .expect("Expected server address as first arg")
            .parse()
            .expect("Bad server address format"),
        join_request: JoinRequest {
            username: "Some Username".into(),
        },
    });
}
