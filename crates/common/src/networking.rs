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
    protocol.add_message::<crate::modules::LoadModule>();
    protocol.add_message::<crate::elements::ship_map::NewShipMap>();
    protocol.add_message::<crate::elements::ship_map::ShipMapPositionUpdate>();
    protocol.add_message::<crate::elements::tank::NewTank>();
    protocol.add_message::<crate::elements::tank::UpdateTankState>();
    protocol.add_message::<crate::elements::tank::UpdateTankPercentage>();

    protocol
}

pub fn client_server_protocol() -> ProtocolBuilder<ClientServerMessages> {
    let mut protocol = ProtocolBuilder::new();

    protocol.add_message::<crate::state::JoinRequest>();
    protocol.add_message::<crate::player::ClientPlayerUpdate>();
    protocol.add_message::<crate::elements::ship_map::ShipMapMoveRequest>();
    protocol.add_message::<crate::elements::tank::RequestToggleTankEnabled>();

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
