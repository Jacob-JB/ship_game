use bevy::{
    ecs::{entity::EntityHashMap, system::SystemParam},
    prelude::*,
};
use common::ServerEntity;

pub fn build(app: &mut App) {
    app.init_resource::<ServerEntityMap>();

    app.add_systems(PostUpdate, remove_despawned_entities);
}

/// a two way map of [ServerEntities](ServerEntity) to local [Entities](Entity)
///
/// you should use [ServerEntityMapper] to spawn server entities locally,
/// any entity that had the [LocalServerEntity] component will be automatically
/// removed from the map when the component is removed or it is despawned
#[derive(Default, Resource)]
pub struct ServerEntityMap {
    server_client: EntityHashMap<Entity>,
    client_server: EntityHashMap<Entity>,
}

impl ServerEntityMap {
    /// inserts a new entry, returns `true` if there was a conflicting existing entry
    ///
    /// if you are calling this because you have spawned an local server entity,
    /// use [ServerEntityMapper] instead
    pub fn insert(&mut self, server_entity: ServerEntity, client_entity: Entity) -> bool {
        use bevy::utils::hashbrown::hash_map::Entry::Vacant;

        match (
            self.server_client.entry(server_entity.into()),
            self.client_server.entry(client_entity),
        ) {
            (Vacant(server_client), Vacant(client_server)) => {
                server_client.insert(client_entity);
                client_server.insert(server_entity.into());
                false
            }
            _ => true,
        }
    }

    /// gets a client entity from a server entity
    pub fn get_client_entity(&self, server_entity: ServerEntity) -> Option<Entity> {
        self.server_client
            .get(&Entity::from(server_entity))
            .copied()
    }

    /// gets a client entity from a server entity
    pub fn get_server_entity(&self, client_entity: Entity) -> Option<ServerEntity> {
        self.client_server
            .get(&client_entity)
            .copied()
            .map(Into::into)
    }

    /// removes an entry using the server entity
    ///
    /// returns the client entity if there was an entry
    pub fn remove_from_server(&mut self, server_entity: ServerEntity) -> Option<Entity> {
        let client_entity = self.server_client.remove(&Entity::from(server_entity))?;

        self.client_server
            .remove(&client_entity)
            .expect("Should have matching entry in reverse direction");

        Some(client_entity)
    }

    /// removes an entry using the client entity
    ///
    /// returns the server entity if there was an entry
    pub fn remove_from_client(&mut self, client_entity: Entity) -> Option<ServerEntity> {
        let server_entity = self.server_client.remove(&client_entity)?;

        self.client_server
            .remove(&server_entity)
            .expect("Should have matching entry in reverse direction");

        Some(server_entity.into())
    }
}

/// should exist on every local entity that is mapped to a server entity
///
/// when this component is removed the entity will be removed from the map
#[derive(Component)]
pub struct LocalServerEntity {
    server_entity: ServerEntity,
}

impl LocalServerEntity {
    pub fn get(&self) -> ServerEntity {
        self.server_entity
    }
}

/// system parameter for spawning and mapping server entities
#[derive(SystemParam)]
pub struct ServerEntityMapper<'w, 's> {
    commands: Commands<'w, 's>,
    pub map: ResMut<'w, ServerEntityMap>,
}

impl<'w, 's> ServerEntityMapper<'w, 's> {
    /// gets the entity for a server entity, spawning it if there isn't one yet
    pub fn get_or_spawn(&mut self, server_entity: ServerEntity) -> Entity {
        if let Some(client_entity) = self.map.get_client_entity(server_entity) {
            client_entity
        } else {
            let client_entity = self
                .commands
                .spawn(LocalServerEntity { server_entity })
                .id();

            debug_assert!(
                !self.map.insert(server_entity, client_entity),
                "Should only be called if there is no conflict"
            );

            client_entity
        }
    }
}

/// removes despawned local server entities from the [ServerEntityMap]
fn remove_despawned_entities(
    mut despawned: RemovedComponents<LocalServerEntity>,
    mut map: ResMut<ServerEntityMap>,
) {
    for client_entity in despawned.read() {
        map.remove_from_client(client_entity);
    }
}
