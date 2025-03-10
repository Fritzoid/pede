use crate::config;
use crate::stream;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

#[derive(Component)]
pub struct RadarCamera;

pub fn spawn_radar_cam(
    mut meshes: ResMut<Assets<Mesh>>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
    mut images: ResMut<Assets<Image>>,
    pivot: Entity,
    frame_buffer: Res<stream::FrameBuffer>,
    config: &Res<config::Config>,
) -> Handle<Image> {
    let radar_cam_pos = Vec3::new(0.0, 1.3, 0.0);
    let size = Extent3d {
        width: frame_buffer.width,
        height: frame_buffer.height,
        depth_or_array_layers: 1,
        ..default()
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let image_handle = images.add(image);
    commands.entity(pivot).with_child((
        Transform::from_xyz(radar_cam_pos.x, radar_cam_pos.y, radar_cam_pos.z),
        Camera3d::default(),
        Camera {
            target: image_handle.clone().into(),
            order: 1,
            ..default()
        },
        PerspectiveProjection {
            fov: config.radar_cam_vertical_fov.to_radians(),
            ..default()
        },
        RadarCamera,
        Visibility::Visible,
    ));
    let radar_screen = meshes.add(Plane3d {
        normal: Dir3::Z,
        half_size: Vec2::new(0.4, 0.2),
        ..default()
    });
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle.clone()),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });
    commands.spawn((
        Mesh3d(radar_screen),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(0.0, 0.3, 0.57).with_rotation(Quat::from_rotation_x(-0.19)),
    ));
    image_handle
}

pub fn force_projection_update(mut query: Query<&mut PerspectiveProjection, With<RadarCamera>>) {
    for mut projection in &mut query {
        *projection = PerspectiveProjection {
            fov: projection.fov,
            aspect_ratio: projection.aspect_ratio,
            near: projection.near,
            far: projection.far,
        };
    }
}
