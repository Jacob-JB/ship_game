use bevy::prelude::*;

use crate::networking::prelude::MessageQueuePlugin;

pub mod room_vent;
pub mod ship_map;
pub mod tank;

pub fn build(app: &mut App) {
    ship_map::build(app);
    tank::build(app);
    room_vent::build(app);

    app.add_plugins(MessageQueuePlugin::<ModuleMessages>::default());
}

/// Message queue marker for ordered reliable messages for modules
pub struct ModuleMessages;
