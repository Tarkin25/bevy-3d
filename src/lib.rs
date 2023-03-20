pub mod game;
pub mod menu;
pub mod my_material;
pub mod settings;
pub mod utils;
pub mod wireframe_controller;

use bevy::prelude::*;

#[macro_export]
macro_rules! vec3 {
    ($x: expr, $y: expr, $z: expr) => {
        Vec3::new($x as f32, $y as f32, $z as f32)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    Menu,
    InGame,
}

pub struct VoxelConfig {
    pub material: Handle<StandardMaterial>,
}
