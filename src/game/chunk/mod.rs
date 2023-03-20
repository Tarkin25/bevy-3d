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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BlockType {
    Grass,
    Stone,
}

impl BlockType {
    pub fn uv_bounds(self) -> UvBounds {
        use BlockType::*;

        match self {
            Grass => UvBounds::from_index(2, 0),
            Stone => UvBounds::from_index(0, 0),
        }
    }

    pub fn texture_uvs(self) -> TextureUvs {
        use BlockType::*;

        match self {
            Grass => TextureUvs {
                pos_x: UvBounds::from_index(3, 0),
                neg_x: UvBounds::from_index(3, 0),
                pos_y: UvBounds::from_index(2, 0),
                neg_y: UvBounds::from_index(18, 1),
                pos_z: UvBounds::from_index(3, 0),
                neg_z: UvBounds::from_index(3, 0),
            },
            Stone => TextureUvs {
                pos_x: UvBounds::from_index(19, 0),
                neg_x: UvBounds::from_index(19, 0),
                pos_y: UvBounds::from_index(19, 0),
                neg_y: UvBounds::from_index(19, 0),
                pos_z: UvBounds::from_index(19, 0),
                neg_z: UvBounds::from_index(19, 0),
            },
        }
    }
}

#[derive(Debug)]
pub struct TextureUvs {
    pub pos_x: UvBounds,
    pub neg_x: UvBounds,
    pub pos_y: UvBounds,
    pub neg_y: UvBounds,
    pub pos_z: UvBounds,
    pub neg_z: UvBounds,
}

impl TextureUvs {
    pub fn uv_by_normal(self, normal: Vec3) -> UvBounds {
        if normal == Vec3::X {
            self.pos_x
        } else if normal == Vec3::NEG_X {
            self.neg_x
        } else if normal == Vec3::Y {
            self.pos_y
        } else if normal == Vec3::NEG_Y {
            self.neg_y
        } else if normal == Vec3::Z {
            self.pos_z
        } else if normal == Vec3::NEG_Z {
            self.neg_z
        } else {
            panic!("Invalid normal provided")
        }
    }
}

#[derive(Debug)]
pub struct UvBounds {
    pub lower: Vec2,
    pub upper: Vec2,
}

impl UvBounds {
    pub fn from_index(x: u32, y: u32) -> Self {
        const ATLAS_WIDTH: f32 = 512.0;
        const ATLAS_HEIGHT: f32 = 256.0;
        const TEXTURE_SIZE: f32 = 16.0;

        let lower = Vec2::new(
            x as f32 * TEXTURE_SIZE / ATLAS_WIDTH,
            y as f32 * TEXTURE_SIZE / ATLAS_HEIGHT,
        );
        let upper = Vec2::new(
            (x + 1) as f32 * TEXTURE_SIZE / ATLAS_WIDTH,
            (y + 1) as f32 * TEXTURE_SIZE / ATLAS_HEIGHT,
        );

        Self { lower, upper }
    }
}

#[derive(Component)]
pub struct Chunk {
    data: Vec<Vec<Vec<Option<BlockType>>>>,
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
                vec![vec![None; Chunk::WIDTH as usize]; Chunk::HEIGHT as usize];
                Chunk::WIDTH as usize
            ],
            coordinates,
        }
    }

    pub fn is_solid(&self, position: [isize; 3]) -> bool {
        self.get(position).is_some()
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

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: Option<BlockType>) {
        self.data[x][y][z] = value;
    }

    pub fn get(&self, position: [isize; 3]) -> Option<BlockType> {
        let [x, y, z] = position.map(|n| n.to_usize());
        self.data[x][y][z]
    }

    /// http://ilkinulas.github.io/development/unity/2016/04/30/cube-mesh-in-unity3d.html
    pub fn compute_mesh(&self, settings: MeshBuilderSettings, grid: Arc<ChunkGrid>) -> Mesh {
        let mut builder = MeshBuilder::new(settings);

        for x in 0..Chunk::WIDTH {
            for y in 0..Chunk::HEIGHT {
                for z in 0..Chunk::WIDTH {
                    builder.move_to(vec3!(x, y, z));
                    builder.set_block_type(self.get([x, y, z]));

                    if self.is_solid([x, y, z]) {
                        if y == Chunk::HEIGHT - 1 || self.is_air([x, y + 1, z]) {
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

        self.data[x.to_usize()][y.to_usize()][z.to_usize()].is_some()
    }

    fn adjacent_is_air(&self, [x, y, z]: [isize; 3], grid: &Arc<ChunkGrid>) -> bool {
        !self.adjacent_is_solid([x, y, z], grid)
    }
}

#[derive(Component)]
struct GenerateChunk(Task<Chunk>);

#[derive(Component)]
struct ComputeMesh(Task<Mesh>);

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
    mut generation_tasks: Query<(Entity, &mut GenerateChunk, &GridCoordinates)>,
    grid: Res<Arc<ChunkGrid>>,
    settings: Res<Settings>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    let mesh_builder_settings = settings.mesh_builder;

    for (entity, mut generation_task, coordinates) in &mut generation_tasks {
        if let Some(chunk) = future::block_on(future::poll_once(&mut generation_task.0)) {
            let coordinates = *coordinates;
            let grid = Arc::clone(&grid);
            let task = task_pool
                .spawn(async move { grid.compute_mesh(coordinates, chunk, mesh_builder_settings) });

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
    mut mesh_computation_tasks: Query<(Entity, &mut ComputeMesh, &GridCoordinates)>,
) {
    for (entity, mut mesh_computation_task, coordinates) in
        mesh_computation_tasks.iter_mut().take(20)
    {
        if let Some(mesh) = future::block_on(future::poll_once(&mut mesh_computation_task.0)) {
            let mut entity = commands.entity(entity);
            entity.insert_bundle(PbrBundle {
                mesh: meshes.add(mesh),
                material: config.material.clone(),
                transform: Transform::from_translation((*coordinates).into()),
                ..Default::default()
            });
            entity.remove::<ComputeMesh>();
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
