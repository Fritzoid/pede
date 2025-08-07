# Pede
A virtual radar pedestal written with the bevy 3d engine. Contains a simple tcp protocol and server to manipulate azimuth and elevation. Also contains a extra camera on the pedestal that renders to a texture and creates a RTSP stream that you can connect to (rtsp://127.0.0.1:8554/live) and watch.

# Install
You need rust to build this program. You can find it here https://www.rust-lang.org/tools/install and it should be self explanatory how to install this. On windows, depending on how your machine is configured by your admintrastors, you might need to be adminstrator to install rust properly and you should make sure also that you have the visual studio C++ package prior to installation. I use vscode as an editor. Make sure to install the rust-analyser extention if you do as well. In order to run the RTSP stream, you need to unpack ffmpeg.zip in the ffmpeg directory. There is versions for windows and macs. After that you need to copy the ffmpeg executable and the mediamtx executable from the ffmpeg directory to the target directory (usually this is target/debug or target/release from the root of the repo) corresponding to wheter you build in debug or release mode. 

Ive used this on both windows and mac.

# Working, building and running
cargo build --debug

cargo run --debug

or 

cargo build --release

cargo run --release

# radar-console

you can run the radar-console afterwards with cargo run --release --bin radar-console
You can then enter commands to move the radar and thereby the camera. 
ie: azimuth 179 or elevation 35
azimuth is between 0 and 360 and elevation is between -20 and 90.



