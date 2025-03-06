use bevy::prelude::*;

use crate::networking::prelude::*;

pub mod ship_map;
pub mod tank;

pub fn build(app: &mut App) {
    app.add_plugins(MessageQueuePlugin::<ElementUpdateMessageQueue>::default());

    ship_map::build(app);
    tank::build(app);
}

/// Marker type for the message queue used for element updates.
///
/// Used by all elements for non latency/bandwidth critical updates,
/// such as spawning and despawning, doors opening and closing, etc
pub struct ElementUpdateMessageQueue;
