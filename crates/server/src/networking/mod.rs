use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use bevy::prelude::*;
use common::networking::*;
use nevy::prelude::*;

pub fn build(app: &mut App) {
    app.add_plugins((
        EndpointPlugin::default(),
        StreamHeaderPlugin::default(),
        client_server_protocol().build_deserialization(PreUpdate),
        server_client_protocol().build_serialization(),
    ));

    app.add_event::<ClientConnected>();
    app.add_event::<ClientDisconnected>();

    app.add_systems(Startup, spawn_endpoint);
    app.add_systems(
        PreUpdate,
        (on_connections, on_disconnections).after(UpdateEndpoints),
    );
}

/// marker component for the singular server endpoint
///
/// inserted by [spawn_endpoint]
#[derive(Component)]
pub struct ServerEndpoint;

/// marker component for client connections
///
/// inserted by [on_connection]
#[derive(Component)]
pub struct ClientConnection;

/// event fired when a client connects
///
/// fired by [on_connections]
#[derive(Event)]
pub struct ClientConnected {
    pub client_entity: Entity,
}

/// event fired when a client disconnects
///
/// fired by [on_disconnection]
#[derive(Event)]
pub struct ClientDisconnected {
    pub client_entity: Entity,
}

/// system responsible for spawning the server endpoint at startup
fn spawn_endpoint(mut commands: Commands, server_config: Res<crate::ServerConfig>) {
    let endpoint = QuinnEndpoint::new(
        SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), server_config.port),
        None,
        Some(create_server_endpoint_config()),
    )
    .expect("Failed to create quic endpoint");

    info!("Bound to port {}", server_config.port);

    commands.spawn((
        BevyEndpoint::new(endpoint),
        EndpointStreamHeaders,
        EndpointMessagingHeader {
            header: StreamHeader::Messages.into(),
        },
        ClientServerMessages,
        ServerEndpoint,
    ));
}

/// system responsible for filtering [Connected] events for connections on the
/// [ServerEndpoint] to insert [ClientConnection] components and fire [ClientConnected] events
fn on_connections(
    mut commands: Commands,
    endpoint_q: Query<(), With<ServerEndpoint>>,
    mut connected_r: EventReader<Connected>,
    mut connected_w: EventWriter<ClientConnected>,
) {
    for &Connected {
        endpoint_entity,
        connection_entity,
    } in connected_r.read()
    {
        if endpoint_q.contains(endpoint_entity) {
            commands.entity(connection_entity).insert(ClientConnection);

            info!("New Connection: {}", connection_entity);

            connected_w.send(ClientConnected {
                client_entity: connection_entity,
            });
        }
    }
}

/// system responsibe for filtering [Disconnected] events for disconnections on the
/// [ServerEndpoint] to fire [ClientDiswconnected] events
fn on_disconnections(
    endpoint_q: Query<(), With<ServerEndpoint>>,
    mut disconnected_r: EventReader<Disconnected>,
    mut disconnected_w: EventWriter<ClientDisconnected>,
) {
    for &Disconnected {
        endpoint_entity,
        connection_entity,
    } in disconnected_r.read()
    {
        if endpoint_q.contains(endpoint_entity) {
            info!("Connection Closed: {}", connection_entity);

            disconnected_w.send(ClientDisconnected {
                client_entity: connection_entity,
            });
        }
    }
}

fn create_server_endpoint_config() -> nevy::quic::quinn_proto::ServerConfig {
    let cert = rcgen::generate_simple_self_signed(vec!["dev.shipgame".to_string()]).unwrap();
    let key = rustls::pki_types::PrivateKeyDer::try_from(cert.key_pair.serialize_der()).unwrap();
    let chain = cert.cert.der().clone();

    let mut tls_config = rustls::ServerConfig::builder_with_provider(std::sync::Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_protocol_versions(&[&rustls::version::TLS13])
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(vec![chain], key)
    .unwrap();

    tls_config.max_early_data_size = u32::MAX;
    tls_config.alpn_protocols = vec![b"h3".to_vec()]; // this one is important

    let quic_tls_config =
        nevy::quic::quinn_proto::crypto::rustls::QuicServerConfig::try_from(tls_config).unwrap();

    let mut server_config =
        nevy::quic::quinn_proto::ServerConfig::with_crypto(std::sync::Arc::new(quic_tls_config));

    let mut transport_config = nevy::quic::quinn_proto::TransportConfig::default();
    transport_config.max_idle_timeout(Some(std::time::Duration::from_secs(10).try_into().unwrap()));
    transport_config.keep_alive_interval(Some(std::time::Duration::from_millis(200)));

    server_config.transport = std::sync::Arc::new(transport_config);

    server_config
}
