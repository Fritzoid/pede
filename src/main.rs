use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_egui::EguiPlugin;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use std::sync::Mutex;

mod config;
mod env;
mod radar;
mod radar_cam;
mod stream;
mod ui;

fn main() {
    let config = config::Config::from_file("config.toml")
        .expect("Failed to load configuration from config.toml");
    let frame_buffer = radar_cam::FrameBuffer::new(
        config.radar_cam_render_width,
        config.radar_cam_render_height,
    );

    App::new()
        .insert_resource(config)
        .insert_resource(frame_buffer)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::Windowed,
                canvas: Some("#bevy".to_owned()),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_seconds(1.0 / 25.0))
        .insert_resource(radar::Radar {
            current: radar::RadarState {
                azimuth: 0.0,
                elevation: 0.0,
            },
            target: radar::RadarState {
                azimuth: 0.0,
                elevation: 0.0,
            },
        })
        .add_plugins(EguiPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, ui::ui_system)
        .add_systems(FixedUpdate, radar_cam::stream_frames)
        .add_systems(Update, radar::handle_commands)
        .add_systems(Update, radar::update_radar)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    frame_buffer: Res<radar_cam::FrameBuffer>,
    config: Res<config::Config>
) {
    env::spawn_env(&mut commands, &mut meshes, &mut materials, asset_server);

    let (cmd_rx, pivot) = radar::spawn_radar(&mut meshes, &mut materials, &mut commands);
    let image = radar_cam::spawn_radar_cam(
        meshes,
        &mut materials,
        &mut commands,
        images,
        pivot,
        frame_buffer,
        &config
    );
    let stdin = stream::start_stream(config.radar_cam_render_width, config.radar_cam_render_height);

    commands.insert_resource(radar_cam::CameraRenderTexture {
        handle: image,
        ffmpeg_stdin: stdin,
    });
    commands.insert_resource(radar::CommandReceiver {
        receiver: Mutex::new(cmd_rx),
    })
}
