use bevy::prelude::*;

use crate::networking::prelude::*;

pub mod ship_map;

pub fn build(app: &mut App) {
    app.add_plugins(MessageQueuePlugin::<ElementUpdateMessageQueue>::default());

    ship_map::build(app);
}

/// Marker type for the message queue used for element updates.
///
/// They share a message queue for simplicity.
pub struct ElementUpdateMessageQueue;
