use bevy::{
    log::{Level, LogPlugin, DEFAULT_FILTER},
    prelude::*,
    window::{PresentMode, WindowTheme},
};
use common::{mesh_colliders::GltfColliderPlugin, state::JoinRequest};
use state::ConnectToServer;

pub mod camera;
pub mod elements;
pub mod entity_map;
pub mod modules;
pub mod networking;
pub mod physics;
pub mod player;
pub mod screens;
pub mod state;
pub mod ui;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                level: Level::DEBUG,
                filter: DEFAULT_FILTER.to_string()
                    + ",bevy_render=warn,bevy_app=warn,cosmic_text::buffer=warn",
                ..default()
            })
            .set(AssetPlugin {
                file_path: "../../assets".into(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Ship Game".into(),
                    present_mode: PresentMode::Mailbox,
                    window_theme: Some(WindowTheme::Dark),
                    ..default()
                }),
                ..default()
            }),
    );

    app.add_plugins(GltfColliderPlugin);

    networking::build(&mut app);
    entity_map::build(&mut app);
    state::build(&mut app);
    physics::build(&mut app);
    player::build(&mut app);
    camera::build(&mut app);
    modules::build(&mut app);
    screens::build(&mut app);
    elements::build(&mut app);
    ui::build(&mut app);

    app.insert_resource(AmbientLight {
        brightness: 100.0,
        ..default()
    });

    app.add_plugins(avian3d::debug_render::PhysicsDebugPlugin::default());

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
