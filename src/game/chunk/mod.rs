use std::sync::{Arc, RwLock};

use bevy::{prelude::*, tasks::{AsyncComputeTaskPool, Task}};

use futures_lite::future;

use crate::{settings::Settings, VoxelConfig, vec3};

use self::{generator::ChunkGenerator, grid::{ChunkGrid, GridCoordinates}, mesh_builder::{MeshBuilderSettings, MeshBuilder}};

use super::camera_controller::CameraController;

pub mod generator;
pub mod grid;
pub mod mesh_builder;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(Arc::new(RwLock::new(ChunkGenerator::default())))
        .insert_resource(ChunkGrid::default())
        .add_system(generate_chunks)
        .add_system(compute_meshes)
        .add_system(spawn_chunks)
        .add_system(unload_chunks)
        .add_system(despawn_chunks);
    }
}

#[derive(Component)]
pub struct Chunk(Vec<Vec<Vec<bool>>>);

impl Chunk {
    pub const WIDTH: usize = 16;
    pub const HEIGHT: usize = 256;
    pub const LOWER_BOUND: usize = 0;
    pub const UPPER_BOUND: usize = Chunk::WIDTH - 1;
    
    pub fn empty() -> Self {
        Self(vec![
            vec![vec![false; Chunk::WIDTH]; Chunk::HEIGHT];
            Chunk::WIDTH
        ])
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
    pub fn compute_mesh(&self, settings: MeshBuilderSettings) -> Mesh {
        let mut builder = MeshBuilder::new(settings);

        for x in 0..Chunk::WIDTH {
            for y in 0..Chunk::HEIGHT {
                for z in 0..Chunk::WIDTH {
                    builder.move_to(vec3!(x, y, z));

                    if self.is_solid(x, y, z) {
                        if y == Chunk::UPPER_BOUND || self.is_air(x, y + 1, z) {
                            builder.face_top();
                        }
                        if y == Chunk::LOWER_BOUND || self.is_air(x, y - 1, z) {
                            builder.face_bottom();
                        }
                        if x == Chunk::LOWER_BOUND || self.is_air(x - 1, y, z) {
                            builder.face_right();
                        }
                        if x == Chunk::UPPER_BOUND || self.is_air(x + 1, y, z) {
                            builder.face_left();
                        }
                        if z == Chunk::LOWER_BOUND || self.is_air(x, y, z - 1) {
                            builder.face_front();
                        }
                        if z == Chunk::UPPER_BOUND || self.is_air(x, y, z + 1) {
                            builder.face_back();
                        }
                    }
                }
            }
        }

        //std::thread::sleep(std::time::Duration::from_millis(10));

        builder.build()
    }
}

#[derive(Component)]
struct GenerateChunk(Task<Chunk>);

#[derive(Component)]
struct ComputeMesh(Task<(Chunk, Mesh)>);

fn generate_chunks(
    mut commands: Commands,
    player: Query<&Transform, With<CameraController>>,
    mut grid: ResMut<ChunkGrid>,
    generator: Res<Arc<RwLock<ChunkGenerator>>>,
    settings: Res<Settings>,
) {
    let translation = player.single().translation;
    let player_xz_coordinates = GridCoordinates::new(
        translation.x.round() as isize,
        0,
        translation.z.round() as isize,
    );
    let player_grid_coordinates = player_xz_coordinates.to_grid();
    let task_pool = AsyncComputeTaskPool::get();

    for x in -settings.render_distance..=settings.render_distance {
        for z in -settings.render_distance..=settings.render_distance {
            let chunk_coordinates =
                player_grid_coordinates + [x * Chunk::WIDTH as isize, 0, z * Chunk::WIDTH as isize];

            if !grid.contains_key(&chunk_coordinates) {
                let generator = Arc::clone(&generator);

                let task =
                    task_pool.spawn(async move { generator.read().unwrap().generate(chunk_coordinates.into()) });

                commands
                    .spawn()
                    .insert(GenerateChunk(task))
                    .insert(chunk_coordinates);

                grid.insert(chunk_coordinates, None);
            }
        }
    }
}

fn compute_meshes(
    mut commands: Commands,
    mut generation_tasks: Query<(Entity, &mut GenerateChunk)>,
    settings: Res<Settings>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    for (entity, mut generation_task) in &mut generation_tasks {
        if let Some(chunk) = future::block_on(future::poll_once(&mut generation_task.0)) {
            let mesh_builder_settings = settings.mesh_builder.clone();
            let task = task_pool.spawn(async move {
                let mesh = chunk.compute_mesh(mesh_builder_settings);

                (chunk, mesh)
            });

            let mut entity = commands.entity(entity);
            entity.insert(ComputeMesh(task));
            entity.remove::<GenerateChunk>();
        }
    }
}

fn spawn_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<VoxelConfig>,
    mut grid: ResMut<ChunkGrid>,
    mut mesh_computation_tasks: Query<(Entity, &mut ComputeMesh, &GridCoordinates)>,
) {
    for (entity, mut mesh_computation_task, coordinates) in mesh_computation_tasks.iter_mut().take(20) {
        if let Some((chunk, mesh)) =
            future::block_on(future::poll_once(&mut mesh_computation_task.0))
        {
            let mut entity = commands.entity(entity);
            entity.insert_bundle(PbrBundle {
                mesh: meshes.add(mesh),
                material: config.material.clone(),
                transform: Transform::from_translation((*coordinates).into()),
                ..Default::default()
            });
            entity.remove::<ComputeMesh>();

            grid.insert(*coordinates, Some(chunk));
        }
    }
}

fn unload_chunks(
    mut commands: Commands,
    mut chunks: Query<(Entity, &GridCoordinates)>,
    player: Query<&Transform, With<CameraController>>,
    mut grid: ResMut<ChunkGrid>,
    settings: Res<Settings>,
) {
    let translation = player.single().translation;
    let player_xz_coordinates = GridCoordinates::new(
        translation.x.round() as isize,
        0,
        translation.z.round() as isize,
    );
    let player_grid_coordinates = player_xz_coordinates.to_grid();
    let bounds_distance = Chunk::WIDTH as isize * settings.render_distance;

    for (entity, coordinates) in &mut chunks {
        let is_outside_pos_x = player_grid_coordinates.x + bounds_distance < coordinates.x;
        let is_outside_neg_x = player_grid_coordinates.x - bounds_distance > coordinates.x;
        let is_outside_pos_z = player_grid_coordinates.z + bounds_distance < coordinates.z;
        let is_outside_neg_z = player_grid_coordinates.z - bounds_distance > coordinates.z;

        let is_outside_render_distance =
            is_outside_pos_x || is_outside_neg_x || is_outside_pos_z || is_outside_neg_z;

        if is_outside_render_distance && grid.contains_key(coordinates) {
            grid.remove(coordinates);
            commands.entity(entity).insert(DespawnChunk);
        }
    }
}

#[derive(Component)]
pub struct DespawnChunk;

fn despawn_chunks(mut commands: Commands, chunks: Query<Entity, (With<DespawnChunk>, Without<GenerateChunk>, Without<ComputeMesh>)>) {
    for entity in chunks.iter().take(20) {
        commands.entity(entity).despawn();
    }
}