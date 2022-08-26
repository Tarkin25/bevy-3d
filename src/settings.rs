use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::game::chunk::mesh_builder::MeshBuilderSettings;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::default())
            .add_system(update_render_distance)
            .add_system(show_debug_info);
    }
}

fn show_debug_info(mut context: ResMut<EguiContext>, settings: Res<Settings>) {
    egui::Window::new("Debug")
    .collapsible(false)
    .title_bar(false)
    .resizable(false)
    .show(context.ctx_mut(), move |ui| {
        ui.label(format!("Render Distance: {}", settings.render_distance));
    });
}

fn update_render_distance(input: Res<Input<KeyCode>>, mut settings: ResMut<Settings>) {
    if input.just_pressed(KeyCode::W) {
        settings.render_distance += 1;
    }
    if input.just_pressed(KeyCode::S) {
        settings.render_distance -= 1;
    }

    if settings.render_distance < 0 {
        settings.render_distance = 0;
    }
}

#[derive(Clone, Copy)]
pub struct Settings {
    pub render_distance: isize,
    pub mesh_builder: MeshBuilderSettings,
    pub noise: NoiseSettings,
    prev_noise: NoiseSettings,
}

impl Settings {
    pub fn detect_changes(&mut self) -> bool {
        if self.noise != self.prev_noise {
            self.prev_noise = self.noise;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct NoiseSettings {
    pub octaves: i32,
    pub lacunarity: f32,
    pub gain: f32,
    pub frequency: f32,
    pub amplitude: f32,
}

impl Default for NoiseSettings {
    fn default() -> Self {
        Self {
            octaves: 5,
            lacunarity: 2.0,
            gain: 0.5,
            frequency: 2.0,
            amplitude: 100.0,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            render_distance: 16,
            mesh_builder: Default::default(),
            noise: Default::default(),
            prev_noise: Default::default(),
        }
    }
}
