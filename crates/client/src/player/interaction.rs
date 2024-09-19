use avian3d::prelude::*;
use bevy::{color::palettes::css::*, prelude::*};
use common::GameLayer;

use crate::camera::MainCamera;

const INTERACTION_DISTANCE: f32 = 2.;

pub fn build(app: &mut App) {
    app.add_systems(Update, (set_interaction_target, debug_interaction).chain());
}

/// Insert onto an entity with a collider to enable interaction detection
///
/// The entity must have the [GameLayer::Interaction] collision layer to be interactable
#[derive(Component, Clone)]
pub struct Interactable;

/// Inserted onto [Interactable] entities that the player camera is interacting with
#[derive(Component)]
pub struct InteractionTarget;

fn set_interaction_target(
    mut commands: Commands,
    target_q: Query<Entity, With<InteractionTarget>>,
    interactable_q: Query<(), With<Interactable>>,
    camera_q: Query<&Transform, With<MainCamera>>,
    spatial_query: SpatialQuery,
) {
    for target_entity in target_q.iter() {
        commands.entity(target_entity).remove::<InteractionTarget>();
    }

    let Ok(camera_transform) = camera_q.get_single() else {
        return;
    };

    if let Some(hit) = spatial_query.cast_ray(
        camera_transform.translation,
        camera_transform.forward(),
        INTERACTION_DISTANCE,
        true,
        SpatialQueryFilter::from_mask([GameLayer::Interaction]),
    ) {
        if interactable_q.contains(hit.entity) {
            commands.entity(hit.entity).insert(InteractionTarget);
        }
    }
}

fn debug_interaction(mut interactable_q: Query<(&mut DebugRender, Has<InteractionTarget>)>) {
    for (mut debug_render, is_target) in interactable_q.iter_mut() {
        debug_render.collider_color = Some(if is_target {
            PURPLE.into()
        } else {
            ORANGE.into()
        });
    }
}
