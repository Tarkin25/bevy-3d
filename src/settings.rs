use bevy::prelude::*;

use crate::mesh_builder::MeshBuilderSettings;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::default());
    }
}

#[derive(Clone, Copy)]
pub struct Settings {
    pub render_distance: isize,
    pub mesh_builder: MeshBuilderSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            render_distance: 16,
            mesh_builder: Default::default()
        }
    }
}