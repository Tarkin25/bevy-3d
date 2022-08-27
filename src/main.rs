use bevy::{
    prelude::*,
    window::WindowMode,
};
use bevy_egui::EguiPlugin;
use game::{chunk::{mesh_builder::MeshBuilder, ChunkPlugin}, camera_controller::CameraControllerPlugin, debug_info::DebugInfoPlugin};
use menu::MenuPlugin;
use settings::{Settings, SettingsPlugin};
use tap::Pipe;
pub mod settings;
mod menu;
mod button_test;
mod game;

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
            transform: Transform::from_xyz(0.5, 100.0, -1.0),
        })
        .add_plugin(ChunkPlugin)
        .add_plugin(SettingsPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(DebugInfoPlugin)
        .add_state(AppState::InGame)
        .add_startup_system(setup_config)
        .add_startup_system(setup_light)
        .add_startup_system(custom_mesh_setup)
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

fn custom_mesh_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<Settings>,
) {
    let mut builder = MeshBuilder::new(settings.mesh_builder);
    builder.move_to(vec3!(0, 100, 0));
    builder.face_front();
    builder.face_top();
    builder.face_bottom();
    builder.face_back();
    builder.face_right();
    builder.face_left();

    let mesh = builder.build().pipe(|mesh| meshes.add(mesh));
    let material = materials.add(Color::RED.into());
    let transform = Transform::from_xyz(0.0, 0.0, 0.0);

    commands.spawn_bundle(PbrBundle {
        mesh,
        material,
        transform,
        ..Default::default()
    });
}

fn setup_config(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let material = materials.add(Color::GREEN.into());

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
