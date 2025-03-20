use bevy::{color::palettes::css::WHITE, prelude::*};

pub fn build(app: &mut App) {
    app.add_systems(Startup, create_ui);
}

/// Contains the entities of ui nodes created on startup
#[derive(Resource)]
pub struct UiElements {
    pub oxygen_level_entity: Entity,
    pub health_level_entity: Entity,
}

fn create_ui(mut commands: Commands) {
    // ui node that contains all ui elements
    let root_node_entity = commands
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        })
        .id();

    // ui node that is aligned to the bottom left corner
    let lower_left_quad_entity = commands
        .spawn((
            Node {
                left: Val::Px(20.),
                bottom: Val::Px(20.),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(WHITE.with_alpha(0.1).into()),
        ))
        .set_parent(root_node_entity)
        .id();

    // ui node in the bottom left corner containing player health stats stacked vertically
    let player_health_items = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .set_parent(lower_left_quad_entity)
        .id();

    let oxygen_level_entity = commands
        .spawn(Text::new("Oxygen Level"))
        .set_parent(player_health_items)
        .id();

    let health_level_entity = commands
        .spawn(Text::new("Health Level"))
        .set_parent(player_health_items)
        .id();

    commands.insert_resource(UiElements {
        oxygen_level_entity,
        health_level_entity,
    });
}
