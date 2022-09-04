use bevy::prelude::*;
use bevy_inspector_egui::{InspectorPlugin, Inspectable, plugin::InspectorWindows};

use crate::{game::chunk::mesh_builder::MeshBuilderSettings, AppState};

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::default())
            .add_plugin(InspectorPlugin::<Settings>::new())
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(hide_menu))
            .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(show_menu))
            .add_system(update_render_distance);
    }
}

fn show_menu(mut inspector_windows: ResMut<InspectorWindows>) {
    let mut settings_inspector = inspector_windows.window_data_mut::<Settings>();
    settings_inspector.visible = true;
}

fn hide_menu(mut inspector_windows: ResMut<InspectorWindows>) {
    let mut settings_inspector = inspector_windows.window_data_mut::<Settings>();
    settings_inspector.visible = false;
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

#[derive(Clone, Copy, Inspectable)]
pub struct Settings {
    #[inspectable(min = 0, max = 32)]
    pub render_distance: isize,
    pub update_chunks: bool,
    pub mesh_builder: MeshBuilderSettings,
    pub noise: NoiseSettings,
    #[inspectable(ignore)]
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

#[derive(Clone, Copy, PartialEq, Inspectable)]
pub struct NoiseSettings {
    pub octaves: i32,
    pub lacunarity: f32,
    pub gain: f32,
    pub frequency: f32,
    pub amplitude: f32,
    pub scale: f32,
}

impl Default for NoiseSettings {
    fn default() -> Self {
        Self {
            octaves: 3,
            lacunarity: 2.0,
            gain: 0.5,
            frequency: 2.0,
            amplitude: 100.0,
            scale: 0.005,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            render_distance: 16,
            update_chunks: true,
            mesh_builder: Default::default(),
            noise: Default::default(),
            prev_noise: Default::default(),
        }
    }
}
