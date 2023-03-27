use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use super::{
    generator::ChunkGenerator,
    grid::{ChunkGrid, GridCoordinates},
    mesh_builder::MeshBuilderSettings,
    Chunk, GeneratedChunkData,
};

pub struct ChunkDataGenerationFuture {
    coordinates: GridCoordinates,
    generator: ChunkGenerator,
    grid: ChunkGrid,
    mesh_builder_settings: MeshBuilderSettings,
    state: Option<State>,
}

enum State {
    Initial,
    GeneratedChunk(Chunk),
    ComputedMesh(Mesh),
    Done(Mesh, Collider),
}

impl ChunkDataGenerationFuture {
    pub fn new(
        coordinates: GridCoordinates,
        generator: ChunkGenerator,
        grid: ChunkGrid,
        mesh_builder_settings: MeshBuilderSettings,
    ) -> Self {
        Self {
            coordinates,
            generator,
            grid,
            mesh_builder_settings,
            state: Some(State::Initial),
        }
    }
}

impl Future for ChunkDataGenerationFuture {
    type Output = GeneratedChunkData;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use State::*;

        let new_state = match self
            .state
            .take()
            .expect("future should not be polled with empty state")
        {
            Initial => {
                let chunk = self.generator.generate_chunk(self.coordinates.into());
                GeneratedChunk(chunk)
            }
            GeneratedChunk(chunk) => {
                let mesh =
                    self.grid
                        .compute_mesh(self.coordinates, chunk, self.mesh_builder_settings);
                ComputedMesh(mesh)
            }
            ComputedMesh(mesh) => {
                let collider =
                    Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).unwrap();
                Done(mesh, collider)
            }
            Done(mesh, collider) => {
                return Poll::Ready(GeneratedChunkData { mesh, collider });
            }
        };

        self.state = Some(new_state);
        cx.waker().wake_by_ref();

        Poll::Pending
    }
}
