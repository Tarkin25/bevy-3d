#![allow(dead_code)]

use std::time::Duration;

use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::{prelude::*, window::WindowMode};
use bevy_3d::array_texture::{ArrayTextureMaterial, ArrayTexturePlugin, ATTRIBUTE_TEXTURE_INDEX};
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
use bevy_rapier3d::prelude::*;

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
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        /* .add_plugin(RapierDebugRenderPlugin {
            mode: DebugRenderMode::COLLIDER_SHAPES,
            ..Default::default()
        }) */
        .insert_resource(AtmosphereModel::default())
        .insert_resource(CycleTimer(Timer::new(
            Duration::from_millis(100),
            TimerMode::Repeating,
        )))
        .add_plugin(AtmospherePlugin)
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
        .add_system(spawn_ball.in_set(OnUpdate(AppState::InGame)))
        .add_system(daylight_cycle.in_set(OnUpdate(AppState::InGame)))
        .run();
}

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
struct Sun;

// Timer for updating the daylight cycle (updating the atmosphere every frame is slow, so it's better to do incremental changes)
#[derive(Resource)]
struct CycleTimer(Timer);

// We can edit the Atmosphere resource and it will be updated automatically
fn daylight_cycle(
    mut atmosphere: AtmosphereMut<Nishita>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut timer: ResMut<CycleTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        let t = time.elapsed_seconds_wrapped() as f32 / 20.0;
        atmosphere.sun_position = Vec3::new(0., t.sin(), t.cos());

        if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
            light_trans.rotation = Quat::from_rotation_x(-t.sin().atan2(t.cos()));
            directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
        }
    }
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
        32.0,
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
            transform: Transform::from_xyz(4.0, 200.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
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
        let new_state = match state.0 {
            AppState::InGame => AppState::Menu,
            AppState::Menu => AppState::InGame,
        };

        next_state.set(new_state);
    }
}
