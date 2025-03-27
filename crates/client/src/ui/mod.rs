use bevy::{color::palettes::css::WHITE, prelude::*};
use vitality::VitalityUi;

pub mod vitality;

pub fn build(app: &mut App) {
    app.add_systems(Startup, create_ui);
}

/// Contains the entities of ui nodes created on startup
#[derive(Resource)]
pub struct UiElements {
    pub vitality: VitalityUi,
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

    let vitality = VitalityUi::new(&mut commands, lower_left_quad_entity);

    commands.insert_resource(UiElements { vitality });
}
