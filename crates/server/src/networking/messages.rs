use bevy::{ecs::system::SystemParam, prelude::*, utils::HashMap};
use common::networking::{ServerClientMessages, StreamHeader};
use nevy::prelude::*;
use serde::Serialize;

use super::{ClientConnection, ClientDisconnected, ServerEndpoint};

pub type MessageId<T> = nevy::messaging::serialize::MessageId<ServerClientMessages, T>;

/// holds an individual message stream state per client
#[derive(Default)]
pub struct MessageSenderState {
    streams: HashMap<Entity, MessageStreamState<ServerClientMessages>>,
}

/// system parameters needed by [MessageSenderState]
#[derive(SystemParam)]
pub struct MessageSenderParams<'w, 's> {
    endpoint_q: Query<'w, 's, &'static mut BevyEndpoint, With<ServerEndpoint>>,
    client_q: Query<'w, 's, Entity, With<ClientConnection>>,
    disconnected_r: EventReader<'w, 's, ClientDisconnected>,
}

/// wraps a [MessageSenderState] unique to a system local
#[derive(SystemParam)]
pub struct MessageSender<'w, 's> {
    params: MessageSenderParams<'w, 's>,
    state: Local<'s, MessageSenderState>,
}

impl MessageSenderState {
    fn remove_disconnected_clients(&mut self, params: &mut MessageSenderParams) {
        for ClientDisconnected { client_entity } in params.disconnected_r.read() {
            self.streams.remove(client_entity);
        }
    }

    /// should be called once per tick to drive partially sent messages to completion
    pub fn flush(&mut self, params: &mut MessageSenderParams) {
        self.remove_disconnected_clients(params);

        let mut endpoint = params.endpoint_q.single_mut();

        for client_entity in params.client_q.iter() {
            let Some(stream) = self.streams.get_mut(&client_entity) else {
                continue;
            };

            let Some(mut connection) = endpoint.connection_mut(client_entity) else {
                error!(
                    "Couldn't get client connection {:?} from the simulator endpoint when flushing messages",
                    client_entity
                );
                continue;
            };

            let Ok(_) = stream.flush(&mut connection) else {
                error!("Fatal error flushing stream to {:?}", client_entity);
                continue;
            };
        }
    }

    /// attempts to send a message to a client
    ///
    /// will panic if the client doesn't exist or if
    /// the endpoint fails to perform an operation
    ///
    /// returns `true` if the message was accepted and
    /// `false` if the message message was blocked
    pub fn send<T: Serialize + Send + Sync + 'static>(
        &mut self,
        params: &mut MessageSenderParams,
        message_id: MessageId<T>,
        client_entity: Entity,
        message: &T,
    ) -> bool {
        // verify that client exists
        let true = params.client_q.contains(client_entity) else {
            panic!(
                "Tried to send message \"{}\" to disconnected client {:?}",
                std::any::type_name::<T>(),
                client_entity
            );
        };

        let mut endpoint = params.endpoint_q.single_mut();

        // get the connection and stream state
        let Some(mut connection) = endpoint.connection_mut(client_entity) else {
            panic!(
                        "Couldn't get client connection {:?} from the simulator endpoint when sending a message \"{}\"",
                        client_entity,
                        std::any::type_name::<T>()
                    );
        };

        let stream = match self.streams.entry(client_entity) {
            bevy::utils::hashbrown::hash_map::Entry::Occupied(occupied) => occupied.into_mut(),
            bevy::utils::hashbrown::hash_map::Entry::Vacant(vacant) => {
                let Some(stream) = MessageStreamState::new(
                    &mut connection,
                    Description::new_open_description::<QuinnStreamId>(
                        nevy::quic::quinn_proto::Dir::Uni,
                    ),
                    StreamHeader::Messages.into(),
                )
                .expect("shouldn't mismatch stream type") else {
                    panic!("failed to open stream for messaging");
                };

                vacant.insert(stream)
            }
        };

        // send the message
        stream
            .send(&mut connection, message_id, message)
            .expect("Error here is fatal")
    }

    /// closes any streams that don't have partially sent messages
    ///
    /// useful for closing streams that rarely send messages and don't need to be kept open
    ///
    /// calls `description` when a close description is needed
    pub fn close_unused_streams(
        &mut self,
        params: &mut MessageSenderParams,
        mut description: impl FnMut() -> Description,
    ) {
        let mut endpoint = params.endpoint_q.single_mut();

        for client_entity in params.client_q.iter() {
            let bevy::utils::hashbrown::hash_map::Entry::Occupied(entry) =
                self.streams.entry(client_entity)
            else {
                continue;
            };

            if !entry.get().ready() {
                continue;
            }

            let stream_id = entry.remove().end();

            // get the connection
            let Some(mut connection) = endpoint.connection_mut(client_entity) else {
                panic!(
                            "Couldn't get client connection {:?} from the simulator endpoint when closing an unused stream",
                            client_entity
                        );
            };

            let Some(mut stream) = connection
                .send_stream(stream_id)
                .expect("Shouldn't mismatch stream type")
            else {
                panic!(
                    "Couldn't get stream from bevy connection {:?}",
                    client_entity
                );
            };

            stream
                .close(description())
                .expect("Mismatched stream type")
                .expect("Shouldn't fail to close stream");

            debug!("closed an unused messaging stream");
        }
    }
}

impl<'w, 's> MessageSender<'w, 's> {
    /// should be called once per tick to drive partially sent messages to completion
    pub fn flush(&mut self) {
        self.state.flush(&mut self.params);
    }

    /// attempts to send a message to a client
    ///
    /// will panic if the client doesn't exist or if
    /// the endpoint fails to perform an operation
    ///
    /// returns `true` if the message was accepted and
    /// `false` if the message message was blocked
    pub fn send<T: Serialize + Send + Sync + 'static>(
        &mut self,
        message_id: MessageId<T>,
        client_entity: Entity,
        message: &T,
    ) -> bool {
        self.state
            .send(&mut self.params, message_id, client_entity, message)
    }

    /// closes any streams that don't have partially sent messages
    ///
    /// useful for closing streams that rarely send messages and don't need to be kept open
    ///
    /// calls `description` when a close description is needed
    pub fn close_unused_streams(&mut self, description: impl FnMut() -> Description) {
        self.state
            .close_unused_streams(&mut self.params, description);
    }
}
