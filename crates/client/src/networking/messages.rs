use bevy::{ecs::system::SystemParam, prelude::*};
use common::networking::*;
use nevy::prelude::*;
use serde::Serialize;

use super::{ClientEndpoint, ServerConnection};

pub type MessageId<T> = nevy::messaging::serialize::MessageId<ClientServerMessages, T>;

struct MessageSenderState {
    connection_entity: Entity,
    stream: MessageStreamState<ClientServerMessages>,
}

#[derive(SystemParam)]
pub struct MessageSender<'w, 's> {
    state: Local<'s, Option<MessageSenderState>>,
    endpoint_q: Query<'w, 's, &'static mut BevyEndpoint, With<ClientEndpoint>>,
    connection_q: Query<'w, 's, Entity, With<ServerConnection>>,
}

impl<'w, 's> MessageSender<'w, 's> {
    /// calls the provided callback with a valid stream state
    /// and the connection context for the server
    fn in_connection_context<R>(
        &mut self,
        callback: impl FnOnce(
            Option<(
                &mut MessageStreamState<ClientServerMessages>,
                BevyConnectionMut,
            )>,
        ) -> R,
    ) -> R {
        // get the connection
        let mut endpoint = self.endpoint_q.single_mut();

        let Ok(connection_entity) = self.connection_q.get_single() else {
            *self.state = None;
            return callback(None);
        };

        let Some(mut connection) = endpoint.connection_mut(connection_entity) else {
            error!("simulation connection {:?} exists the simulator endpoint wouldn't return its state", connection_entity);
            return callback(None);
        };

        // get a connection state that is up to date

        let existing_state;

        if let Some(state) = self.state.as_mut() {
            if state.connection_entity == connection_entity {
                existing_state = Some(&mut state.stream);
            } else {
                existing_state = None;
            }
        } else {
            existing_state = None;
        }

        if let Some(stream) = existing_state {
            callback(Some((stream, connection)))
        } else {
            let description = Description::new_open_description::<QuinnStreamId>(
                nevy::quic::quinn_proto::Dir::Uni,
            );

            let Some(mut stream) = MessageStreamState::new(
                &mut connection,
                description,
                StreamHeader::Messages.into(),
            )
            .expect("Shouldn't mismatch type") else {
                error!("failed to open stream for messaging");
                return callback(None);
            };

            let result = callback(Some((&mut stream, connection)));

            *self.state = Some(MessageSenderState {
                connection_entity,
                stream,
            });

            result
        }
    }

    /// should be called once every tick to drive partially sent messages to completion
    pub fn flush(&mut self) {
        self.in_connection_context(|connected| {
            if let Some((stream, mut connection)) = connected {
                stream
                    .flush(&mut connection)
                    .expect("Shouldn't mismatch connection");
            }
        });
    }

    /// attempts to send a message to the simulator
    ///
    /// returns `true` if the message was accepted
    /// or `false` if the message was blocked or if there
    /// is no endpoint connected
    pub fn send<T: Serialize + Send + Sync + 'static>(
        &mut self,
        message_id: MessageId<T>,
        message: &T,
    ) -> bool {
        self.in_connection_context(|connected| {
            if let Some((stream, mut connection)) = connected {
                stream
                    .send(&mut connection, message_id, message)
                    .expect("Error here is fatal")
            } else {
                false
            }
        })
    }

    /// closes the stream if there isn't a partially sent message buffered
    ///
    /// good for streams that aren't used to send messages very often and don't need to be kept open
    pub fn close_unused(&mut self) {
        let Some(state) = self.state.as_ref() else {
            return;
        };

        let true = state.stream.ready() else {
            return;
        };

        let stream = self.state.take().unwrap();
        let stream_id = stream.stream.end();

        let mut endpoint = self.endpoint_q.single_mut();

        let Some(mut connection) = endpoint.connection_mut(stream.connection_entity) else {
            // if the connection is not the same just return
            return;
        };

        let mut stream = connection
            .send_stream(stream_id)
            .expect("Shouldn't mismatch stream type")
            .expect("Messaging stream was closed prematurely");

        if let Err(err) = stream.close(Description::new_send_close_description::<QuinnStreamId>(
            None,
        )) {
            error!("error closing message stream {:?}", err);
        }
    }
}

/// used to receive messages from the current server
#[derive(SystemParam)]
pub struct MessageReceiver<'w, 's, T: Send + Sync + 'static> {
    connection_q: Query<'w, 's, &'static mut ReceivedMessages<T>, With<ServerConnection>>,
}

impl<'w, 's, T: Send + Sync + 'static> MessageReceiver<'w, 's, T> {
    /// returns an iterator that drains the buffer of received messages from the server
    ///
    /// will not panic if not connected to a server
    pub fn drain(&mut self) -> MessageReceiveIter<T> {
        if let Ok(messages) = self.connection_q.get_single_mut() {
            MessageReceiveIter::Some(messages)
        } else {
            MessageReceiveIter::None
        }
    }
}

pub enum MessageReceiveIter<'w, T> {
    None,
    Some(Mut<'w, ReceivedMessages<T>>),
}

impl<'w, T> Iterator for MessageReceiveIter<'w, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let MessageReceiveIter::Some(messages) = self {
            messages.pop()
        } else {
            None
        }
    }
}
