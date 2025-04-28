# Pede
A virtual radar pedestal written with the bevy 3d engine. Contains a simple tcp protocol and server to manipulate azimuth and elevation. Also contains a extra camera on the pedestal that renders to a texture and creates a RTSP stream that you can connect to and watch.

# Install
You need rust to build this program. You can find it here https://www.rust-lang.org/tools/install and it should be self explanatory how to install this. On windows, depending on how your machine is configured by your admintrastors, you might need to be adminstrator to install rust properly and you should make sure also that you have the visual studio C++ package prior to installation. I use vscode as an editor. Make sure to install the rust-analyser extention if you do as well. In order to run the RTSP stream, you need to unpack ffmpeg.zip in the ffmpeg directory. After that you need to copy ffmpeg.exe and mediamtx.exe from the ffmpeg directory to the target directory (usually this is target/debug or target/release from the root of the repo) corresponding to wheter you build in debug or release mode. 

# Working, building and running
cargo build --debug
cargo run --debug 

or 

cargo build --release
cargo run --release

