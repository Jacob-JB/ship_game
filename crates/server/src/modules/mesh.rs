use bevy::prelude::*;
use common::modules::LoadModule;

use crate::{networking::prelude::*, state::ReceiveGameUpdates};

pub fn build(app: &mut App) {
    app.add_plugins(MessageQueuePlugin::<LoadModuleMessageQueue>::default());

    app.add_systems(Update, (init_new_modules, init_existing_modules));
}

/// When inserted on an entity, the given scene
/// will be loaded on both the server and client.
///
/// The transform of the scene must be inserted before or at
/// insertion of this component and shouldn't move.
#[derive(Component)]
pub struct ModuleAssetPath {
    pub path: String,
}

/// Marker for the stream for load module messages
pub struct LoadModuleMessageQueue;

/// Responsible for telling clients about new modules.
fn init_new_modules(
    scene_q: Query<(Entity, &ModuleAssetPath, &Transform), Added<ModuleAssetPath>>,
    client_q: Query<Entity, With<ReceiveGameUpdates>>,
    mut messages: QueuedMessageSender<LoadModuleMessageQueue>,
    message_id: Res<MessageId<LoadModule>>,
) {
    for (scene_entity, scene, transform) in scene_q.iter() {
        for client_entity in client_q.iter() {
            messages.send(
                *message_id,
                client_entity,
                LoadModule {
                    path: scene.path.clone(),
                    server_entity: scene_entity.into(),
                    translation: transform.translation,
                    rotation: transform.rotation,
                },
            );
        }
    }
}

/// Responsible for telling clients about existing modules when they join.
fn init_existing_modules(
    scene_q: Query<(Entity, &ModuleAssetPath, &Transform)>,
    client_q: Query<Entity, Added<ReceiveGameUpdates>>,
    mut messages: QueuedMessageSender<LoadModuleMessageQueue>,
    message_id: Res<MessageId<LoadModule>>,
) {
    for client_entity in client_q.iter() {
        for (scene_entity, scene, transform) in scene_q.iter() {
            messages.send(
                *message_id,
                client_entity,
                LoadModule {
                    path: scene.path.clone(),
                    server_entity: scene_entity.into(),
                    translation: transform.translation,
                    rotation: transform.rotation,
                },
            );
        }
    }
}
