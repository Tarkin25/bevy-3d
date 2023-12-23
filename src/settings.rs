use bevy::prelude::*;
use bevy_inspector_egui::{
    prelude::ReflectInspectorOptions, quick::ResourceInspectorPlugin, InspectorOptions,
};

use crate::game::chunk::mesh_builder::MeshBuilderSettings;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::default())
            .add_plugins(ResourceInspectorPlugin::<Settings>::default())
            .add_systems(Update, update_render_distance);
    }
}

/* fn show_menu(mut inspector_windows: ResMut<InspectorWindows>) {
    let mut settings_inspector = inspector_windows.window_data_mut::<Settings>();
    settings_inspector.visible = true;
}

fn hide_menu(mut inspector_windows: ResMut<InspectorWindows>) {
    let mut settings_inspector = inspector_windows.window_data_mut::<Settings>();
    settings_inspector.visible = false;
} */

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

#[derive(Clone, Copy, Resource, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct Settings {
    #[inspector(min = 0, max = 32)]
    pub render_distance: isize,
    pub update_chunks: bool,
    pub task_polls_per_frame: usize,
    pub mesh_updates_per_frame: usize,
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

#[derive(Debug, Clone, Copy, PartialEq, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct NoiseSettings {
    pub scale: u32,
}

impl Default for NoiseSettings {
    fn default() -> Self {
        Self { scale: 100 }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            render_distance: 16,
            update_chunks: true,
            task_polls_per_frame: 1,
            mesh_updates_per_frame: 1,
            mesh_builder: Default::default(),
            noise: Default::default(),
            prev_noise: Default::default(),
        }
    }
}
