use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::radar_cam;
use crate::radar;


pub fn ui_system(
    mut contexts: EguiContexts, 
    framebuffer: Res<radar_cam::FrameBuffer>, 
    radar_state: Res<radar::Radar>,
    query: Query<&PerspectiveProjection, With<radar_cam::RadarCamera>>
) {
    let ctx = contexts.ctx_mut();

    // Set the background color of the panels to light blue
    ctx.set_visuals(egui::Visuals {
        panel_fill: egui::Color32::from_rgb(173, 216, 230),
        override_text_color: Some(egui::Color32::from_rgb(0, 0, 0)),
        ..Default::default()
    });

    let projection = query.get_single().unwrap();
    let horizontal_fov = 2.0 * ((projection.fov / 2.0).tan() * (framebuffer.width as f32 / framebuffer.height as f32)).atan();

    // Top panel
    egui::TopBottomPanel::top("top_panel")
        .default_height(50.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Framebuffer:");
                ui.label(format!("Width: {}", framebuffer.width));
                ui.label(format!("Height: {}", framebuffer.height));
                ui.label(format!("Vertical fov: {:>6.2}", projection.fov.to_degrees()));
                ui.label(format!("Horizontal fov: {:>6.2}", horizontal_fov.to_degrees()));
                ui.label(format!("Far plane: {:>6.2}", projection.far));
                ui.label(format!("Near plane: {:>6.2}", projection.near));
                ui.label(format!("Aspect ratio: {:>6.2}", projection.aspect_ratio));
            });
        });

    // Bottom panel
    egui::TopBottomPanel::bottom("bottom_panel")
        .default_height(50.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Radarstate:");
                ui.label(format!("Current azimuth: {:>6.2}", radar_state.current.azimuth));
                ui.label(format!("Current elevation: {:>6.2}", radar_state.current.elevation));
                ui.label(format!("Target azimuth: {:>6.2}", radar_state.target.azimuth));
                ui.label(format!("Target elevation: {:>6.2}", radar_state.target.elevation));
            });
        });
}
