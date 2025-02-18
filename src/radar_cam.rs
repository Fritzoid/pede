use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use once_cell::sync::Lazy;
use std::io::Write;
use std::ops::Deref;
use std::process::ChildStdin;
use std::sync::{Arc, Mutex};

static EXPORT_WIDTH: u32 = 1280;
static EXPORT_HEIGHT: u32 = 720;
static ONE_FRAME: Lazy<Arc<Mutex<Vec<u8>>>> = Lazy::new(|| {
    let capacity: usize = (EXPORT_HEIGHT * EXPORT_WIDTH * 4).try_into().expect("crap");
    let mut vec = Vec::with_capacity(capacity);
    vec.resize(capacity, 0);
    Arc::new(Mutex::new(vec))
});

#[derive(Component)]
struct RadarCamera;

#[derive(Resource)]
pub struct CameraRenderTexture {
    pub handle: Handle<Image>,
    pub ffmpeg_stdin: ChildStdin,
}

pub fn spawn_radar_cam(
    mut meshes: ResMut<Assets<Mesh>>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
    mut images: ResMut<Assets<Image>>,
    pivot: Entity,
) -> Handle<Image> {

    let radar_cam_pos = Vec3::new(0.0, 1.3, 0.0);
    let radar_cam_lookat = Vec3::new(0., 1.3, -10.);

    let size = Extent3d {
        width: EXPORT_WIDTH,
        height: EXPORT_HEIGHT,
        depth_or_array_layers: 1,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // You need to set these texture usage flags in order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    commands.entity(pivot).with_child((
        Transform::from_xyz(radar_cam_pos.x, radar_cam_pos.y, radar_cam_pos.z).looking_at(
            Vec3::new(radar_cam_lookat.x, radar_cam_lookat.y, radar_cam_lookat.z),
            Vec3::Y,
        ),
        Camera3d::default(),
        Camera {
            target: image_handle.clone().into(),
            order: 1,
            ..default()
        },
        RadarCamera,
        Visibility::Visible
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

pub fn stream_frames(mut resource: ResMut<CameraRenderTexture>, mut commands: Commands) {
    let sc = Screenshot::image(resource.handle.clone());
    commands.spawn(sc).observe(save_to_buffer());
    let buffer = ONE_FRAME.lock().unwrap();
    let _ = resource.ffmpeg_stdin.write(&buffer);
}

pub fn save_to_buffer() -> impl FnMut(Trigger<ScreenshotCaptured>) {
    move |trigger| {
        let img = trigger.event().deref().clone();
        let data = &img.data;
        let mut buffer = ONE_FRAME.lock().unwrap();
        buffer.copy_from_slice(data);
    }
}
