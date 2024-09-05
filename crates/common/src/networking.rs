use bevy::prelude::*;
use nevy::prelude::*;

pub mod prelude {}

/// marker component for server -> client messages
#[derive(Component)]
pub struct ServerClientMessages;

/// marker component for client -> server messages
#[derive(Component)]
pub struct ClientServerMessages;

pub fn server_client_protocol() -> ProtocolBuilder<ServerClientMessages> {
    let mut protocol = ProtocolBuilder::new();

    protocol.add_message::<crate::physics::PhysicsSnapshot>();
    protocol.add_message::<crate::physics::TimeSample>();

    protocol.add_message::<crate::player::NewPlayer>();

    protocol
}

pub fn client_server_protocol() -> ProtocolBuilder<ClientServerMessages> {
    let mut protocol = ProtocolBuilder::new();

    protocol.add_message::<crate::state::JoinRequest>();

    protocol.add_message::<crate::player::ClientPlayerUpdate>();

    protocol
}

/// enumerated stream headers
pub enum StreamHeader {
    Messages,
}

impl From<StreamHeader> for u16 {
    fn from(value: StreamHeader) -> Self {
        value as u16
    }
}
