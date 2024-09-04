use std::{collections::VecDeque, marker::PhantomData};

use bevy::{ecs::system::SystemParam, prelude::*};
use serde::Serialize;

use super::{
    messages::{MessageId, MessageSender},
    ConnectionState,
};

/// Adds a message queue for `S`. S is a marker type used to identify the queue.
pub struct MessageQueuePlugin<S>(PhantomData<S>);

impl<S> Default for MessageQueuePlugin<S> {
    fn default() -> Self {
        MessageQueuePlugin(PhantomData)
    }
}

impl<S> Plugin for MessageQueuePlugin<S>
where
    S: Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(MessageQueue::<S> {
            _p: PhantomData,
            queue: VecDeque::new(),
        });

        app.add_systems(
            PostUpdate,
            (
                clear_queue::<S>.run_if(not(in_state(ConnectionState::Connected))),
                drain_queue::<S>.run_if(in_state(ConnectionState::Connected)),
            ),
        );
    }
}

#[derive(Resource)]
struct MessageQueue<S> {
    _p: PhantomData<S>,
    queue: VecDeque<Box<dyn FnMut(&mut MessageSender) -> bool + Send + Sync + 'static>>,
}

fn clear_queue<S: Send + Sync + 'static>(mut queue: ResMut<MessageQueue<S>>) {
    queue.queue.clear()
}

fn drain_queue<S: Send + Sync + 'static>(
    mut queue: ResMut<MessageQueue<S>>,
    mut sender: MessageSender,
) {
    sender.flush();

    while let Some(message) = queue.queue.front_mut() {
        if message(&mut sender) {
            queue.queue.pop_front();
        } else {
            break;
        }
    }
}

/// used for ordered message sending
#[derive(SystemParam)]
pub struct QueuedMessageSender<'w, S>
where
    S: Send + Sync + 'static,
{
    queue: Option<ResMut<'w, MessageQueue<S>>>,
}

impl<'w, S> QueuedMessageSender<'w, S>
where
    S: Send + Sync + 'static,
{
    /// Queues a message, adding to to the queue for `S`.
    ///
    /// The queue will be sent on one stream, in order.
    /// The queue will be cleared if there is no server connected.
    ///
    /// Will log an error if no server is connected
    ///
    /// Requires that [MessageQueuePlugin<S>] has been added.
    pub fn send<T: Serialize + Send + Sync + 'static>(
        &mut self,
        message_id: MessageId<T>,
        message: T,
    ) {
        if let Some(queue) = self.queue.as_mut() {
            queue
                .queue
                .push_back(Box::new(move |sender: &mut MessageSender| {
                    sender.send(message_id, &message)
                }));
        } else {
            error!(
                "a message \"{}\" attempted to add to queue \"{}\" but no server was connected yet",
                std::any::type_name::<T>(),
                std::any::type_name::<S>(),
            );
        }
    }
}
