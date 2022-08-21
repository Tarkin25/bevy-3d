use crate::{Chunk, CHUNK_SIZE};

pub struct ChunkGenerator;

impl ChunkGenerator {
    pub fn generate(&self, _position: [usize; 3]) -> Chunk {
        let mut chunk = Chunk::empty();
        
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.set(x, 0, z, true);
            }
        }

        chunk
    }
}