use bevy::prelude::*;

pub fn build(app: &mut App) {}

/// marker component inserted onto all clients
/// that should receive game updates
#[derive(Component)]
pub struct ReceiveGameUpdates;
