use std::{fmt::Debug, time::Duration};

use bevy::{
    prelude::*,
    window::{close_on_esc, WindowMode},
};
use camera_controller::{CameraControllerPlugin, CameraController};
use chunk_generator::ChunkGenerator;
use grid::{GridCoordinates, ChunkGrid};
use mesh_builder::MeshBuilder;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Perlin,
};
use tap::Pipe;

mod camera_controller;
mod chunk_generator;
mod mesh_builder;
pub mod grid;

#[macro_export]
macro_rules! vec3 {
    ($x: expr, $y: expr, $z: expr) => {
        Vec3::new($x as f32, $y as f32, $z as f32)
    };
}

const CHUNK_SIZE: usize = 25;
const CHUNK_LOWER_BOUND: usize = 0;
const CHUNK_UPPER_BOUND: usize = CHUNK_SIZE - 1;
const VOXEL_SIZE: f32 = 1.0;
const RENDER_DISTANCE: isize = 10;

struct VoxelConfig {
    material: Handle<StandardMaterial>,
}

#[derive(Default, Component)]
pub struct Chunk([[[bool; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Chunk").finish()
    }
}

impl Chunk {
    pub fn empty() -> Self {
        Self([[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE])
    }

    pub fn is_solid(&self, x: usize, y: usize, z: usize) -> bool {
        self.0[x][y][z]
    }

    pub fn is_air(&self, x: usize, y: usize, z: usize) -> bool {
        !self.is_solid(x, y, z)
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: bool) {
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

#[derive(Deref, DerefMut, Default, Debug)]
pub struct ChunkLoadQueue(Vec<GridCoordinates>);

#[derive(Deref, DerefMut, Default, Debug)]
pub struct ChunkUnloadQueue(Vec<(Entity, GridCoordinates)>);

fn custom_mesh_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut builder = MeshBuilder::default();
    builder.move_to(vec3!(0, 0, 0));
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

#[derive(Deref, DerefMut)]
struct GenerationTimer(Timer);

fn generate_chunks(
    mut chunk_load_queue: ResMut<ChunkLoadQueue>,
    player: Query<&Transform, With<CameraController>>,
    grid: Res<ChunkGrid>,
) {
    let translation = player.single().translation;
    let player_xz_coordinates = GridCoordinates::new(translation.x.round() as isize, 0, translation.z.round() as isize);
    let player_grid_coordinates = player_xz_coordinates.to_grid();

    for x in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for z in -RENDER_DISTANCE..=RENDER_DISTANCE {
            let chunk_coordinates = player_grid_coordinates + [x * CHUNK_SIZE as isize, 0, z * CHUNK_SIZE as isize];
            
            if !grid.contains_key(&chunk_coordinates) {
                chunk_load_queue.push(chunk_coordinates);
            }
        }
    }
}

#[derive(Deref, DerefMut)]
struct DespawnTimer(Timer);

fn despawn_chunks(
    mut chunk_unload_queue: ResMut<ChunkUnloadQueue>,
    chunks: Query<(Entity, &GridCoordinates)>,
    player: Query<&Transform, With<CameraController>>,
    grid: Res<ChunkGrid>,
) {
    let translation = player.single().translation;
    let player_xz_coordinates = GridCoordinates::new(translation.x.round() as isize, 0, translation.z.round() as isize);
    let player_grid_coordinates = player_xz_coordinates.to_grid();
    let bounds_distance = CHUNK_SIZE as isize * RENDER_DISTANCE;

    for (entity, coordinates) in &chunks {
        let is_outside_pos_x = player_grid_coordinates.x + bounds_distance < coordinates.x;
        let is_outside_neg_x = player_grid_coordinates.x - bounds_distance > coordinates.x;
        let is_outside_pos_z = player_grid_coordinates.z + bounds_distance < coordinates.z;
        let is_outside_neg_z = player_grid_coordinates.z - bounds_distance > coordinates.z;

        let is_outside_render_distance = is_outside_pos_x || is_outside_neg_x || is_outside_pos_z || is_outside_neg_z;
        
        if is_outside_render_distance && grid.contains_key(coordinates) {
            chunk_unload_queue.push((entity, *coordinates));
        }
    }
}

fn load_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_load_queue: ResMut<ChunkLoadQueue>,
    config: Res<VoxelConfig>,
    generator: Res<ChunkGenerator>,
    mut grid: ResMut<ChunkGrid>,
) {
    while let Some(coordinates) = chunk_load_queue.pop() {
        let chunk = generator.generate(coordinates.into()); // TODO - separate chunk generation from mesh computation
        let mesh = chunk.compute_mesh();
        let transform = Transform::from_translation(coordinates.into());

        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(mesh),
                material: config.material.clone(),
                transform,
                ..Default::default()
            })
            .insert(coordinates);

        grid.insert(coordinates, chunk);
    }
}

fn unload_chunks(
    mut commands: Commands,
    mut chunk_unload_queue: ResMut<ChunkUnloadQueue>,
    mut grid: ResMut<ChunkGrid>,
) {
    while let Some((entity, chunk_id)) = chunk_unload_queue.pop() {
        commands.entity(entity).despawn();
        grid.remove(&chunk_id);
    }
}

fn setup_config(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let material = materials.add(Color::GREEN.into());
    let perlin = Perlin::new();
    let _noise_map = PlaneMapBuilder::new(&perlin)
        .set_size(1024, 1024)
        .set_x_bounds(0.0, 1.0)
        .set_y_bounds(0.0, 1.0)
        .build();

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
        .insert_resource(ChunkLoadQueue::default())
        .insert_resource(ChunkUnloadQueue::default())
        .insert_resource(ChunkGenerator)
        .insert_resource(ChunkGrid::default())
        .insert_resource(GenerationTimer(Timer::new(Duration::from_secs(1), true)))
        .insert_resource(DespawnTimer(Timer::new(Duration::from_secs(1), true)))
        .add_startup_system(setup_config)
        .add_startup_system(setup_light)
        .add_startup_system(custom_mesh_setup)
        .add_system(generate_chunks)
        .add_system(despawn_chunks)
        .add_system(load_chunks)
        .add_system(unload_chunks)
        .add_system(close_on_esc)
        .run();
}
