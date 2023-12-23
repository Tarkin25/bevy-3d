use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::AppState;

pub struct MyMaterialPlugin;

impl Plugin for MyMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<MyMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (move_movables, update_material_time, update_transparency)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MyMaterial>>,
) {
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            transform: Transform::from_xyz(0.0, 99.0, 0.0),
            material: materials.add(MyMaterial {
                color: Color::BLUE,
                time: 0.0,
                opacity: 1.0,
            }),
            ..Default::default()
        },
        Movable,
    ));
}

#[derive(AsBindGroup, TypeUuid, Clone, Asset, TypePath)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct MyMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(0)]
    time: f32,
    #[uniform(0)]
    opacity: f32,
}

impl Material for MyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/my_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[derive(Component)]
struct Movable;

fn move_movables(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Movable>>,
    time: Res<Time>,
) {
    let mut direction = Vec3::ZERO;

    if input.pressed(KeyCode::Up) {
        direction.z += 1.0;
    }
    if input.pressed(KeyCode::Down) {
        direction.z -= 1.0;
    }
    if input.pressed(KeyCode::Left) {
        direction.x += 1.0;
    }
    if input.pressed(KeyCode::Right) {
        direction.x -= 1.0;
    }

    let direction = direction * time.delta_seconds() * 10.0;

    query.for_each_mut(|mut transform| {
        transform.translation += direction;
    });
}

fn update_material_time(mut materials: ResMut<Assets<MyMaterial>>, time: Res<Time>) {
    for (_, mut material) in materials.iter_mut() {
        material.time = time.elapsed_seconds();
    }
}

fn update_transparency(
    mut materials: ResMut<Assets<MyMaterial>>,
    input: Res<Input<KeyCode>>,
    mut transparent: Local<bool>,
) {
    if input.just_pressed(KeyCode::T) {
        *transparent = !*transparent;

        let opacity = if *transparent { 0.8 } else { 1.0 };

        for (_, mut material) in materials.iter_mut() {
            material.opacity = opacity;
        }
    }
}
