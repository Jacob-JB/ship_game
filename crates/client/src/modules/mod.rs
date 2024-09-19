use bevy::prelude::*;

pub mod load;

pub fn build(app: &mut App) {
    load::build(app);
}

/// points to a ship modules sprite entity with it's own transform
#[derive(Component)]
pub struct ModuleMapSprite {
    pub entity: Entity,
}
