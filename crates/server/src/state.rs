use bevy::prelude::*;

/// marker component inserted onto all clients
/// that should receive game updates
#[derive(Component)]
pub struct ReceiveGameUpdates;
