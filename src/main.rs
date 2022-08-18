use std::f32::consts::TAU;

use bevy::{prelude::*, window::close_on_esc};
use camera_controller::CameraControllerPlugin;
use skybox::SkyboxPlugin;

mod camera_controller;
mod cubemap_material;
mod skybox;

#[derive(Component)]
struct Rotate {
    speed: f32,
    axis: Vec3,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });

    // Cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert(Rotate {
            speed: 0.1,
            axis: Vec3::Y,
        });

    // Light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
}

fn rotate(mut rotates: Query<(&mut Transform, &Rotate)>, time: Res<Time>) {
    for (mut transform, rotate) in &mut rotates {
        transform.rotate_axis(rotate.axis, rotate.speed * TAU * time.delta_seconds());
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Voxels".into(),
            cursor_locked: true,
            cursor_visible: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraControllerPlugin {
            transform: Transform::from_xyz(0.0, 1.0, -4.0)
                .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        })
        .add_plugin(SkyboxPlugin::new("textures/Ryfjallet_cubemap.png"))
        .add_startup_system(setup)
        .add_system(rotate)
        .add_system(close_on_esc)
        .run();
}
