use bracket_noise::prelude::*;
use splines::{Spline, Key, Interpolation};
use tap::Pipe;

use crate::{settings::NoiseSettings, utils::ToUsize};

use super::{Chunk, BlockType};

pub trait ChunkGenerator: Send + Sync {
    fn generate(&self, position: [isize; 3]) -> Chunk;

    fn apply_noise_settings(&mut self, _settings: NoiseSettings) {}
}

pub struct PerlinNoiseGenerator {
    noise: FastNoise,
    amplitude: f32,
    scale: f32,
}

impl Default for PerlinNoiseGenerator {
    fn default() -> Self {
        let mut noise = FastNoise::seeded(0);
        noise.set_noise_type(NoiseType::PerlinFractal);
        noise.set_fractal_type(FractalType::FBM);
        noise.set_fractal_octaves(5);
        noise.set_fractal_lacunarity(2.0);
        noise.set_fractal_gain(0.5);
        noise.set_frequency(2.0);

        Self { noise, amplitude: 100.0, scale: 0.01 }
    }
}

impl ChunkGenerator for PerlinNoiseGenerator {
    fn generate(&self, position: [isize; 3]) -> Chunk {
        let mut chunk = Chunk::new(position.into());

        for x in 0..Chunk::WIDTH.to_usize() {
            for z in 0..Chunk::WIDTH.to_usize() {
                let x_coord = (x as isize + position[0]) as f32 * self.scale;
                let z_coord = (z as isize + position[2]) as f32 * self.scale;

                let y = self.noise.get_noise(x_coord, z_coord) * self.amplitude;
                let y = y.abs().round() as usize;

                for y in 0..y {
                    chunk.set(x, y, z, Some(BlockType::Grass));
                }
            }
        }

        //std::thread::sleep(std::time::Duration::from_millis(10));

        chunk
    }

    fn apply_noise_settings(&mut self, settings: NoiseSettings) {
        self.noise.set_fractal_octaves(settings.octaves);
        self.noise.set_fractal_lacunarity(settings.lacunarity);
        self.noise.set_fractal_gain(settings.gain);
        self.noise.set_frequency(settings.frequency);
        self.amplitude = settings.amplitude;
        self.scale = settings.scale;
    }
}

pub struct ContinentalGenerator {
    continental_noise: FastNoise,
    noise_scale: f32,
    base_height: usize,
    spline: Spline<f32, f32>,
}

impl ContinentalGenerator {
    pub fn new(base_height: usize, spline: impl IntoIterator<Item = (f32, f32)>) -> Self {
        let mut continental_noise = FastNoise::seeded(0);
        continental_noise.set_noise_type(NoiseType::PerlinFractal);
        continental_noise.set_fractal_type(FractalType::FBM);

        Self {
            continental_noise,
            noise_scale: 0.01,
            base_height,
            spline: Spline::from_iter(spline.into_iter().map(|(key, value)| Key::new(key, value, Interpolation::Linear)))
        }
    }
}

impl ChunkGenerator for ContinentalGenerator {
    fn generate(&self, position: [isize; 3]) -> Chunk {
        let mut chunk = Chunk::new(position.into());

        for x in 0..Chunk::WIDTH.to_usize() {
            for z in 0..Chunk::WIDTH.to_usize() {
                let x_coord = (x as isize + position[0]) as f32 * self.noise_scale;
                let z_coord = (z as isize + position[2]) as f32 * self.noise_scale;
                let continentality = self.continental_noise.get_noise(x_coord, z_coord);
                let splined_height = self.spline
                .sample(continentality)
                .unwrap()
                .pipe(|s| s.round() as isize);
                let height = (self.base_height as isize + splined_height) as usize;

                for y in 0..height-1 {
                    chunk.set(x, y, z, Some(BlockType::Stone));
                }
                chunk.set(x, height-1, z, Some(BlockType::Grass));
            }
        }

        chunk
    }

    fn apply_noise_settings(&mut self, settings: NoiseSettings) {
        self.continental_noise.set_fractal_octaves(settings.octaves);
        self.continental_noise.set_fractal_lacunarity(settings.lacunarity);
        self.continental_noise.set_fractal_gain(settings.gain);
        self.continental_noise.set_frequency(settings.frequency);
        self.noise_scale = settings.scale;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spline_works() {
        let spline = Spline::from_iter([Key::new(0.0_f32, 0.0_f32, Interpolation::Linear)].into_iter());
        assert_eq!(spline.sample(0.0), Some(0.0));
    }
}
