use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use common::state::JoinRequest;
use state::ConnectToServer;

pub mod camera;
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
    camera::build(&mut app);

    app.add_plugins(avian3d::debug_render::PhysicsDebugPlugin::default());

    app.add_systems(Startup, (debug_join_server, spawn_debug_ground));

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
