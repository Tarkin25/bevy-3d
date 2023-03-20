#![allow(dead_code)]

use bevy::asset::AssetServerSettings;
use bevy::{prelude::*, window::WindowMode};
use bevy_3d::game::chunk::ChunkPlugin;
use bevy_3d::game::{camera_controller::CameraControllerPlugin, debug_info::DebugInfoPlugin};
use bevy_3d::menu::MenuPlugin;
use bevy_3d::my_material::MyMaterialPlugin;
use bevy_3d::settings::SettingsPlugin;
use bevy_3d::texture_atlas::TextureAtlasMaterial;
use bevy_3d::texture_atlas::TextureAtlasPlugin;
use bevy_3d::wireframe_controller::WireframeControllerPlugin;
use bevy_3d::{AppState, VoxelConfig};
use bevy_egui::EguiPlugin;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Voxels".into(),
            cursor_locked: true,
            cursor_visible: false,
            mode: WindowMode::Fullscreen,
            ..Default::default()
        })
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(CameraControllerPlugin {
            transform: Transform::from_xyz(0.5, 100.0, -1.0)
                .looking_at(Vec3::new(0.0, 99.0, 0.0), Vec3::Y),
        })
        .add_plugin(TextureAtlasPlugin)
        .add_plugin(ChunkPlugin)
        .add_plugin(SettingsPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(DebugInfoPlugin)
        .add_plugin(MyMaterialPlugin)
        .add_plugin(WireframeControllerPlugin)
        .add_state(AppState::InGame)
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_config)
        .add_startup_system(setup_light)
        .add_startup_system(textured_cube)
        .add_system(toggle_app_state)
        .run();
}

fn textured_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<VoxelConfig>,
) {
    let mesh = Mesh::from(shape::Cube { size: 1.0 });

    commands.spawn_bundle(MaterialMeshBundle {
        mesh: meshes.add(mesh),
        material: config.material.clone(),
        transform: Transform::from_xyz(2.0, 99.0, 0.0),
        ..Default::default()
    });
}

fn setup_config(
    mut commands: Commands,
    mut materials: ResMut<Assets<TextureAtlasMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let material = materials.add(TextureAtlasMaterial {
        size: Vec2::new(66.0, 66.0),
        resolution: 32.0,
        gap: 1.0,
        texture_handle: asset_server.load("textures/texture-atlas.png"),
    });

    commands.insert_resource(VoxelConfig { material });
}

fn setup_light(mut commands: Commands) {
    // Light
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(400.0, 800.0, 400.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    /* commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            range: 10_000.0,
            intensity: 100_000_000.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(400.0, 800.0, 400.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    }); */
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
