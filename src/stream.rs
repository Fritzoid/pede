use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use std::io::Write;
use std::ops::Deref;
use std::process::{ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Resource)]
pub struct CameraRenderTexture {
    pub handle: Handle<Image>,
    pub ffmpeg_stdin: ChildStdin,
}

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

pub fn start_stream(commands: &mut Commands, image: Handle<Image>, width: u32, height: u32) {
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

    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-fflags",
            "+genpts",
            "-fflags",
            "nobuffer",
            "-f",
            "rawvideo", // Input format is raw video
            "-video_size",
            &format!("{}x{}", width, height), // Replace with your texture size
            "-framerate",
            "25", // Replace with your target framerate
            "-use_wallclock_as_timestamps",
            "1",
            "-pixel_format",
            "bgra",
            "-i",
            "-", // Read from stdin
            "-c:v",
            "libx264", // Encode to H.264
            "-r",
            "25", // Output format
            "-g",
            "25",
            "-pix_fmt",
            "yuv420p",
            "-preset",
            "ultrafast",
            "-f",
            "rtsp", // Output format
            "-rtsp_transport",
            "udp",
            "rtsp://127.0.0.1:8554/live", // RTSP output URL
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start FFmpeg");

    let ffmpeg_stdin = ffmpeg.stdin.take().expect("Failed to capture FFmpeg stdin");
    thread::sleep(Duration::from_secs(1));

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

    commands.insert_resource(CameraRenderTexture {
        handle: image,
        ffmpeg_stdin: ffmpeg_stdin,
    });
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

fn save_to_buffer(buffer: Arc<Mutex<Vec<u8>>>) -> impl FnMut(Trigger<ScreenshotCaptured>) {
    move |trigger| {
        let img = trigger.event().deref().clone();
        let data = &img.data;
        let mut buffer = buffer.lock().unwrap();
        buffer.copy_from_slice(data);
    }
}
