use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    render::{
        camera::{RenderTarget, ScalingMode},
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
        view::RenderLayers,
    },
};

pub fn build(app: &mut App) {
    app.init_resource::<ScreenMesh>();
}

#[derive(Resource)]
struct ScreenMesh {
    screen_mesh: Handle<Mesh>,
}

impl FromWorld for ScreenMesh {
    fn from_world(world: &mut World) -> Self {
        ScreenMesh {
            screen_mesh: world
                .resource_mut::<Assets<Mesh>>()
                .add(Rectangle::new(0.5, 0.5)),
        }
    }
}

#[derive(Component)]
pub struct Screen {
    pub camera_entity: Entity,
}

#[derive(SystemParam)]
pub struct Screens<'w, 's> {
    commands: Commands<'w, 's>,
    image_assets: ResMut<'w, Assets<Image>>,
    material_assets: ResMut<'w, Assets<StandardMaterial>>,
    screen_assets: Res<'w, ScreenMesh>,
}

impl<'w, 's> Screens<'w, 's> {
    /// Initializes an entity as a screen and returns the camera spawned for the screen.
    pub fn create_screen(
        &mut self,
        screen_entity: Entity,
        resolution: UVec2,
        transform: Transform,
        scale: f32,
        translation: Vec2,
        render_layers: &[usize],
    ) -> Entity {
        let size = Extent3d {
            width: resolution.x,
            height: resolution.y,
            ..default()
        };

        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("Screen image"),
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                mag_filter: ImageFilterMode::Nearest,
                min_filter: ImageFilterMode::Nearest,
                ..default()
            }),
            ..default()
        };

        image.resize(size);

        let image = self.image_assets.add(image);

        let camera_entity = self
            .commands
            .spawn((
                Camera2dBundle {
                    camera: Camera {
                        order: -1,
                        target: RenderTarget::Image(image.clone()),
                        clear_color: ClearColorConfig::Custom(Color::BLACK),
                        ..default()
                    },
                    projection: OrthographicProjection {
                        near: -1000.,
                        far: 1000.,
                        scale,
                        scaling_mode: ScalingMode::Fixed {
                            width: 1.,
                            height: 1.,
                        },
                        ..default()
                    },
                    transform: Transform::from_translation(translation.extend(0.)),
                    ..default()
                },
                RenderLayers::from_layers(render_layers),
            ))
            .id();

        let material = self.material_assets.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::WHITE * 10.,
            emissive_texture: Some(image),
            perceptual_roughness: 0.,
            ..default()
        });

        self.commands.entity(screen_entity).insert((
            MaterialMeshBundle {
                mesh: self.screen_assets.screen_mesh.clone(),
                material,
                transform,
                ..default()
            },
            Screen { camera_entity },
        ));

        camera_entity
    }
}
