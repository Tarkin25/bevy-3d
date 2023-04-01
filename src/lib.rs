pub mod array_texture;
pub mod color_grading;
pub mod daylight_cycle;
pub mod game;
pub mod menu;
pub mod my_material;
pub mod settings;
pub mod utils;
pub mod wireframe_controller;

use array_texture::ArrayTextureMaterial;
use bevy::prelude::*;

#[macro_export]
macro_rules! vec3 {
    ($x: expr, $y: expr, $z: expr) => {
        Vec3::new($x as f32, $y as f32, $z as f32)
    };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AppState {
    Menu,
    #[default]
    InGame,
}

#[derive(Resource)]
pub struct VoxelConfig {
    pub material: Handle<ArrayTextureMaterial>,
}
