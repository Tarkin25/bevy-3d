use noise::{utils::{NoiseMap, PlaneMapBuilder, NoiseMapBuilder}, Perlin};

use super::Chunk;

pub struct ChunkGenerator {
    noise_map: NoiseMap,
}

impl Default for ChunkGenerator {
    fn default() -> Self {
        let perlin = Perlin::new();
        let noise_map = PlaneMapBuilder::new(&perlin)
            .set_size(1024, 1024)
            .set_x_bounds(0., 1.0)
            .set_y_bounds(0., 1.0)
            .build();

        Self { noise_map }
    }
}

impl ChunkGenerator {
    pub fn generate(&self, position: [usize; 3]) -> Chunk {
        let mut chunk = Chunk::empty();

        for x in 0..Chunk::WIDTH {
            for z in 0..Chunk::WIDTH {
                let y = self.noise_map.get_value(x + position[0], z + position[2]) * 30.0;
                let y = y.round() as usize;

                for y in 0..y {
                    chunk.set(x, y, z, true);
                }
            }
        }

        chunk
    }
}