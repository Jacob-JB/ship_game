use bevy::{ecs::system::SystemParam, prelude::*};
use serde::Serialize;
use std::{collections::VecDeque, marker::PhantomData};

use super::{
    messages::{MessageId, MessageSender},
    ClientConnected, InitializeClients,
};

/// adds an ordered message queue of `S` to the app
///
/// [OrderedMessageSender<S>] can then be used
pub struct MessageQueuePlugin<S>(PhantomData<S>);

impl<S> Default for MessageQueuePlugin<S> {
    fn default() -> Self {
        MessageQueuePlugin(PhantomData)
    }
}

impl<S: Send + Sync + 'static> Plugin for MessageQueuePlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            insert_stream_queues::<S>.after(InitializeClients),
        );

        app.add_systems(PostUpdate, drain_queues::<S>);
    }
}

fn insert_stream_queues<S: Send + Sync + 'static>(
    mut commands: Commands,
    mut connected_r: EventReader<ClientConnected>,
) {
    for &ClientConnected { client_entity } in connected_r.read() {
        commands.entity(client_entity).insert(MessageQueue::<S> {
            _p: PhantomData,
            queue: VecDeque::new(),
        });
    }
}

#[derive(Component)]
struct MessageQueue<S> {
    _p: PhantomData<S>,
    queue: VecDeque<Box<dyn FnMut(&mut MessageSender, Entity) -> bool + Send + Sync + 'static>>,
}

fn drain_queues<S: Send + Sync + 'static>(
    mut client_q: Query<(Entity, &mut MessageQueue<S>)>,
    mut messages: MessageSender,
) {
    messages.flush();

    for (client_entity, mut queue) in client_q.iter_mut() {
        while let Some(sender) = queue.queue.front_mut() {
            if sender(&mut messages, client_entity) {
                queue.queue.pop_front();
            } else {
                break;
            }
        }
    }
}

/// system parameter for sending ordered messages on the `S` stream
///
/// messages are received normally
#[derive(SystemParam)]
pub struct QueuedMessageSender<'w, 's, S: Send + Sync + 'static> {
    client_q: Query<'w, 's, &'static mut MessageQueue<S>>,
}

impl<'w, 's, S: Send + Sync + 'static> QueuedMessageSender<'w, 's, S> {
    /// queues a message to be sent to a client in order
    ///
    /// will queue the message if bandwidth isn't available
    pub fn send<T: Serialize + Send + Sync + 'static>(
        &mut self,
        message_id: MessageId<T>,
        client_entity: Entity,
        message: T,
    ) {
        let Ok(mut queue) = self.client_q.get_mut(client_entity) else {
            panic!(
                "Tried to send message \"{}\" to client {} that didn't have a queue component for \"{}\"",
                std::any::type_name::<T>(),
                client_entity,
                std::any::type_name::<S>(),
            );
        };

        queue.queue.push_back(Box::new(
            move |sender: &mut MessageSender, client_entity| {
                sender.send(message_id, client_entity, &message)
            },
        ));
    }
}
