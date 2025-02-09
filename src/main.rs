use bevy::prelude::*;
use bevy::color::palettes::css::LIGHT_GREEN;
use bevy::window::WindowMode;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::TextureDimension;
use bevy::render::render_resource::TextureFormat;
use bevy::render::render_resource::TextureUsages;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy_egui::EguiPlugin;
use bevy_egui::{egui, EguiContexts};
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_panorbit_camera::PanOrbitCamera;
use rand::Rng;
use std::f32::consts::PI;
use std::process::{Stdio, ChildStdin, Command};
use std::io::Write;
use std::thread;
use std::time::Duration;
use std::ops::Deref;
use std::sync::Mutex;
use std::sync::Arc;
use once_cell::sync::Lazy;

#[derive(Component)]
struct GameCamera;

#[derive(Resource)]
struct CameraRenderTexture {
    handle: Handle<Image>,
    ffmpeg_stdin: ChildStdin,
}

static EXPORT_WIDTH: u32 = 1280;
static EXPORT_HEIGHT: u32 = 720;
static ONE_FRAME: Lazy<Arc<Mutex<Vec<u8>>>> = Lazy::new(
    || {
        let capacity: usize = (EXPORT_HEIGHT * EXPORT_WIDTH * 4).try_into().expect("crap");
        let mut vec = Vec::with_capacity(capacity);
        vec.resize(capacity, 0);
        Arc::new(Mutex::new(vec))
    }
);

fn main() {

    let mut mediamtx = Command::new("mediamtx.exe")
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()
    .expect("Failed to start mediamtx.exe");

    thread::sleep(Duration::from_secs(1));

    if let Some(mediamtx_stderr) = mediamtx.stdout.take() {
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(mediamtx_stderr);
            for line in reader.lines() {
                match line {
                    Ok(log) => println!("mediamtx Log: {}", log),
                    Err(e) => eprintln!("Error reading mediamtx stderr: {}", e),
                }
            }
        });
    }

    App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            mode: WindowMode::Windowed,
            canvas: Some("#bevy".to_owned()),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(Time::<Fixed>::from_seconds(1.0/25.0))
    .add_plugins(EguiPlugin)
    .add_plugins(PanOrbitCameraPlugin)
    .add_systems(Startup, setup)
    .add_systems(Update, ui_system)
    .add_systems(FixedUpdate, stream_frames)
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    images: ResMut<Assets<Image>>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        ..default()
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 3_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(-1.0, -0.7, -1.0), Vec3::X),
    ));
    // Sky
    commands.spawn((
        Mesh3d(meshes.add(Sphere::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true,
            base_color: Color::linear_rgb(0.1, 0.6, 1.0),
            ..default()
        })),
        Transform::default().with_scale(Vec3::splat(-4000.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(LIGHT_GREEN))),
        Transform::from_xyz(0.0, 0.0, 0.0)
    ));

    spawn_trees(&mut meshes, &mut materials, &mut commands);

    commands.spawn((
        Camera3d::default(), 
        PanOrbitCamera { 
            pitch_lower_limit: Some(PI/6.0),
            pitch_upper_limit: Some(PI/4.0),
            zoom_lower_limit: 10.0,
            zoom_upper_limit: Some(500.0),
            ..default() 
        },
        Transform::from_xyz(0.0, 0.0, 15.0)
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        GameCamera,
    ));

    let image = spawn_radar(&mut meshes, &mut materials, &mut commands, images);

    let mut ffmpeg = Command::new("ffmpeg")
    .args([
        "-fflags", "+genpts",
        "-f", "rawvideo",          // Input format is raw video
        "-video_size", "1280x720", // Replace with your texture size
        "-framerate", "25",        // Replace with your target framerate
        "-pixel_format", "bgra",
        "-i", "-",            // Read from stdin
        "-c:v", "libx264",         // Encode to H.264
        "-r", "25",              // Output format
        "-g", "25",
        "-f", "rtsp",              // Output format
        "rtsp://127.0.0.1:8554/live", // RTSP output URL
    ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start FFmpeg");

    let ffmpeg_stdin = ffmpeg.stdin.take().expect("Failed to capture FFmpeg stdin");
    thread::sleep(Duration::from_secs(1));

    commands.insert_resource(CameraRenderTexture {
        handle: image,
        ffmpeg_stdin: ffmpeg_stdin,
    });

     // Spawn a thread to read and display FFmpeg logs
     if let Some(stderr) = ffmpeg.stderr.take() {
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(log) => println!("FFmpeg Log: {}", log),
                    Err(e) => eprintln!("Error reading FFmpeg stderr: {}", e),
                }
            }
        });
    }

}

#[derive(Component)]
struct RadarCamera;

fn spawn_trees(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
) {
    const N_TREES: usize = 75;
    let trunk = meshes.add(Cylinder::default());
    let crown = meshes.add(Sphere::default());
    let trunk_mat = materials.add(Color::linear_rgb(0.4, 0.2, 0.2));
    let crown_mat = materials.add(Color::linear_rgb(0.0, 1.0, 0.0));

    for _i in 0..N_TREES {
        let x = rnd_tree_coord();
        let z = rnd_tree_coord();
        commands.spawn((
            Mesh3d(trunk.clone()),
            MeshMaterial3d(trunk_mat.clone()),
            Transform::from_xyz(x, 0.0, z).with_scale(Vec3::new(0.1, 1.0, 0.1)),
        ));
        commands.spawn((
            Mesh3d(crown.clone()),
            MeshMaterial3d(crown_mat.clone()),
            Transform::from_xyz(x, 1.0, z),
        ));
    }
}

fn rnd_tree_coord() -> f32 {
    let mut rng = rand::rng();
    let random_number: f32 = if rng.random_bool(0.5) {
        rng.random_range(-25.0..=-2.0)
    } else {
        rng.random_range(2.0..=25.0)
    };
    random_number
}

fn spawn_radar(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
    mut images: ResMut<Assets<Image>>,
) -> Handle<Image> {

    let radar_mount = meshes.add(Cuboid { half_size: Vec3::new(1.0, 0.1, 0.5), ..default() } );
    let radar_mount_mat = materials.add(Color::linear_rgb(0.5, 0.5, 0.5));

    commands.spawn((Mesh3d(radar_mount), MeshMaterial3d(radar_mount_mat), Transform::from_xyz(0.0, 0.0, 0.0)));

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
        TextureUsages::TEXTURE_BINDING |TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    let radar_cam_pos = Vec3::new(0.0, 1.0, 0.0);
    let radar_cam_lookat = Vec3::new(0., 1.0, -10.);

    commands.spawn((
        Transform::from_xyz(radar_cam_pos.x, radar_cam_pos.y, radar_cam_pos.z)
            .looking_at(Vec3::new(radar_cam_lookat.x, radar_cam_lookat.y, radar_cam_lookat.z), Vec3::Y),
        Camera3d::default(),
        Camera {
            target: image_handle.clone().into(),
            order: 1,
            ..default()
        },
        RadarCamera,
    ));
    image_handle
}

fn stream_frames(
    mut resource: ResMut<CameraRenderTexture>,
    mut commands: Commands,
) {
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

pub fn ui_system(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();

    // Top panel
    egui::TopBottomPanel::top("top_panel")
        .default_height(50.0)
        .show(ctx, |ui| {
            ui.add(egui::Label::new("Top Panel"));
        });

    // Bottom panel
    egui::TopBottomPanel::bottom("bottom_panel")
        .default_height(50.0)
        .show(ctx, |ui| {
            ui.add(egui::Label::new("Bottom Panel"));
        });

    // Left panel
    egui::SidePanel::left("left_panel")
        .default_width(100.0)
        .show(ctx, |ui| {
            ui.add(egui::Label::new("Left Panel"));
        });

    // Right panel
    egui::SidePanel::right("right_panel")
        .default_width(100.0)
        .show(ctx, |ui| {
            ui.add(egui::Label::new("Right Panel"));
        });

    // Set the background color of the panels to light blue
    ctx.set_visuals(egui::Visuals {
        panel_fill: egui::Color32::from_rgb(173, 216, 230),
        ..Default::default()
    });
}

