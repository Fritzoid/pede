use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_panorbit_camera::PanOrbitCameraPlugin;

mod config;
mod env;
mod radar;
mod radar_cam;
mod stream;
mod ui;

fn main() {
    let config = config::Config::from_file("config.toml")
        .expect("Failed to load configuration from config.toml");
    let frame_buffer = stream::FrameBuffer::new(
        config.radar_cam_render_width,
        config.radar_cam_render_height,
    );

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::Windowed,
                canvas: Some("#bevy".to_owned()),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(config)
        .insert_resource(frame_buffer)
        .insert_resource(Time::<Fixed>::from_seconds(1.0 / 25.0))
        .insert_resource(radar::Radar::default())
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, ui::ui_system)
        .add_systems(FixedUpdate, stream::stream_frames)
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
    frame_buffer: Res<stream::FrameBuffer>,
    config: Res<config::Config>,
) {
    env::spawn_env(
        &mut commands,
        &mut meshes,
        &mut materials,
        asset_server,
        &config,
    );
    let pivot = radar::spawn_radar(&mut meshes, &mut materials, &mut commands, &config);
    let image = radar_cam::spawn_radar_cam(
        meshes,
        &mut materials,
        &mut commands,
        images,
        pivot,
        frame_buffer,
        &config,
    );
    stream::start_stream(
        &mut commands,
        image,
        config.radar_cam_render_width,
        config.radar_cam_render_height,
    );
}
