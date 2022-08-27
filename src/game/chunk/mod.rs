use std::sync::{Arc, RwLock};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};

use futures_lite::future;

use crate::{settings::Settings, utils::ToUsize, vec3, VoxelConfig};

use self::{
    generator::{ChunkGenerator, ContinentalGenerator},
    grid::{ChunkGrid, GridCoordinates},
    mesh_builder::{MeshBuilder, MeshBuilderSettings},
};

use super::camera_controller::CameraController;

pub mod generator;
pub mod grid;
pub mod mesh_builder;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Arc::new(ChunkGrid::default()))
            .add_startup_system(insert_generator)
            .add_system(generate_chunks)
            .add_system(compute_meshes)
            .add_system(spawn_chunks)
            .add_system(unload_chunks)
            .add_system(despawn_chunks);
    }
}

fn insert_generator(mut commands: Commands, settings: Res<Settings>) {
    let mut generator = ContinentalGenerator::new(
        50,
        [
            (-1.0, 50.0),
            (-0.7, 80.0),
            (-0.5, 75.0),
            (-0.4, 20.0),
            (0.0, 0.0),
            (1.0, 50.0),
        ],
    );
    generator.apply_noise_settings(settings.noise);
    let generator: Arc<RwLock<dyn ChunkGenerator>> = Arc::new(RwLock::new(generator));

    commands.insert_resource(generator);
}

#[derive(Component)]
pub struct Chunk {
    data: Vec<Vec<Vec<bool>>>,
    coordinates: GridCoordinates,
}

impl Chunk {
    pub const WIDTH: isize = 16;
    pub const HEIGHT: isize = 256;
    pub const LOWER_BOUND: isize = 0;
    pub const UPPER_BOUND: isize = Chunk::WIDTH - 1;

    pub fn new(coordinates: GridCoordinates) -> Self {
        Self {
            data: vec![
                vec![vec![false; Chunk::WIDTH as usize]; Chunk::HEIGHT as usize];
                Chunk::WIDTH as usize
            ],
            coordinates,
        }
    }

    pub fn is_solid(&self, position: [isize; 3]) -> bool {
        let [x, y, z] = position.map(|n| n.to_usize());
        self.data[x][y][z]
    }

    pub fn is_air(&self, [x, y, z]: [isize; 3]) -> bool {
        !self.is_solid([x, y, z])
    }

    fn check_adjacent_chunk(
        &self,
        grid: &Arc<ChunkGrid>,
        grid_offset: [isize; 3],
        chunk_coordinates: [isize; 3],
    ) -> bool {
        grid.get(&(self.coordinates + grid_offset))
            .and_then(|r| {
                r.value()
                    .as_ref()
                    .map(|chunk| chunk.is_solid(chunk_coordinates))
            })
            .unwrap_or(true)
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: bool) {
        self.data[x][y][z] = value;
    }

    /// http://ilkinulas.github.io/development/unity/2016/04/30/cube-mesh-in-unity3d.html
    pub fn compute_mesh(&self, settings: MeshBuilderSettings, grid: Arc<ChunkGrid>) -> Mesh {
        let mut builder = MeshBuilder::new(settings);

        for x in 0..Chunk::WIDTH {
            for y in 0..Chunk::HEIGHT {
                for z in 0..Chunk::WIDTH {
                    builder.move_to(vec3!(x, y, z));

                    if self.is_solid([x, y, z]) {
                        if y == Chunk::HEIGHT-1 || self.is_air([x, y + 1, z]) {
                            builder.face_top();
                        }
                        if y == Chunk::LOWER_BOUND || self.is_air([x, y - 1, z]) {
                            builder.face_bottom();
                        }
                        if self.adjacent_is_air([x - 1, y, z], &grid) {
                            builder.face_right();
                        }
                        if self.adjacent_is_air([x + 1, y, z], &grid) {
                            builder.face_left();
                        }
                        if self.adjacent_is_air([x, y, z - 1], &grid) {
                            builder.face_front();
                        }
                        if self.adjacent_is_air([x, y, z + 1], &grid) {
                            builder.face_back();
                        }
                    }
                }
            }
        }

        //std::thread::sleep(std::time::Duration::from_millis(10));

        builder.build()
    }

    fn adjacent_is_solid(&self, [x, y, z]: [isize; 3], grid: &Arc<ChunkGrid>) -> bool {
        if x < Chunk::LOWER_BOUND {
            return self.check_adjacent_chunk(
                grid,
                [-Chunk::WIDTH, 0, 0],
                [Chunk::UPPER_BOUND, y, z],
            );
        }
        if x > Chunk::UPPER_BOUND {
            return self.check_adjacent_chunk(
                grid,
                [Chunk::WIDTH, 0, 0],
                [Chunk::LOWER_BOUND, y, z],
            );
        }
        if z < Chunk::LOWER_BOUND {
            return self.check_adjacent_chunk(
                grid,
                [0, 0, -Chunk::WIDTH],
                [x, y, Chunk::UPPER_BOUND],
            );
        }
        if z > Chunk::UPPER_BOUND {
            return self.check_adjacent_chunk(
                grid,
                [0, 0, Chunk::WIDTH],
                [x, y, Chunk::LOWER_BOUND],
            );
        }

        self.data[x.to_usize()][y.to_usize()][z.to_usize()]
    }

    fn adjacent_is_air(&self, [x, y, z]: [isize; 3], grid: &Arc<ChunkGrid>) -> bool {
        !self.adjacent_is_solid([x, y, z], grid)
    }
}

#[derive(Component)]
struct GenerateChunk(Task<Chunk>);

#[derive(Component)]
struct ComputeMesh(Task<(Chunk, Mesh)>);

fn generate_chunks(
    mut commands: Commands,
    player: Query<&Transform, With<CameraController>>,
    grid: Res<Arc<ChunkGrid>>,
    generator: Res<Arc<RwLock<dyn ChunkGenerator>>>,
    settings: Res<Settings>,
) {
    if settings.update_chunks {
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
                let chunk_coordinates = player_grid_coordinates
                    + [x * Chunk::WIDTH as isize, 0, z * Chunk::WIDTH as isize];

                if !grid.contains_key(&chunk_coordinates) {
                    let generator = Arc::clone(&generator);

                    let task = task_pool.spawn(async move {
                        generator.read().unwrap().generate(chunk_coordinates.into())
                    });

                    commands
                        .spawn()
                        .insert(GenerateChunk(task))
                        .insert(chunk_coordinates);

                    grid.insert(chunk_coordinates, None);
                }
            }
        }
    }
}

fn compute_meshes(
    mut commands: Commands,
    mut generation_tasks: Query<(Entity, &mut GenerateChunk)>,
    settings: Res<Settings>,
    grid: Res<Arc<ChunkGrid>>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    for (entity, mut generation_task) in &mut generation_tasks {
        if let Some(chunk) = future::block_on(future::poll_once(&mut generation_task.0)) {
            let mesh_builder_settings = settings.mesh_builder.clone();
            let grid = Arc::clone(&grid);
            let task = task_pool.spawn(async move {
                let mesh = chunk.compute_mesh(mesh_builder_settings, grid);

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
    grid: Res<Arc<ChunkGrid>>,
    mut mesh_computation_tasks: Query<(Entity, &mut ComputeMesh, &GridCoordinates)>,
) {
    for (entity, mut mesh_computation_task, coordinates) in
        mesh_computation_tasks.iter_mut().take(20)
    {
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
    grid: Res<Arc<ChunkGrid>>,
    settings: Res<Settings>,
) {
    if settings.update_chunks {
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
}

#[derive(Component)]
pub struct DespawnChunk;

fn despawn_chunks(
    mut commands: Commands,
    chunks: Query<
        Entity,
        (
            With<DespawnChunk>,
            Without<GenerateChunk>,
            Without<ComputeMesh>,
        ),
    >,
) {
    for entity in chunks.iter().take(20) {
        commands.entity(entity).despawn();
    }
}
