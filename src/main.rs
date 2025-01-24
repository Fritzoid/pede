use bevy::color::palettes::css::{LIGHT_GREEN, RED};
use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy::render::camera::Viewport;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::TextureDimension;
use bevy::render::render_resource::TextureFormat;
use bevy::render::render_resource::TextureUsages;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::window::WindowResized;
use bevy_egui::EguiPlugin;
use bevy_egui::{egui, EguiContexts};
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_panorbit_camera::PanOrbitCamera;
use std::f32::consts::PI;

mod buildings;

#[derive(Component)]
struct GameCamera;

fn main() {

    /* 
    match gstreamer::init() {
        Ok(_) => println!("GStreamer initialized"),
        Err(e) => println!("Error initializing GStreamer: {:?}", e),
    }
    */

    App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            mode: WindowMode::Windowed,
            canvas: Some("#bevy".to_owned()),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(EguiPlugin)
    .add_plugins(PanOrbitCameraPlugin)
    .add_systems(Startup, setup)
    .add_systems(Update, ui_system)
    .add_systems(Update, set_camera_viewports)
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
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
        Transform::from_xyz(0.0, -0.65, 0.0)
    ));

    spawn_trees(&mut meshes, &mut materials, &mut commands);
/*    
    for _ in 0..250 {
        buildings::spawn_random_building(&mut commands, &mut meshes, &mut materials);
    }
*/

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

    spawn_radar(&mut meshes, &mut materials, &mut commands, images);
}

#[derive(Component)]
struct RadarCamera;

#[derive(Component)]
struct RadarCameraToTexture;

fn spawn_trees(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
) {
    const N_TREES: usize = 30;
    let capsule = meshes.add(Capsule3d::default());
    let sphere = meshes.add(Sphere::default());
    let leaves = materials.add(Color::linear_rgb(0.0, 1.0, 0.0));
    let trunk = materials.add(Color::linear_rgb(0.4, 0.2, 0.2));

    let mut spawn_with_offset = |offset: f32| {
        for i in 0..N_TREES {
            let pos = race_track_pos(
                offset,
                (i as f32) / (N_TREES as f32) * std::f32::consts::PI * 2.0,
            );
            let [x, z] = pos.into();
            commands.spawn((
                Mesh3d(sphere.clone()),
                MeshMaterial3d(leaves.clone()),
                Transform::from_xyz(x, -0.3, z).with_scale(Vec3::splat(0.3)),
            ));
            commands.spawn((
                Mesh3d(capsule.clone()),
                MeshMaterial3d(trunk.clone()),
                Transform::from_xyz(x, -0.5, z).with_scale(Vec3::new(0.05, 0.3, 0.05)),
            ));
        }
    };
    spawn_with_offset(0.07);
    spawn_with_offset(-0.07);
}

fn race_track_pos(offset: f32, t: f32) -> Vec2 {
    let x_tweak = 2.0;
    let y_tweak = 3.0;
    let scale = 8.0;
    let x0 = ops::sin(x_tweak * t);
    let y0 = ops::cos(y_tweak * t);
    let dx = x_tweak * ops::cos(x_tweak * t);
    let dy = y_tweak * -ops::sin(y_tweak * t);
    let dl = ops::hypot(dx, dy);
    let x = x0 + offset * dy / dl;
    let y = y0 - offset * dx / dl;
    Vec2::new(x, y) * scale
}

fn spawn_radar(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let radar_body = meshes.add(Cuboid { half_size: Vec3::new(1.0, 0.1, 0.5), ..default() } );
    let radar_body_mat = materials.add(Color::linear_rgb(0.5, 0.5, 0.5));
    commands.spawn((Mesh3d(radar_body), MeshMaterial3d(radar_body_mat), Transform::from_xyz(0.0, 0.0, 0.0)));

    let size = Extent3d {
        width: 512,
        height: 512,
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

    let radar_cam_pos = Vec3::new(0.0, 1.0, 0.0);
    let radar_cam_lookat = Vec3::new(0., 1.0, -10.);
    // Radar Cameras
     commands.spawn((
        Transform::from_xyz(radar_cam_pos.x, radar_cam_pos.y, radar_cam_pos.z)
            .looking_at(Vec3::new(radar_cam_lookat.x, radar_cam_lookat.y, radar_cam_lookat.z), Vec3::Y),
        Camera3d::default(),
        RadarCamera,
    ));
    commands.spawn((
        Transform::from_xyz(radar_cam_pos.x, radar_cam_pos.y, radar_cam_pos.z)
            .looking_at(Vec3::new(radar_cam_lookat.x, radar_cam_lookat.y, radar_cam_lookat.z), Vec3::Y),
        Camera {
            target: image_handle.clone().into(),
            order: 2,
            ..default()
        },
        RadarCameraToTexture,
    ));
}

fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut right_camera: Query<&mut Camera, With<RadarCamera>>,
) {
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        let mut right_camera = right_camera.single_mut();
        let size = window.resolution.physical_width() / 5;
        right_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(window.resolution.physical_width() - size, 0),
            physical_size: UVec2::new(size, size),
            ..default()
        });
        right_camera.order = 1
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

