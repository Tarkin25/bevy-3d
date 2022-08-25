use std::ops::{Add, Sub};

use bevy::{prelude::*, utils::HashMap};

use crate::{CHUNK_SIZE, Chunk};

#[derive(Debug, Default, Deref, DerefMut)]
pub struct ChunkGrid(HashMap<GridCoordinates, Chunk>);

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
        self.x -= self.x % CHUNK_SIZE as isize;
        self.y -= self.y % CHUNK_SIZE as isize;
        self.z -= self.z % CHUNK_SIZE as isize;

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
        Self { x: g.x as f32, y: g.y as f32, z: g.z as f32 }
    }
}

impl From<GridCoordinates> for [isize; 3] {
    fn from(GridCoordinates { x, y, z }: GridCoordinates) -> Self {
        [x, y, z]
    }
}

impl From<GridCoordinates> for [usize; 3] {
    fn from(GridCoordinates { x, y, z }: GridCoordinates) -> Self {
        [x as usize, y as usize, z as usize]
    }
}