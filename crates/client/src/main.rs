use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use networking::ConnectToEndpoint;

pub mod networking;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(LogPlugin {
        level: Level::DEBUG,
        ..default()
    }));

    networking::build(&mut app);

    app.add_systems(Startup, debug_connect);

    app.run();
}

fn debug_connect(mut connect_w: EventWriter<ConnectToEndpoint>) {
    connect_w.send(ConnectToEndpoint {
        addr: std::env::args()
            .nth(1)
            .expect("Expected server address as first arg")
            .parse()
            .expect("Bad server address format"),
    });
}
