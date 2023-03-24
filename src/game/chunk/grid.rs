use std::{
    ops::{Add, Deref, DerefMut, Sub},
    sync::Arc,
};

use bevy::prelude::*;
use dashmap::DashMap;

use crate::vec3;

use super::{
    mesh_builder::{MeshBuilder, MeshBuilderSettings},
    Chunk,
};

#[derive(Resource, Clone, Deref, Default)]
pub struct ChunkGrid(Arc<ChunkGridInner>);

#[derive(Default)]
pub struct ChunkGridInner {
    chunks: DashMap<GridCoordinates, Option<Chunk>>,
}

impl ChunkGridInner {
    pub fn compute_mesh(
        &self,
        coordinates: GridCoordinates,
        chunk: Chunk,
        mesh_builder_settings: MeshBuilderSettings,
    ) -> Mesh {
        let mut builder = MeshBuilder::new(mesh_builder_settings);

        for x in 0..Chunk::WIDTH {
            for y in 0..Chunk::HEIGHT {
                for z in 0..Chunk::WIDTH {
                    builder.move_to(vec3!(x, y, z));
                    builder.set_block_type(chunk.get([x, y, z]));

                    if chunk.is_solid([x, y, z]) {
                        if y == Chunk::HEIGHT - 1 || chunk.is_air([x, y + 1, z]) {
                            builder.face_top();
                        }
                        if y == Chunk::LOWER_BOUND || chunk.is_air([x, y - 1, z]) {
                            builder.face_bottom();
                        }
                        if x == Chunk::UPPER_BOUND || chunk.is_air([x + 1, y, z]) {
                            builder.face_left();
                        }
                        if x == Chunk::LOWER_BOUND || chunk.is_air([x - 1, y, z]) {
                            builder.face_right();
                        }
                        if z == Chunk::UPPER_BOUND || chunk.is_air([x, y, z + 1]) {
                            builder.face_back();
                        }
                        if z == Chunk::LOWER_BOUND || chunk.is_air([x, y, z - 1]) {
                            builder.face_front();
                        }
                    }
                }
            }
        }

        self.insert(coordinates, Some(chunk));

        builder.build()
    }
}

impl Deref for ChunkGridInner {
    type Target = DashMap<GridCoordinates, Option<Chunk>>;

    fn deref(&self) -> &Self::Target {
        &self.chunks
    }
}

impl DerefMut for ChunkGridInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chunks
    }
}

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCoordinates {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl GridCoordinates {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        Self { x, y, z }
    }

    pub fn to_grid(mut self) -> Self {
        self.x -= self.x % Chunk::WIDTH as isize;
        self.y -= self.y % Chunk::WIDTH as isize;
        self.z -= self.z % Chunk::WIDTH as isize;

        self
    }

    pub fn length(self) -> f32 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32).length()
    }
}

impl Add for GridCoordinates {
    type Output = Self;

    fn add(mut self, GridCoordinates { x, y, z }: Self) -> Self::Output {
        self.x += x;
        self.y += y;
        self.z += z;

        self
    }
}

impl Sub for GridCoordinates {
    type Output = Self;

    fn sub(mut self, GridCoordinates { x, y, z }: Self) -> Self::Output {
        self.x -= x;
        self.y -= y;
        self.z -= z;

        self
    }
}

impl Add<[isize; 3]> for GridCoordinates {
    type Output = GridCoordinates;

    fn add(mut self, [x, y, z]: [isize; 3]) -> Self::Output {
        self.x += x;
        self.y += y;
        self.z += z;

        self
    }
}

impl From<GridCoordinates> for Vec3 {
    fn from(g: GridCoordinates) -> Self {
        Self {
            x: g.x as f32,
            y: g.y as f32,
            z: g.z as f32,
        }
    }
}

impl From<GridCoordinates> for [isize; 3] {
    fn from(GridCoordinates { x, y, z }: GridCoordinates) -> Self {
        [x, y, z]
    }
}

impl From<[isize; 3]> for GridCoordinates {
    fn from([x, y, z]: [isize; 3]) -> Self {
        Self { x, y, z }
    }
}

impl From<GridCoordinates> for [usize; 3] {
    fn from(GridCoordinates { x, y, z }: GridCoordinates) -> Self {
        [x as usize, y as usize, z as usize]
    }
}
