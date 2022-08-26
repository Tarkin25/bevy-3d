use bracket_noise::prelude::*;

use super::Chunk;

pub struct ChunkGenerator {
    noise: FastNoise,
}

impl Default for ChunkGenerator {
    fn default() -> Self {
        let mut noise = FastNoise::seeded(0);
        noise.set_noise_type(NoiseType::PerlinFractal);
        noise.set_fractal_type(FractalType::FBM);
        noise.set_fractal_octaves(5);
        noise.set_fractal_lacunarity(2.0);
        noise.set_fractal_gain(0.5);
        noise.set_frequency(2.0);

        Self { noise }
    }
}

impl ChunkGenerator {
    const SCALE: f32 = 0.001;

    pub fn generate(&self, position: [isize; 3]) -> Chunk {
        let mut chunk = Chunk::empty();

        for x in 0..Chunk::WIDTH {
            for z in 0..Chunk::WIDTH {
                let x_coord = (x as isize + position[0]) as f32 * Self::SCALE;
                let z_coord = (z as isize + position[2]) as f32 * Self::SCALE;

                let y = self.noise.get_noise(x_coord, z_coord) * 100.0;
                let y = y.abs().round() as usize;

                for y in 0..y {
                    chunk.set(x, y, z, true);
                }
            }
        }

        //std::thread::sleep(std::time::Duration::from_millis(10));

        chunk
    }
}
