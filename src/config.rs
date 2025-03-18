use bevy::prelude::*;
use serde::Deserialize;
use std::{error::Error, fs};

#[derive(Resource, Deserialize, Debug, Clone)]
pub struct Config {
    pub radar_cam_render_width: u32,
    pub radar_cam_render_height: u32,
    pub radar_cam_vertical_fov: f32,
    pub calibrate_panels_on: bool,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}
