use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_egui::{egui, EguiPlugin, EguiContexts};
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use std::thread;
use std::sync::Mutex;
use std::io::{Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::f32::consts::PI;

mod stream;
mod env;
mod radar_cam;

#[derive(Resource)]
struct CommandReceiver {
    receiver: Mutex<Receiver<RadarCommand>>,
}

#[derive(Debug, Clone)]
struct RadarState {
    azimuth: f32,
    elevation: f32,
}

#[derive(Resource)]
struct Radar {
    current: RadarState,
    target: RadarState,
}

fn main() {

    let (cmd_tx, cmd_rx) = mpsc::channel::<RadarCommand>();

    thread::spawn(move || {
        run_tcp_listener(cmd_tx);
    });

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
    .insert_resource(CommandReceiver {receiver: Mutex::new(cmd_rx),})
    .insert_resource(Radar {
        current: RadarState {
            azimuth: 0.0,
            elevation: 0.0,
        },
        target: RadarState {
            azimuth: 0.0,
            elevation: 0.0,
        },
    })
    .add_plugins(EguiPlugin)
    .add_plugins(PanOrbitCameraPlugin)
    .add_systems(Startup, setup)
    .add_systems(Update, ui_system)
    .add_systems(FixedUpdate, radar_cam::stream_frames)
    .add_systems(Update, handle_commands)
    .add_systems(Update, update_radar)
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    images: ResMut<Assets<Image>>,
) {
    env::spawn_env(&mut commands, &mut meshes, &mut materials);
    let image = radar_cam::spawn_radar_cam(&mut commands, images);
    spawn_radar(&mut meshes, &mut materials, &mut commands);
    let stdin = stream::start_stream();

    commands.insert_resource(radar_cam::CameraRenderTexture {
        handle: image,
        ffmpeg_stdin: stdin,
    });
}

fn spawn_radar(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
) {
    let radar_mount = meshes.add(Cuboid { half_size: Vec3::new(1.0, 0.4, 0.5), ..default() } );
    let radar_pole = meshes.add(Cylinder::default());
    let radar_hor_pole = meshes.add(Cylinder::default());
    let radar_antenna = meshes.add(Cuboid { half_size: Vec3::new(0.5, 0.05, 0.3), ..default() } );
    let radar_mount_mat = materials.add(Color::linear_rgb(0.5, 0.5, 0.5));
    let radar_pole_mat = materials.add(Color::linear_rgb(0.5, 0.5, 0.5));
    let radar_antennna_mat = materials.add(Color::linear_rgb(0.1, 0.1, 0.1));

    commands.spawn((
        Mesh3d(radar_mount), 
        MeshMaterial3d(radar_mount_mat), 
        Transform::from_xyz(0.0, 0.0, 0.0)));
    commands.spawn((
        Mesh3d(radar_pole), 
        MeshMaterial3d(radar_pole_mat.clone()), 
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(0.1, 4.0, 0.1)),
    ));
    commands.spawn((
        Mesh3d(radar_hor_pole), 
        MeshMaterial3d(radar_pole_mat.clone()), 
        Transform::from_xyz(0.5, 1.3, 0.0).with_scale(Vec3::new(0.09, 2.0, 0.09)).with_rotation(Quat::from_rotation_z(PI/2.0)),
    ));
    commands.spawn((
        Mesh3d(radar_antenna.clone()), 
        MeshMaterial3d(radar_antennna_mat.clone()), 
        Transform::from_xyz(-0.65, 1.3, 0.0).with_rotation(Quat::from_rotation_x(PI/2.0)),
    ));
    commands.spawn((
        Mesh3d(radar_antenna.clone()), 
        MeshMaterial3d(radar_antennna_mat.clone()), 
        Transform::from_xyz(0.65, 1.3, 0.0).with_rotation(Quat::from_rotation_x(PI/2.0)),
    ));
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

enum RadarCommand {
    Azimuth { az: f32, tx: Sender<String> },
    Elevation { el: f32, tx: Sender<String> },
    AzimuthQuery { tx: Sender<f32> },
    ElevationQuery { tx: Sender<f32> },
}

fn run_tcp_listener(cmd_tx: Sender<RadarCommand>) {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Could not bind to address");
    println!("TCP listener running on 127.0.0.1:7878");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // For each connection, clone the sender and spawn a thread.
                let tx_clone = cmd_tx.clone();
                thread::spawn(move || {
                    handle_client(stream, tx_clone);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept a connection: {:?}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, cmd_tx: Sender<RadarCommand>) {
    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream"));
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // End-of-stream (client disconnected)
                break;
            }
            Ok(_) => {
                let line = line.trim().to_uppercase();
                if line.starts_with("AZIMUTH") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() == 2 {
                        let (reply_tx, reply_rx) = mpsc::channel();
                        if let Ok(az) = parts[1].parse::<f32>()
                        {
                            let cmd = RadarCommand::Azimuth { az, tx: reply_tx };
                            if let Err(e) = cmd_tx.send(cmd) {
                                eprintln!("Failed to send AZIMUTH command: {:?}", e);
                            } else {
                                // Wait for the reply from the main thread.
                                if let Ok(o) = reply_rx.recv() {
                                    let _ = stream.write_all(o.as_bytes());
                                }
                            }
                        }
                    }
                    else if parts.len() == 1 {
                        let (reply_tx, reply_rx) = mpsc::channel();
                        let cmd = RadarCommand::AzimuthQuery { tx: reply_tx };
                        if let Err(e) = cmd_tx.send(cmd) {
                            eprintln!("Failed to send AZIMUTH QUERY command: {:?}", e);
                        } else {
                            // Wait for the reply from the main thread.
                            if let Ok(az) = reply_rx.recv() {
                                let response = format!("Current Radar Azimuth: {:.2}\n", az);
                                let _ = stream.write_all(response.as_bytes());
                            }
                        }
                    }
                } else if line.starts_with("ELEVATION") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() == 2 {
                        let (reply_tx, reply_rx) = mpsc::channel();
                        if let Ok(el) = parts[1].parse::<f32>()
                        {
                            let cmd = RadarCommand::Elevation {el, tx: reply_tx};
                            if let Err(e) = cmd_tx.send(cmd) {
                                eprintln!("Failed to send ELEVATION command: {:?}", e);
                            } else {
                                // Wait for the reply from the main thread.
                                if let Ok(o) = reply_rx.recv() {
                                    let _ = stream.write_all(o.as_bytes());
                                }
                            }
                        }
                    }
                    else if parts.len() == 1 {
                        // For a QUERY, create a one-shot channel for the reply.
                        let (reply_tx, reply_rx) = mpsc::channel();
                        let cmd = RadarCommand::ElevationQuery { tx: reply_tx };
                        if let Err(e) = cmd_tx.send(cmd) {
                            eprintln!("Failed to send AZIMUTH QUERY command: {:?}", e);
                        } else {
                            // Wait for the reply from the main thread.
                            if let Ok(az) = reply_rx.recv() {
                                let response = format!("{:.2}\n", az);
                                let _ = stream.write_all(response.as_bytes());
                            }
                        }
                    }
                } else {
                    let _ = stream.write_all(b"Unknown command\n");
                }
            }
            Err(e) => {
                eprintln!("Error reading from client: {:?}", e);
                break;
            }
        }
    }
}

fn handle_commands(cmd_receiver: ResMut<CommandReceiver>) {
    let receiver = cmd_receiver.receiver.lock().unwrap();
    while let Ok(command) = receiver.try_recv() {
        match command {
            RadarCommand::Azimuth { az, tx } => {
                let _ = tx.send("O".to_string());
            }
            RadarCommand::Elevation { el, tx } => {
                let _ = tx.send("O".to_string());
            }
            RadarCommand::AzimuthQuery { tx } => {
                let _ = tx.send(0.0);
            }
            RadarCommand::ElevationQuery { tx } => {
                let _ = tx.send(0.0);
            }
        }
    }
}

fn update_radar(mut radar: ResMut<Radar>, time: Res<Time>) {
    // Speed in degrees per second.
    let speed = 30.0;
    let dt = time.delta_secs();

    // Update azimuth.
    let diff_az = radar.target.azimuth - radar.current.azimuth;
    let step_az = if diff_az.abs() < speed * dt {
        diff_az
    } else {
        diff_az.signum() * speed * dt
    };
    radar.current.azimuth += step_az;

    // Update elevation.
    let diff_el = radar.target.elevation - radar.current.elevation;
    let step_el = if diff_el.abs() < speed * dt {
        diff_el
    } else {
        diff_el.signum() * speed * dt
    };
    radar.current.elevation += step_el;
}