use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::{prelude::*, window::WindowMode};
use bevy_3d::array_texture::{ArrayTextureMaterial, ArrayTexturePlugin, ATTRIBUTE_TEXTURE_INDEX};
use bevy_3d::daylight_cycle::{DaylightCyclePlugin, Sun};
use bevy_3d::game::camera_controller::CameraController;
use bevy_3d::game::chunk::ChunkPlugin;
use bevy_3d::game::{camera_controller::CameraControllerPlugin, debug_info::DebugInfoPlugin};
use bevy_3d::menu::MenuPlugin;
use bevy_3d::my_material::MyMaterialPlugin;
use bevy_3d::settings::SettingsPlugin;
use bevy_3d::wireframe_controller::WireframeControllerPlugin;
use bevy_3d::{AppState, VoxelConfig};
use bevy_atmosphere::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_state::<AppState>()
        .init_schedule(OnEnter(AppState::Menu))
        .init_schedule(OnExit(AppState::Menu))
        .init_schedule(OnEnter(AppState::InGame))
        .init_schedule(OnExit(AppState::InGame))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxels".into(),
                mode: WindowMode::BorderlessFullscreen,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(AtmosphereModel::default())
        .add_plugins((
            //EguiPlugin,
            DefaultInspectorConfigPlugin,
            AtmospherePlugin,
            CameraControllerPlugin {
                transform: Transform::from_xyz(0.5, 100.0, -1.0)
                    .looking_at(Vec3::new(0.0, 99.0, 0.0), Vec3::Y),
            },
            ArrayTexturePlugin,
            ChunkPlugin,
            SettingsPlugin,
            MenuPlugin,
            DebugInfoPlugin,
            MyMaterialPlugin,
            WireframeControllerPlugin,
            DaylightCyclePlugin,
        ))
        .add_systems(PreStartup, setup_config)
        .add_systems(Startup, (setup_light, textured_cube))
        .add_systems(Update, toggle_app_state)
        .add_systems(Update, spawn_ball.run_if(in_state(AppState::InGame)))
        .run();
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player: Query<&Transform, With<CameraController>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::B) {
        let translation = player.single().translation;

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere {
                    radius: 0.5,
                    ..Default::default()
                })),
                material: materials.add(StandardMaterial::from(Color::BLUE)),
                transform: Transform::from_translation(translation),
                ..Default::default()
            },
            RigidBody::Dynamic,
            Collider::ball(0.5),
            Restitution::coefficient(0.7),
        ));
    }
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
        32,
    ));

    commands.insert_resource(VoxelConfig { material });
}

fn setup_light(mut commands: Commands) {
    // Light
    commands.spawn((
        /* PointLightBundle {
            point_light: PointLight {
                shadows_enabled: true,
                range: 1000.0,
                intensity: 100_000.0,
                ..Default::default()
            },
            transform: Transform::from_xyz(4.0, 200.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        }, */
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..Default::default()
            },
            cascade_shadow_config: CascadeShadowConfigBuilder {
                first_cascade_far_bound: 7.0,
                maximum_distance: 1000.0,
                ..Default::default()
            }
            .build(),
            ..Default::default()
        },
        Sun,
    ));
}

fn toggle_app_state(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        let new_state = match state.get() {
            AppState::InGame => AppState::Menu,
            AppState::Menu => AppState::InGame,
        };

        next_state.set(new_state);
    }
}
