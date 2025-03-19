use bevy::prelude::*;
use std::f32::consts::PI;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::thread;
use crate::config;

#[derive(Resource)]
pub struct CommandReceiver {
    pub receiver: Mutex<Receiver<RadarCommand>>,
}

#[derive(Debug, Clone)]
pub struct RadarState {
    pub azimuth: f32,
    pub elevation: f32,
}

impl Default for RadarState {
    fn default() -> Self {
        Self {
            azimuth: 0.0,
            elevation: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct Radar {
    pub current: RadarState,
    pub target: RadarState,
    pub azimuth_velocity: f32,
    pub azimuth_acceleration: f32, 
    pub max_azimuth_velocity: f32,
    pub elevation_velocity: f32,
    pub elevation_acceleration: f32, 
    pub max_elevation_velocity: f32,
}

impl Default for Radar {
    fn default() -> Self {
        Self {
            current: RadarState::default(),
            target: RadarState::default(),
            azimuth_velocity: 0.0,
            azimuth_acceleration: 10.0,
            max_azimuth_velocity: 50.0,
            elevation_velocity: 0.0,
            elevation_acceleration: 10.0,
            max_elevation_velocity: 50.0,
        }
    }
}

#[derive(Component, Debug)]
pub struct FollowOrientation;

pub fn spawn_radar(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
    config: &Res<config::Config>,
) -> Entity {
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
    let radar_cam_box_mat = materials.add(Color::linear_rgb(0.8, 0.2, 0.2));

    let (cmd_tx, cmd_rx) = mpsc::channel::<RadarCommand>();

    commands.insert_resource(CommandReceiver {
        receiver: Mutex::new(cmd_rx),
    });

    thread::spawn(move || {
        run_tcp_listener(cmd_tx);
    });

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
    let mut pivot = commands.spawn((
        Mesh3d(pivot_object),
        Visibility::Hidden,
        Transform::from_xyz(0.0, 1.3, 0.0),
        FollowOrientation,
    ));
    pivot.with_child((
        Mesh3d(radar_hor_pole),
        MeshMaterial3d(radar_pole_mat.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_scale(Vec3::new(0.09, 2.0, 0.09))
            .with_rotation(Quat::from_rotation_z(PI / 2.0)),
        Visibility::Visible,
    ));
    pivot.with_child((
        Mesh3d(radar_antenna.clone()),
        MeshMaterial3d(radar_antennna_mat.clone()),
        Transform::from_xyz(-0.65, 0.0, 0.0).with_rotation(Quat::from_rotation_x(PI / 2.0)),
        Visibility::Visible,
    ));
    pivot.with_child((
        Mesh3d(radar_antenna.clone()),
        MeshMaterial3d(radar_antennna_mat.clone()),
        Transform::from_xyz(0.65, 0.0, 0.0).with_rotation(Quat::from_rotation_x(PI / 2.0)),
        Visibility::Visible,
    ));
    pivot.with_child((
        Mesh3d(radar_cam_box.clone()),
        MeshMaterial3d(radar_cam_box_mat.clone()),
        Transform::from_xyz(config.radar_cam_x_displacement, 0.0, -0.1),
        Visibility::Visible,
    ));

    pivot.id()
}

pub enum RadarCommand {
    Remote { tx: Sender<String> },
    ServoOn { tx: Sender<String> },
    Azimuth { az: f32, tx: Sender<String> },
    Elevation { el: f32, tx: Sender<String> },
    AzimuthQuery { tx: Sender<String> },
    ElevationQuery { tx: Sender<String> },
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
    let mut buffer = [0u8; 1024]; // Buffer for incoming data
    let mut data = Vec::new(); // Accumulates command bytes

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Client disconnected
            Ok(n) => {
                data.extend_from_slice(&buffer[..n]); // Append new data

                while let Some(pos) = data.iter().position(|&b| b == b'\r') {
                    let command_bytes = data.drain(..=pos).collect::<Vec<u8>>(); // Extract command (including \r)

                    // Convert bytes to a string
                    if let Ok(command_str) =
                        str::from_utf8(&command_bytes[..command_bytes.len() - 1])
                    {
                        // Remove \r
                        let line = command_str.trim().to_uppercase();

                        if line.starts_with("AZIMUTH") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() == 2 {
                                let (reply_tx, reply_rx) = mpsc::channel();
                                if let Ok(az) = parts[1].parse::<f32>() {
                                    let cmd = RadarCommand::Azimuth { az, tx: reply_tx };
                                    if let Err(e) = cmd_tx.send(cmd) {
                                        eprintln!("Failed to send AZIMUTH command: {:?}", e);
                                    } else {
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
                                    if let Ok(response) = reply_rx.recv() {
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
                                        if let Ok(o) = reply_rx.recv() {
                                            let _ = stream.write_all(o.as_bytes());
                                        }
                                    }
                                }
                            } else if parts.len() == 1 {
                                let (reply_tx, reply_rx) = mpsc::channel();
                                let cmd = RadarCommand::ElevationQuery { tx: reply_tx };
                                if let Err(e) = cmd_tx.send(cmd) {
                                    eprintln!("Failed to send ELEVATION QUERY command: {:?}", e);
                                } else {
                                    if let Ok(response) = reply_rx.recv() {
                                        let _ = stream.write_all(response.as_bytes());
                                    }
                                }
                            }
                        } else if line.starts_with("REMOTE") {
                            let (reply_tx, reply_rx) = mpsc::channel();
                            let cmd = RadarCommand::Remote { tx: reply_tx };
                            if let Err(e) = cmd_tx.send(cmd) {
                                eprintln!("Failed to send REMOTE command: {:?}", e);
                            } else {
                                if let Ok(o) = reply_rx.recv() {
                                    let _ = stream.write_all(o.as_bytes());
                                }
                            }
                        } else if line.starts_with("SERVOON") {
                            let (reply_tx, reply_rx) = mpsc::channel();
                            let cmd = RadarCommand::ServoOn { tx: reply_tx };
                            if let Err(e) = cmd_tx.send(cmd) {
                                eprintln!("Failed to send SERVOON command: {:?}", e);
                            } else {
                                if let Ok(o) = reply_rx.recv() {
                                    let _ = stream.write_all(o.as_bytes());
                                }
                            }
                        } else {
                            let _ = stream.write_all(b"Unknown command\n");
                        }
                    } else {
                        eprintln!("Invalid UTF-8 received");
                    }
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
            RadarCommand::Remote { tx } => {
                println!("Handle remote command");
                let _ = tx.send("O\r\n".to_string());
            }
            RadarCommand::ServoOn { tx } => {
                let _ = tx.send("No Errors.\r\n".to_string());
            }
            RadarCommand::Azimuth { az, tx } => {
                println!("Setting azimuth to {:.2}", az);
                radar.target.azimuth = az;
                let _ = tx.send("No Errors.\r\n".to_string());
            }
            RadarCommand::Elevation { el, tx } => {
                println!("Setting elevation to {:.2}", el);
                radar.target.elevation = el;
                let _ = tx.send("No Errors.\r\n".to_string());
            }
            RadarCommand::AzimuthQuery { tx } => {
                let s = format!("{:.2}\r\n", radar.current.azimuth);
                let _ = tx.send(s);
            }
            RadarCommand::ElevationQuery { tx } => {
                let s = format!("{:.2}\r\n", radar.current.elevation);
                let _ = tx.send(s);
            }
        }
    }
}

pub fn update_radar(
    mut radar: ResMut<Radar>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &FollowOrientation)>
) {
    if radar.target.azimuth == radar.current.azimuth
        && radar.target.elevation == radar.current.elevation
    {
        return;
    }

    let ds = time.delta_secs();

    (radar.current.azimuth, radar.azimuth_velocity) = update(
        radar.current.azimuth, 
        radar.target.azimuth,
        radar.azimuth_velocity,
        radar.azimuth_acceleration, 
        radar.max_azimuth_velocity, 
        ds,
    );

    (radar.current.elevation, radar.elevation_velocity) = update(
        radar.current.elevation, 
        radar.target.elevation,
        radar.elevation_velocity, 
        radar.elevation_acceleration, 
        radar.max_elevation_velocity, 
        ds,
    );

    let angle_az = radar.current.azimuth.to_radians();
    let angle_el = radar.current.elevation.to_radians();

    for (mut transform, _follow) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_y(-angle_az) * Quat::from_rotation_x(angle_el);
    }
}

pub fn update(
    current: f32,
    target: f32,
    velocity: f32,
    acceleration: f32,
    max_velocity: f32,
    delta_secs: f32,
) -> (f32, f32) {
    let mut v = velocity;
    let mut delta_angle = target - current;
    if delta_angle > 180.0 {
        delta_angle -= 360.0;
    } else if delta_angle < -180.0 {
        delta_angle += 360.0;
    }
    if delta_angle.abs() < 0.01 {
        return (target, 0.0);
    }
    let direction = delta_angle.signum();
    let distance_to_stop = (v * v) / (2.0 * acceleration);
    if delta_angle.abs() > distance_to_stop {
        v += direction * acceleration * delta_secs;
        v = v.clamp(-max_velocity, max_velocity);
    } else {
        v -= direction * acceleration * delta_secs;
        if v * direction < 0.0 {
            v = 0.0;
        }
    }
    let new_azimuth = current + v * delta_secs;
    (new_azimuth, v)
}
