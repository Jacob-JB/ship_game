use bevy::prelude::*;

pub fn build(app: &mut App) {
    app.add_systems(Startup, create_ui);
}

/// Contains the entities of ui nodes created on startup
#[derive(Resource)]
pub struct UiElements {
    pub oxygen_level_entity: Entity,
}

fn create_ui(mut commands: Commands) {
    let root_node_entity = commands.spawn(Node::default()).id();

    let oxygen_level_entity = commands
        .spawn(Text::new("Oxygen Level"))
        .set_parent(root_node_entity)
        .id();

    commands.insert_resource(UiElements {
        oxygen_level_entity,
    });
}
