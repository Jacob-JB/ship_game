use bevy::prelude::*;

pub struct VitalityUi {
    pub vitality_node_entity: Entity,
    pub pressure_level_entity: Entity,
    pub oxygen_level_entity: Entity,
    pub health_level_entity: Entity,
}

impl VitalityUi {
    pub fn new(commands: &mut Commands, parent: Entity) -> Self {
        // ui node in the bottom left corner containing player health stats stacked vertically
        let vitality_node_entity = commands
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .set_parent(parent)
            .id();

        let pressure_level_entity = commands
            .spawn(Text::new(""))
            .set_parent(vitality_node_entity)
            .id();

        let oxygen_level_entity = commands
            .spawn(Text::new(""))
            .set_parent(vitality_node_entity)
            .id();

        let health_level_entity = commands
            .spawn(Text::new(""))
            .set_parent(vitality_node_entity)
            .id();

        Self {
            vitality_node_entity,
            pressure_level_entity,
            oxygen_level_entity,
            health_level_entity,
        }
    }
}
