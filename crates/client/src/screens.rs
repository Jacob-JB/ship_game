use bevy::{
    ecs::system::SystemParam,
    image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::{
        camera::{RenderTarget, ScalingMode},
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::{Layer, RenderLayers},
    },
};

/// Enumerated static render layers for 2d elements that can be displayed on a map screen
///
/// 2d cameras can include different layers based on functionality
///
/// Some cameras require unique render layers that aren't shared,
/// use [RenderLayerAllocator] to allocate these layers at runtime
pub enum ScreenRenderLayer {
    /// Render layer for the ship map
    Map = 1,
    /// Render layer for players
    Players = 2,
    /// The first dynamically allocated render layer
    Dynamic = 3,
}

pub fn build(app: &mut App) {
    app.init_resource::<RenderLayerAllocater>();
    app.init_resource::<ScreenMesh>();
}

/// Use this resource to allocate render layers at runtime
#[derive(Resource)]
pub struct RenderLayerAllocater {
    next_layer: usize,
}

impl Default for RenderLayerAllocater {
    fn default() -> Self {
        RenderLayerAllocater {
            next_layer: ScreenRenderLayer::Dynamic as usize,
        }
    }
}

impl RenderLayerAllocater {
    pub fn next(&mut self) -> usize {
        let layer = self.next_layer;
        self.next_layer += 1;
        layer
    }
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
        render_layers: &[Layer],
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
                min_filter: ImageFilterMode::Linear,
                ..default()
            }),
            ..default()
        };

        image.resize(size);

        let image = self.image_assets.add(image);

        let camera_entity = self
            .commands
            .spawn((
                Camera2d::default(),
                Camera {
                    order: -1,
                    target: RenderTarget::Image(image.clone()),
                    clear_color: ClearColorConfig::Custom(Color::BLACK),
                    ..default()
                },
                OrthographicProjection {
                    near: -1000.,
                    far: 1000.,
                    scale,
                    scaling_mode: ScalingMode::Fixed {
                        width: 1.,
                        height: 1.,
                    },
                    ..OrthographicProjection::default_3d()
                },
                Transform::from_translation(translation.extend(0.)),
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
            Mesh3d(self.screen_assets.screen_mesh.clone()),
            MeshMaterial3d(material),
            transform,
            Screen { camera_entity },
        ));

        camera_entity
    }
}
