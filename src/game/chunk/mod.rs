use std::sync::Arc;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    tasks::{AsyncComputeTaskPool, Task},
};

use bevy_rapier3d::prelude::*;

use futures_lite::future;

use crate::{settings::Settings, utils::ToUsize, vec3, AppState, VoxelConfig};

use self::{
    chunk_data_generation_future::ChunkDataGenerationFuture,
    generator::ChunkGenerator,
    grid::{ChunkGrid, ChunkGridInner, GridCoordinates},
    mesh_builder::{MeshBuilder, MeshBuilderSettings},
};

use super::camera_controller::CameraController;

mod chunk_data_generation_future;
pub mod generator;
pub mod grid;
pub mod mesh_builder;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkGrid>()
            .init_resource::<ChunkGenerator>()
            .init_asset::<GeneratedChunkData>()
            .add_systems(Startup, setup_voxel_material)
            .add_systems(
                Update,
                (
                    Chunk::trigger_generation,
                    Chunk::poll_generation_tasks,
                    Chunk::insert_meshes_and_colliders,
                    unload_chunks,
                    despawn_chunks,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(Resource)]
struct VoxelMaterial {
    handle: Handle<StandardMaterial>,
}

fn setup_voxel_material(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.insert_resource(VoxelMaterial {
        handle: materials.add(StandardMaterial {
            double_sided: false,
            base_color: Color::PURPLE,
            ..Default::default()
        }),
    });
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BlockType {
    Grass,
    Stone,
}

impl BlockType {
    pub fn texture_indices(self) -> TextureIndices {
        use BlockType::*;

        match self {
            Grass => TextureIndices {
                pos_x: 1,
                neg_x: 1,
                pos_y: 0,
                neg_y: 2,
                pos_z: 1,
                neg_z: 1,
            },
            Stone => TextureIndices {
                pos_x: 3,
                neg_x: 3,
                pos_y: 3,
                neg_y: 3,
                pos_z: 3,
                neg_z: 3,
            },
        }
    }
}

pub struct TextureIndices {
    pub pos_x: u32,
    pub neg_x: u32,
    pub pos_y: u32,
    pub neg_y: u32,
    pub pos_z: u32,
    pub neg_z: u32,
}

impl TextureIndices {
    pub fn index_by_normal(self, normal: Vec3) -> u32 {
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

#[derive(Component, TypeUuid, TypePath, Asset)]
#[uuid = "d4d4e3e8-a3ea-4d73-95ed-95ed85bf85e5"]
pub struct GeneratedChunkData {
    pub mesh: Mesh,
    pub collider: Collider,
}

#[derive(Component)]
pub struct Chunk {
    data: Vec<Vec<Vec<Option<BlockType>>>>,
    coordinates: GridCoordinates,
}

impl Chunk {
    pub const WIDTH: isize = 32;
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
        grid: &Arc<ChunkGridInner>,
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
    pub fn compute_mesh(&self, settings: MeshBuilderSettings, grid: Arc<ChunkGridInner>) -> Mesh {
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

    fn adjacent_is_solid(&self, [x, y, z]: [isize; 3], grid: &Arc<ChunkGridInner>) -> bool {
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

    fn adjacent_is_air(&self, [x, y, z]: [isize; 3], grid: &Arc<ChunkGridInner>) -> bool {
        !self.adjacent_is_solid([x, y, z], grid)
    }
}

impl Chunk {
    fn trigger_generation(
        mut commands: Commands,
        player: Query<&Transform, With<CameraController>>,
        grid: Res<ChunkGrid>,
        generator: Res<ChunkGenerator>,
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
                        let generator = generator.clone();
                        let grid = grid.clone();

                        let task = task_pool.spawn(ChunkDataGenerationFuture::new(
                            chunk_coordinates,
                            generator,
                            grid.clone(),
                            settings.mesh_builder,
                        ));

                        commands.spawn((chunk_coordinates, GenerateChunk(task)));

                        grid.insert(chunk_coordinates, None);
                    }
                }
            }
        }
    }

    fn poll_generation_tasks(
        mut commands: Commands,
        mut generation_tasks: Query<(Entity, &mut GenerateChunk)>,
        settings: Res<Settings>,
        mut chunk_data_assets: ResMut<Assets<GeneratedChunkData>>,
    ) {
        for (entity, mut task) in generation_tasks
            .iter_mut()
            .take(settings.task_polls_per_frame)
        {
            let data = future::block_on(future::poll_once(&mut task.0));
            //info!("polled task");
            if let Some(data) = data {
                let handle = chunk_data_assets.add(data);

                commands
                    .entity(entity)
                    .insert(handle)
                    .remove::<GenerateChunk>();
            }
        }
    }

    fn insert_meshes_and_colliders(
        mut commands: Commands,
        query: Query<(Entity, &Handle<GeneratedChunkData>, &GridCoordinates)>,
        mut chunk_data_assets: ResMut<Assets<GeneratedChunkData>>,
        mut meshes: ResMut<Assets<Mesh>>,
        config: Res<VoxelConfig>,
        settings: Res<Settings>,
        voxel_material: Res<VoxelMaterial>,
    ) {
        for (entity, handle, coordinates) in query.iter().take(settings.mesh_updates_per_frame) {
            let GeneratedChunkData { mesh, collider } = chunk_data_assets.remove(handle).unwrap();

            commands
                .entity(entity)
                .remove::<Handle<GeneratedChunkData>>()
                .insert((
                    MaterialMeshBundle {
                        mesh: meshes.add(mesh),
                        //material: config.material.clone(),
                        material: voxel_material.handle.clone(),
                        transform: Transform::from_translation((*coordinates).into()),
                        ..Default::default()
                    },
                    collider,
                    RigidBody::Fixed,
                ));
        }
    }
}

#[derive(Component)]
struct GenerateChunk(Task<GeneratedChunkData>);

fn unload_chunks(
    mut commands: Commands,
    mut chunks: Query<(Entity, &GridCoordinates)>,
    player: Query<&Transform, With<CameraController>>,
    grid: Res<ChunkGrid>,
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
    chunks: Query<Entity, (With<DespawnChunk>, Without<GenerateChunk>)>,
) {
    for entity in chunks.iter().take(20) {
        commands.entity(entity).despawn();
    }
}
