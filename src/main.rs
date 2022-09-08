#![allow(dead_code)]

use bevy::{prelude::*, window::WindowMode};
use bevy_egui::EguiPlugin;
use game::{
    camera_controller::CameraControllerPlugin,
    debug_info::DebugInfoPlugin,
};
use menu::MenuPlugin;
use my_material::MyMaterialPlugin;
use settings::SettingsPlugin;
use spatial_hash_grid::SpatialHashGridPlugin;

mod button_test;
mod game;
mod menu;
mod my_material;
pub mod settings;
pub mod utils;
mod spatial_hash_grid;

#[macro_export]
macro_rules! vec3 {
    ($x: expr, $y: expr, $z: expr) => {
        Vec3::new($x as f32, $y as f32, $z as f32)
    };
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Voxels".into(),
            cursor_locked: true,
            cursor_visible: false,
            mode: WindowMode::Fullscreen,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(CameraControllerPlugin {
            transform: Transform::from_xyz(0.5, 100.0, -1.0)
                .looking_at(Vec3::new(0.0, 99.0, 0.0), Vec3::Y),
        })
        //.add_plugin(ChunkPlugin)
        .add_plugin(SettingsPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(DebugInfoPlugin)
        .add_plugin(MyMaterialPlugin)
        .add_state(AppState::InGame)
        .add_plugin(SpatialHashGridPlugin)
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_config)
        .add_startup_system(setup_light)
        .add_startup_system(textured_cube)
        .add_system(toggle_app_state)
        .run();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    Menu,
    InGame,
}

pub struct VoxelConfig {
    material: Handle<StandardMaterial>,
}

fn textured_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<VoxelConfig>,
) {
    let mesh = Mesh::from(shape::Cube { size: 1.0 });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: config.material.clone(),
        transform: Transform::from_xyz(2.0, 99.0, 0.0),
        ..Default::default()
    });
}

fn setup_config(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/minecraft-texture-atlas.png")),
        alpha_mode: AlphaMode::Opaque,
        ..Default::default()
    });

    commands.insert_resource(VoxelConfig { material });
}

fn setup_light(mut commands: Commands) {
    // Light
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            color: Color::rgb(0.5, 0.5, 0.5),
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn toggle_app_state(mut state: ResMut<State<AppState>>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        let new_state = match *state.current() {
            AppState::InGame => AppState::Menu,
            AppState::Menu => AppState::InGame,
        };

        state.set(new_state).unwrap();
    }
}
