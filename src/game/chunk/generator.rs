use bracket_noise::prelude::*;

use crate::settings::NoiseSettings;

use super::Chunk;

pub struct ChunkGenerator {
    noise: FastNoise,
    amplitude: f32,
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

        Self { noise, amplitude: 100.0 }
    }
}

impl ChunkGenerator {
    const SCALE: f32 = 0.001;

    pub fn update_settings(&mut self, settings: NoiseSettings) {
        self.noise.set_fractal_octaves(settings.octaves);
        self.noise.set_fractal_lacunarity(settings.lacunarity);
        self.noise.set_fractal_gain(settings.gain);
        self.noise.set_frequency(settings.frequency);
        self.amplitude = settings.amplitude;
    }

    pub fn generate(&self, position: [isize; 3]) -> Chunk {
        let mut chunk = Chunk::empty();

        for x in 0..Chunk::WIDTH {
            for z in 0..Chunk::WIDTH {
                let x_coord = (x as isize + position[0]) as f32 * Self::SCALE;
                let z_coord = (z as isize + position[2]) as f32 * Self::SCALE;

                let y = self.noise.get_noise(x_coord, z_coord) * self.amplitude;
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
