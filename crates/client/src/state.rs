use std::net::SocketAddr;

use bevy::prelude::*;
use common::state::*;

use crate::networking::prelude::*;

pub fn build(app: &mut App) {
    app.init_state::<ClientState>();

    app.add_event::<ConnectToServer>();
    app.add_event::<DisconnectFromServer>();

    app.add_plugins(MessageQueuePlugin::<JoinRequestQueue>::default());

    app.add_systems(Update, connect_to_server);
    app.add_systems(OnEnter(ConnectionState::Connected), send_join_request);
}

/// fire this event to connect to and join a servers game
///
/// will error if not in [ClientState::Disconnected]
#[derive(Event)]
pub struct ConnectToServer {
    pub server_addr: SocketAddr,
    pub join_request: JoinRequest,
}

/// fire this event to disconnect from the current server
#[derive(Event)]
pub struct DisconnectFromServer;

/// holds the join request whilst waiting for connection to server
///
/// only exists in the [ClientState::Connecting] state
#[derive(Resource)]
struct CurrentJoinRequest {
    join_request: JoinRequest,
}

/// system responsible for starting the connection with the server
fn connect_to_server(
    mut commands: Commands,
    client_state: Res<State<ClientState>>,
    mut next_client_state: ResMut<NextState<ClientState>>,
    mut connect_r: EventReader<ConnectToServer>,
    mut connect_w: EventWriter<ConnectToEndpoint>,
) {
    let mut connecting = false;
    for &ConnectToServer {
        server_addr,
        ref join_request,
    } in connect_r.read()
    {
        if connecting | !matches!(client_state.get(), ClientState::Disconnected) {
            error!("ConnectToServer event was fired whilst not disconnected");
            continue;
        }
        connecting = true;
        next_client_state.set(ClientState::Connecting);

        connect_w.send(ConnectToEndpoint { addr: server_addr });
        commands.insert_resource(CurrentJoinRequest {
            join_request: join_request.clone(),
        });
    }
}

/// marker for join request message queue
struct JoinRequestQueue;

/// system responsible for sending the join request
fn send_join_request(
    mut commands: Commands,
    join_request: Res<CurrentJoinRequest>,
    mut messages: QueuedMessageSender<JoinRequestQueue>,
    message_id: Res<MessageId<JoinRequest>>,
) {
    commands.remove_resource::<CurrentJoinRequest>();

    info!("joining server: {}", join_request.join_request.username);

    messages.send(*message_id, join_request.join_request.clone());
}

/// highest level state for the client
#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClientState {
    /// client is not connected to a server
    #[default]
    Disconnected,
    /// client is connecting to a server
    Connecting,
    /// client has connected to the server and is waiting to be initialized
    Joining,
    /// client has joined the game
    Ingame,
}
