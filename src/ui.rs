use crate::radar;
use crate::radar_cam;
use crate::stream;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub fn ui_system(
    mut contexts: EguiContexts,
    framebuffer: Res<stream::FrameBuffer>,
    radar_state: Res<radar::Radar>,
    query: Query<&Projection, With<radar_cam::RadarCamera>>,
) {
    let ctx = contexts.ctx_mut();

    // Set the background color of the panels to light blue
    ctx.set_visuals(egui::Visuals {
        panel_fill: egui::Color32::from_rgb(173, 216, 230),
        override_text_color: Some(egui::Color32::from_rgb(0, 0, 0)),
        ..Default::default()
    });

    let mut hfov: f32 = 0.0;
    let mut vfov: f32 = 0.0;
    let mut aspect_ratio = 0.0;
    let mut near = 0.0;
    let mut far = 0.0;

    let projection = query.single().unwrap();

    match projection {
        Projection::Perspective(perspective) => {
            let horizontal_fov: f32 = 2.0
                * ((perspective.fov / 2.0).tan()
                    * (framebuffer.width as f32 / framebuffer.height as f32))
                    .atan();
            hfov = horizontal_fov.to_degrees();
            vfov = perspective.fov.to_degrees();
            aspect_ratio = framebuffer.width as f32 / framebuffer.height as f32;
            near = perspective.near;
            far = perspective.far;
        }
        _ => {}
    }

    egui::TopBottomPanel::top("top_panel")
        .default_height(50.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Framebuffer:");
                ui.label(format!("Width: {}", framebuffer.width));
                ui.label(format!("Height: {}", framebuffer.height));
                ui.label(format!("Vertical fov: {:>6.2}", vfov));
                ui.label(format!("Horizontal fov: {:>6.2}", hfov));
                ui.label(format!("Far plane: {:>6.2}", far));
                ui.label(format!("Near plane: {:>6.2}", near));
                ui.label(format!("Aspect ratio: {:>6.2}", aspect_ratio));
            });
        });

    // Bottom panel
    egui::TopBottomPanel::bottom("bottom_panel")
        .default_height(50.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Radarstate:");
                ui.label(format!(
                    "Current azimuth: {:>6.2}",
                    radar_state.current.azimuth
                ));
                ui.label(format!(
                    "Current elevation: {:>6.2}",
                    radar_state.current.elevation
                ));
                ui.label(format!(
                    "Target azimuth: {:>6.2}",
                    radar_state.target.azimuth
                ));
                ui.label(format!(
                    "Target elevation: {:>6.2}",
                    radar_state.target.elevation
                ));
            });
        });
}
