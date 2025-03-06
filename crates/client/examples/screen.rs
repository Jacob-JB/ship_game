use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);

    app.add_systems(Startup, setup);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut standard_material_assets: ResMut<Assets<StandardMaterial>>,
    mut color_material_assets: ResMut<Assets<ColorMaterial>>,
    mut image_assets: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 512,
        height: 512,
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
        ..default()
    };

    image.resize(size);

    let image_handle = image_assets.add(image);

    commands.spawn((
        Camera2d::default(),
        Camera {
            order: -1,
            target: RenderTarget::Image(image_handle.clone()),
            ..default()
        },
        RenderLayers::from_layers(&[1]),
    ));

    let rect_mesh = mesh_assets.add(Rectangle::new(200., 100.));
    let rect_material = color_material_assets.add(Color::linear_rgb(1., 0., 0.));

    commands.spawn((
        Mesh2d(rect_mesh),
        MeshMaterial2d(rect_material),
        RenderLayers::from_layers(&[1]),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1., 5., 3.).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let screen_mesh = mesh_assets.add(Rectangle::new(0.5, 0.5));

    let screen_material = standard_material_assets.add(StandardMaterial {
        emissive_texture: Some(image_handle),
        emissive: LinearRgba::WHITE,
        ..default()
    });

    commands.spawn((Mesh3d(screen_mesh), MeshMaterial3d(screen_material)));
}
