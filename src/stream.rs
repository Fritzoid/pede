use std::process::{ChildStdin, Command, Stdio};
use std::thread;
use std::time::Duration;

pub fn start_stream() -> ChildStdin {
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
            "1280x720", // Replace with your texture size
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
            "rtsp",                       // Output format
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
    ffmpeg_stdin
}
