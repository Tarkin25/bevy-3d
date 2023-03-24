#![allow(dead_code)]

use bevy::{prelude::*, window::WindowMode};
use bevy_3d::array_texture::{ArrayTextureMaterial, ArrayTexturePlugin, ATTRIBUTE_TEXTURE_INDEX};
use bevy_3d::game::chunk::ChunkPlugin;
use bevy_3d::game::{camera_controller::CameraControllerPlugin, debug_info::DebugInfoPlugin};
use bevy_3d::menu::MenuPlugin;
use bevy_3d::my_material::MyMaterialPlugin;
use bevy_3d::settings::SettingsPlugin;
use bevy_3d::wireframe_controller::WireframeControllerPlugin;
use bevy_3d::{AppState, VoxelConfig};
use bevy_egui::EguiPlugin;

fn main() {
    App::new()
        .add_state::<AppState>()
        .init_schedule(OnEnter(AppState::Menu))
        .init_schedule(OnExit(AppState::Menu))
        .init_schedule(OnEnter(AppState::InGame))
        .init_schedule(OnExit(AppState::InGame))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Voxels".into(),
                        mode: WindowMode::BorderlessFullscreen,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..Default::default()
                }),
        )
        .add_plugin(EguiPlugin)
        .add_plugin(CameraControllerPlugin {
            transform: Transform::from_xyz(0.5, 100.0, -1.0)
                .looking_at(Vec3::new(0.0, 99.0, 0.0), Vec3::Y),
        })
        .add_plugin(ArrayTexturePlugin)
        .add_plugin(ChunkPlugin)
        .add_plugin(SettingsPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(DebugInfoPlugin)
        .add_plugin(MyMaterialPlugin)
        .add_plugin(WireframeControllerPlugin)
        .add_startup_system(setup_config.in_base_set(StartupSet::PreStartup))
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
    let mut mesh = Mesh::from(shape::Cube { size: 1.0 });
    let texture_indices = (0..mesh.count_vertices())
        .map(|_| 0_u32)
        .collect::<Vec<_>>();
    mesh.insert_attribute(ATTRIBUTE_TEXTURE_INDEX, texture_indices);

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(mesh),
        material: config.material.clone(),
        transform: Transform::from_xyz(2.0, 99.0, 0.0),
        ..Default::default()
    });
}

fn setup_config(
    mut commands: Commands,
    mut materials: ResMut<Assets<ArrayTextureMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let material = materials.add(ArrayTextureMaterial::with_resolution(
        asset_server.load("textures/texture-atlas.png"),
        32.0,
    ));

    commands.insert_resource(VoxelConfig { material });
}

fn setup_light(mut commands: Commands) {
    // Light
    commands.spawn(DirectionalLightBundle {
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

fn toggle_app_state(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        let new_state = match state.0 {
            AppState::InGame => AppState::Menu,
            AppState::Menu => AppState::InGame,
        };

        next_state.set(new_state);
    }
}
