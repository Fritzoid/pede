use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use std::io::Write;
use std::ops::Deref;
use std::process::ChildStdin;
use std::sync::{Arc, Mutex};
use crate::config;

#[derive(Resource)]
pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Default for FrameBuffer {
    fn default() -> Self {
        let width = 1280;
        let height = 720;

        let buffer_size = width as usize * height as usize * 4;
        Self {
            width,
            height,
            buffer: Arc::new(Mutex::new(vec![0u8; buffer_size])),
        }
    }
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let size = width as usize * height as usize * 4;
        Self {
            width,
            height,
            buffer: Arc::new(Mutex::new(vec![0u8; size])),
        }
    }
}

#[derive(Component)]
pub struct RadarCamera;

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
    frame_buffer: Res<FrameBuffer>,
    config: &Res<config::Config>
) -> Handle<Image> {
    let radar_cam_pos = Vec3::new(0.0, 1.3, 0.0);

    let size = Extent3d {
        width: frame_buffer.width,
        height: frame_buffer.height,
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

    let pp_component = PerspectiveProjection {
        fov: config.radar_cam_vertical_fov.to_radians(),
        ..default()
    };
    
    println!("Fov is: {}", pp_component.fov);
    println!("far is: {}", pp_component.far);
    println!("near is: {}", pp_component.near);
    println!("ar is: {}", pp_component.aspect_ratio);

    commands.entity(pivot).with_child((
        Transform::from_xyz(radar_cam_pos.x, radar_cam_pos.y, radar_cam_pos.z),
        Camera3d::default(),
        Camera {
            target: image_handle.clone().into(),
            order: 1,
            ..default()
        },
        pp_component,
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

pub fn stream_frames(
    mut resource: ResMut<CameraRenderTexture>,
    mut commands: Commands,
    frame_buffer: Res<FrameBuffer>,
) {
    let buffer_clone = frame_buffer.buffer.clone();
    let sc = Screenshot::image(resource.handle.clone());
    commands.spawn(sc).observe(save_to_buffer(buffer_clone));
    let buffer = frame_buffer.buffer.lock().unwrap();
    let _ = resource.ffmpeg_stdin.write(&buffer);
}

pub fn save_to_buffer(buffer: Arc<Mutex<Vec<u8>>>) -> impl FnMut(Trigger<ScreenshotCaptured>) {
    move |trigger| {
        let img = trigger.event().deref().clone();
        let data = &img.data;
        let mut buffer = buffer.lock().unwrap();
        buffer.copy_from_slice(data);
    }
}
