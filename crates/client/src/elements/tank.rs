use avian3d::prelude::*;
use bevy::prelude::*;
use common::{elements::NewTank, GameLayer};

use crate::{
    entity_map::ServerEntityMapper,
    networking::prelude::*,
    player::interaction::Interactable,
    screens::{RenderLayerAllocater, Screens},
};

const SCREEN_IMAGE_SIZE: UVec2 = UVec2::new(24, 48);

pub fn build(app: &mut App) {
    app.init_resource::<ValveHandleScene>();

    app.add_systems(Update, (spawn_tanks, move_tank_handles));
}

#[derive(Resource)]
struct ValveHandleScene(Handle<Scene>);

impl FromWorld for ValveHandleScene {
    fn from_world(world: &mut World) -> Self {
        ValveHandleScene(
            world
                .resource::<AssetServer>()
                .load("valve_handle.gltf#Scene0"),
        )
    }
}

#[derive(Component)]
pub struct Tank {
    enable_handle_mesh_entity: Entity,
    enable_handle_collider_entity: Entity,
    enabled: bool,
}

const ENABLE_HANDLE_MESH_OFFSET: Vec3 = Vec3::new(0., -0.35, 0.);
const ENABLE_HANDLE_COLLIDER_OFFSET: Vec3 = Vec3::new(0., -0.35, 0.025);

fn spawn_tanks(
    mut commands: Commands,
    mut messages: MessageReceiver<NewTank>,
    mut mapper: ServerEntityMapper,
    mut screens: Screens,
    mut layers: ResMut<RenderLayerAllocater>,
    valve_handle_scene: Res<ValveHandleScene>,
) {
    for NewTank {
        entity,
        translation,
        rotation,
        state,
    } in messages.drain()
    {
        let tank_entity = mapper.get_or_spawn(entity);

        let render_layer = layers.next();

        let tank_transform = Transform {
            translation,
            rotation,
            ..default()
        };

        screens.create_screen(
            tank_entity,
            SCREEN_IMAGE_SIZE,
            Transform {
                scale: Vec3::new(0.5, 1., 1.),
                ..tank_transform
            },
            1.,
            default(),
            &[render_layer],
        );

        let enable_handle_mesh_entity = commands
            .spawn(SceneBundle {
                scene: valve_handle_scene.0.clone(),
                transform: tank_transform.mul_transform(Transform {
                    scale: Vec3::splat(0.25),
                    translation: ENABLE_HANDLE_MESH_OFFSET,
                    rotation: Quat::from_euler(EulerRot::XZY, std::f32::consts::FRAC_PI_2, 0., 0.),
                }),
                ..default()
            })
            .id();

        let collider_transform = tank_transform.mul_transform(Transform {
            translation: ENABLE_HANDLE_COLLIDER_OFFSET,
            ..default()
        });

        let enable_handle_collider_entity = commands
            .spawn((
                Collider::cuboid(0.1, 0.1, 0.05),
                Position(collider_transform.translation),
                Rotation(collider_transform.rotation),
                CollisionLayers::new([GameLayer::Interaction], 0),
                Interactable,
                DebugRender::default(),
            ))
            .id();

        commands.entity(tank_entity).insert(Tank {
            enable_handle_mesh_entity,
            enable_handle_collider_entity,
            enabled: state.enabled,
        });
    }
}

fn move_tank_handles() {}
