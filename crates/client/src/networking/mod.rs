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
        client_server_protocol().build_serialization(),
        server_client_protocol().build_deserialization(PreUpdate),
    ));

    app.add_event::<ConnectToEndpoint>();

    app.init_state::<ConnectionState>();

    app.add_systems(Startup, spawn_endpoint);
    app.add_systems(
        Update,
        (connect_to_simulators, on_connected, on_disconnected),
    );
}

/// marker component for the client endpoint
///
/// spawned on startup by [spawn_endpoint]
#[derive(Component)]
pub struct ClientEndpoint;

/// marker component for the singular server connection on the [ClientEndpoint]
#[derive(Component)]
pub struct ServerConnection;

/// fire this event to connect the [ClientEndpoint] to a server
#[derive(Event)]
pub struct ConnectToEndpoint {
    pub addr: SocketAddr,
}

/// the connection state of the [ClientEndpoint]
///
/// to connect to an endpoint fire a [ConnectToEndpoint] event
#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy, States)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
}

/// system responsible for spawning the [ClientEndpoint] on startup
fn spawn_endpoint(mut commands: Commands) {
    let endpoint = QuinnEndpoint::new(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0), None, None)
        .expect("Failed to create quic endpoint");

    commands.spawn((
        BevyEndpoint::new(endpoint),
        EndpointStreamHeaders,
        EndpointMessagingHeader {
            header: StreamHeader::Messages.into(),
        },
        ClientServerMessages,
        ClientEndpoint,
    ));
}

/// system responsible for initializing connections with the server
fn connect_to_simulators(
    mut connect_r: EventReader<ConnectToEndpoint>,
    current_state: Res<State<ConnectionState>>,
    mut next_state: ResMut<NextState<ConnectionState>>,
    endpoint_q: Query<Entity, With<ClientEndpoint>>,
    mut connections: Connections,
) {
    let endpoint_entity = endpoint_q.single();

    let mut connecting = false;

    for &ConnectToEndpoint { addr } in connect_r.read() {
        let ConnectionState::Disconnected = current_state.get() else {
            error!("a ConnectToEndpoint event was fired when not disconnected");
            continue;
        };

        if connecting {
            error!("a ConnectToEndpoint event was fired when not disconnected");
            continue;
        }

        let Some(_) = connections
            .connect(
                endpoint_entity,
                Description::new_connect_description::<QuinnEndpoint>((
                    create_connection_config(),
                    addr,
                    "dev.drewridley.com".into(),
                )),
            )
            .expect("Description type should not mismatch")
        else {
            error!("Failed to start connection with server {}", addr);
            continue;
        };

        info!("Connecting to server {}", addr);

        connecting = true;
        next_state.set(ConnectionState::Connecting);
    }
}

/// creates the quinn client config for a connection with a server
///
/// includes tls and transport config
fn create_connection_config() -> nevy::quic::quinn_proto::ClientConfig {
    #[derive(Debug)]
    struct AlwaysVerify;

    impl rustls::client::danger::ServerCertVerifier for AlwaysVerify {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::pki_types::CertificateDer<'_>,
            _intermediates: &[rustls::pki_types::CertificateDer<'_>],
            _server_name: &rustls::pki_types::ServerName<'_>,
            _ocsp_response: &[u8],
            _now: rustls::pki_types::UnixTime,
        ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }

        fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
            vec![
                rustls::SignatureScheme::RSA_PKCS1_SHA256,
                rustls::SignatureScheme::RSA_PKCS1_SHA512,
                rustls::SignatureScheme::RSA_PKCS1_SHA384,
                rustls::SignatureScheme::RSA_PKCS1_SHA1,
                rustls::SignatureScheme::RSA_PSS_SHA256,
                rustls::SignatureScheme::RSA_PSS_SHA384,
                rustls::SignatureScheme::RSA_PSS_SHA512,
                rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
                rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
                rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
                rustls::SignatureScheme::ED25519,
                rustls::SignatureScheme::ED448,
                rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            ]
        }
    }

    let mut tls_config = rustls::ClientConfig::builder_with_provider(std::sync::Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_protocol_versions(&[&rustls::version::TLS13])
    .unwrap()
    .dangerous()
    .with_custom_certificate_verifier(Arc::new(AlwaysVerify))
    .with_no_client_auth();
    tls_config.alpn_protocols = vec![b"h3".to_vec()];

    let quic_tls_config =
        nevy::quic::quinn_proto::crypto::rustls::QuicClientConfig::try_from(tls_config).unwrap();
    let mut quinn_client_config =
        nevy::quic::quinn_proto::ClientConfig::new(std::sync::Arc::new(quic_tls_config));

    let mut transport_config = nevy::quic::quinn_proto::TransportConfig::default();
    transport_config.max_idle_timeout(Some(std::time::Duration::from_secs(10).try_into().unwrap()));
    transport_config.keep_alive_interval(Some(std::time::Duration::from_millis(200)));
    quinn_client_config.transport_config(std::sync::Arc::new(transport_config));

    quinn_client_config
}

/// system responsible for changing state when the [ClientEndpoint] successfully makes a connection
fn on_connected(
    mut commands: Commands,
    mut connected_r: EventReader<Connected>,
    endpoint_q: Query<(), With<ClientEndpoint>>,
    mut next_state: ResMut<NextState<ConnectionState>>,
) {
    for &Connected {
        endpoint_entity,
        connection_entity,
    } in connected_r.read()
    {
        if endpoint_q.contains(endpoint_entity) {
            info!("Successfully connected to server");

            commands.entity(connection_entity).insert(ServerConnection);
            next_state.set(ConnectionState::Connected);
        }
    }
}

/// system responsible for transitioning back to the disconnected [ClientState]
fn on_disconnected(
    mut disconnected_r: EventReader<Disconnected>,
    endpoint_q: Query<(), With<ClientEndpoint>>,
    mut next_state: ResMut<NextState<ConnectionState>>,
) {
    for &Disconnected {
        endpoint_entity, ..
    } in disconnected_r.read()
    {
        if endpoint_q.contains(endpoint_entity) {
            next_state.set(ConnectionState::Disconnected);

            info!("Disconnected from server");
        }
    }
}
