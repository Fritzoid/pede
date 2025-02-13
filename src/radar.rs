use bevy::prelude::*;
use std::f32::consts::PI;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::thread;

#[derive(Resource)]
pub struct CommandReceiver {
    pub receiver: Mutex<Receiver<RadarCommand>>,
}

#[derive(Debug, Clone)]
pub struct RadarState {
    pub azimuth: f32,
    pub elevation: f32,
}

#[derive(Resource)]
pub struct Radar {
    pub current: RadarState,
    pub target: RadarState,
}

#[derive(Component, Debug)]
pub struct FollowOrientation;

pub fn spawn_radar(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
) -> Receiver<RadarCommand> {
    let radar_mount = meshes.add(Cuboid {
        half_size: Vec3::new(1.0, 0.4, 0.5),
        ..default()
    });
    let radar_pole = meshes.add(Cylinder::default());
    let radar_hor_pole = meshes.add(Cylinder::default());
    let radar_antenna = meshes.add(Cuboid {
        half_size: Vec3::new(0.5, 0.05, 0.3),
        ..default()
    });
    let radar_cam_box = meshes.add(Cuboid {
        half_size: Vec3::new(0.05, 0.05, 0.2),
        ..default()
    });
    let radar_mount_mat = materials.add(Color::linear_rgb(0.5, 0.5, 0.5));
    let radar_pole_mat = materials.add(Color::linear_rgb(0.5, 0.5, 0.5));
    let radar_antennna_mat = materials.add(Color::linear_rgb(0.1, 0.1, 0.1));
    let radar_cam_box_mat = materials.add(Color::linear_rgb(0.8, 0.8, 0.8));

    commands.spawn((
        Mesh3d(radar_mount),
        MeshMaterial3d(radar_mount_mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    commands.spawn((
        Mesh3d(radar_pole),
        MeshMaterial3d(radar_pole_mat.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(0.1, 4.0, 0.1)),
    ));

    let pivot_object = meshes.add(Cuboid::default());
    let mut pivot_id = commands.spawn((
        Mesh3d(pivot_object),
        Visibility::Hidden,
        Transform::from_xyz(0.0, 1.3, 0.0),
        FollowOrientation
    ));
    pivot_id.with_child((
        Mesh3d(radar_hor_pole),
        MeshMaterial3d(radar_pole_mat.clone()),
        Transform::from_xyz(0.5, 0.0, 0.0)
            .with_scale(Vec3::new(0.09, 2.0, 0.09))
            .with_rotation(Quat::from_rotation_z(PI / 2.0)),
        Visibility::Visible
    ));
    pivot_id.with_child((
        Mesh3d(radar_antenna.clone()),
        MeshMaterial3d(radar_antennna_mat.clone()),
        Transform::from_xyz(-0.65, 0.0, 0.0).with_rotation(Quat::from_rotation_x(PI / 2.0)),
        Visibility::Visible
    ));
    pivot_id.with_child((
        Mesh3d(radar_antenna.clone()),
        MeshMaterial3d(radar_antennna_mat.clone()),
        Transform::from_xyz(0.65, 0.0, 0.0).with_rotation(Quat::from_rotation_x(PI / 2.0)),
        Visibility::Visible
    ));
    pivot_id.with_child((
        Mesh3d(radar_cam_box.clone()),
        MeshMaterial3d(radar_cam_box_mat.clone()),
        Transform::from_xyz(1.3, 0.0, 0.0),
        Visibility::Visible
    ));

    let (cmd_tx, cmd_rx) = mpsc::channel::<RadarCommand>();

    thread::spawn(move || {
        run_tcp_listener(cmd_tx);
    });

    cmd_rx
}

pub enum RadarCommand {
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
                        if let Ok(az) = parts[1].parse::<f32>() {
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
                    } else if parts.len() == 1 {
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
                        if let Ok(el) = parts[1].parse::<f32>() {
                            let cmd = RadarCommand::Elevation { el, tx: reply_tx };
                            if let Err(e) = cmd_tx.send(cmd) {
                                eprintln!("Failed to send ELEVATION command: {:?}", e);
                            } else {
                                // Wait for the reply from the main thread.
                                if let Ok(o) = reply_rx.recv() {
                                    let _ = stream.write_all(o.as_bytes());
                                }
                            }
                        }
                    } else if parts.len() == 1 {
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

pub fn handle_commands(mut radar: ResMut<Radar>, cmd_receiver: ResMut<CommandReceiver>) {
    let receiver = cmd_receiver.receiver.lock().unwrap();
    while let Ok(command) = receiver.try_recv() {
        match command {
            RadarCommand::Azimuth { az, tx } => {
                println!("Setting azimuth to {:.2}", az);
                radar.target.azimuth = az;
                let _ = tx.send("O".to_string());
            }
            RadarCommand::Elevation { el, tx } => {
                println!("Setting elevation to {:.2}", el);
                radar.target.elevation = el;
                let _ = tx.send("O".to_string());
            }
            RadarCommand::AzimuthQuery { tx } => {
                let _ = tx.send(radar.current.azimuth);
            }
            RadarCommand::ElevationQuery { tx } => {
                let _ = tx.send(radar.current.elevation);
            }
        }
    }
}

pub fn update_radar(mut radar: ResMut<Radar>, time: Res<Time>, mut query: Query<(&mut Transform, &FollowOrientation)>) {
    // Speed in degrees per second.
    let speed_az = 15.0;
    let dt = time.delta_secs();

    // Update azimuth.
    let diff_az = radar.target.azimuth - radar.current.azimuth;
    let step_az = if diff_az.abs() < speed_az * dt {
        diff_az
    } else {
        diff_az.signum() * speed_az * dt
    };
    radar.current.azimuth += step_az;
    
    // Update elevation.
    let diff_el = radar.target.elevation - radar.current.elevation;
    let speed_el = 20.0;
    let step_el = if diff_el.abs() < speed_el * dt {
        diff_el
    } else {
        diff_el.signum() * speed_el * dt
    };
    radar.current.elevation += step_el;

    let angle_az = step_az.to_radians();
    let angle_el = step_el.to_radians();
    for (mut transform, _follow) in query.iter_mut() {
        transform.rotate_around(Vec3::new(0.0, 1.3, 0.0),  Quat::from_rotation_x(angle_el) * Quat::from_rotation_y(angle_az));
    }
}
