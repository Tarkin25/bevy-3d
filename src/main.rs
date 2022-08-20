use bevy::{
    prelude::*,
    window::{close_on_esc, WindowMode},
};
use camera_controller::CameraControllerPlugin;
use mesh_builder::MeshBuilder;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Perlin,
};
use tap::Pipe;


mod camera_controller;
mod cubemap_material;
mod mesh_builder;

#[macro_export]
macro_rules! vec3 {
    ($x: expr, $y: expr, $z: expr) => {
        Vec3::new($x as f32, $y as f32, $z as f32)
    };
}

const CHUNK_SIZE: usize = 25;
const CHUNK_LOWER_BOUND: usize = 0;
const CHUNK_UPPER_BOUND: usize = CHUNK_SIZE - 1;
const MAX_HEIGHT: f64 = 10.0;
const VOXEL_SIZE: f32 = 1.0;

struct VoxelConfig {
    material: Handle<StandardMaterial>,
}

struct Chunk([[[bool; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);

impl Chunk {
    fn empty() -> Self {
        Self([[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE])
    }

    fn is_solid(&self, x: usize, y: usize, z: usize) -> bool {
        self.0[x][y][z]
    }

    fn is_air(&self, x: usize, y: usize, z: usize) -> bool {
        !self.is_solid(x, y, z)
    }

    fn set(&mut self, x: usize, y: usize, z: usize, value: bool) {
        self.0[x][y][z] = value;
    }

    /// http://ilkinulas.github.io/development/unity/2016/04/30/cube-mesh-in-unity3d.html
    fn compute_mesh(&self) -> Mesh {
        let mut builder = MeshBuilder::default();
        
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    builder.move_to(vec3!(x, y, z));

                    if self.is_solid(x, y, z) {
                        if y == CHUNK_UPPER_BOUND || self.is_air(x, y + 1, z) {
                            builder.face_top();
                        }
                        if y == CHUNK_LOWER_BOUND || self.is_air(x, y - 1, z) {
                            builder.face_bottom();
                        }
                        if x == CHUNK_LOWER_BOUND || self.is_air(x - 1, y, z) {
                            builder.face_right();
                        }
                        if x == CHUNK_UPPER_BOUND || self.is_air(x + 1, y, z) {
                            builder.face_left();
                        }
                        if z == CHUNK_LOWER_BOUND || self.is_air(x, y, z - 1) {
                            builder.face_front();
                        }
                        if z == CHUNK_UPPER_BOUND || self.is_air(x, y, z + 1) {
                            builder.face_back();
                        }
                    }
                }
            }
        }

        builder.build()
    }
}

fn generate_chunk() -> Chunk {
    let mut chunk = Chunk::empty();
    let perlin = Perlin::new();
    let map = PlaneMapBuilder::new(&perlin)
        .set_size(CHUNK_SIZE, CHUNK_SIZE)
        .set_x_bounds(0.0, 1.0)
        .set_y_bounds(0.0, 1.0)
        .build();

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let y = map
                .get_value(x, z)
                .pipe(|y| y * MAX_HEIGHT)
                .pipe(f64::round)
                .pipe(|y| y as usize);

            for y in 0..y {
                chunk.set(x, y, z, true);
            }
        }
    }

    chunk
}

fn custom_mesh_setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mut builder = MeshBuilder::default();
    builder.move_to(vec3!(0, 0, 0));
    builder.face_front();
    builder.face_top();
    builder.face_bottom();
    builder.face_back();
    builder.face_right();
    builder.face_left();

    let mesh = builder
    .build()
    .pipe(|mesh| meshes.add(mesh));
    let material = materials.add(Color::RED.into());
    let transform = Transform::from_xyz(0.0, 0.0, 0.0);

    commands.spawn_bundle(PbrBundle {
        mesh,
        material,
        transform,
        ..Default::default()
    });
}

fn setup_chunk(
    mut has_run: Local<bool>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<VoxelConfig>,
) {
    if !*has_run {
        let chunk = generate_chunk();
        let mesh = chunk.compute_mesh();

        commands.spawn_bundle(PbrBundle {
            material: config.material.clone(),
            mesh: meshes.add(mesh),
            ..Default::default()
        });

        *has_run = true;
    }
}

fn setup_config(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
        .add_plugin(CameraControllerPlugin {
            transform: Transform::from_xyz(0.5, 1.0, -1.0)
                .looking_at(Vec3::new(0.5, 0.5, 0.5), Vec3::Y),
        })
        .add_startup_system(setup_config)
        .add_startup_system(setup_light)
        .add_startup_system(custom_mesh_setup)
        .add_system(setup_chunk)
        .add_system(close_on_esc)
        .run();
}
