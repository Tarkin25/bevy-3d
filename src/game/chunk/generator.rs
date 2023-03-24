use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use bevy::prelude::Resource;
use noise::{NoiseFn, OpenSimplex, RidgedMulti, ScaleBias};

use crate::utils::ToUsize;

use super::{BlockType, Chunk};

#[derive(Resource, Clone)]
pub struct ChunkGenerator {
    terrain: Arc<dyn NoiseFn<f64, 2> + Send + Sync>,
    scale: Arc<AtomicU32>,
}

impl ChunkGenerator {
    pub fn generate_chunk(&self, position: [isize; 3]) -> Chunk {
        let mut chunk = Chunk::new(position.into());

        for x in 0..Chunk::WIDTH.to_usize() {
            for z in 0..Chunk::WIDTH.to_usize() {
                let scale = self.scale() as f32;
                let x_coord = (x as isize + position[0]) as f32 / scale;
                let z_coord = (z as isize + position[2]) as f32 / scale;

                let y = self.terrain.get([x_coord as f64, z_coord as f64]) as f32 * scale / 2.0;
                let y = y.abs().round() as usize;

                for y in 0..y {
                    chunk.set(x, y, z, Some(BlockType::Stone));
                }
                chunk.set(x, y, z, Some(BlockType::Grass));
            }
        }

        chunk
    }

    pub fn scale(&self) -> u32 {
        self.scale.load(Ordering::Acquire)
    }

    pub fn set_scale(&self, scale: u32) {
        self.scale.store(scale, Ordering::Release);
    }
}

impl Default for ChunkGenerator {
    fn default() -> Self {
        let mut noise = RidgedMulti::<OpenSimplex>::default();
        noise.octaves = 4;
        noise.frequency = 0.5;

        let noise = ScaleBias::new(noise).set_bias(1.0);

        Self {
            terrain: Arc::new(noise),
            scale: Arc::new(AtomicU32::new(100)),
        }
    }
}
