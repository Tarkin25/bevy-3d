use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use super::vec3;
use super::VOXEL_SIZE;

#[derive(Default, Debug)]
pub struct MeshBuilder {
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
    vertex_count: u32,
    normals: Vec<Vec3>,
    position: Vec3,
}

impl MeshBuilder {
    pub fn build(self) -> Mesh {
        let vertices: Vec<_> = self
            .vertices
            .into_iter()
            .map(|Vec3 { x, y, z }| [x, y, z])
            .collect();
        let normals: Vec<_> = self
            .normals
            .into_iter()
            .map(|Vec3 { x, y, z }| [x, y, z])
            .collect();

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_indices(Some(Indices::U32(self.indices)));

        mesh
    }

    pub fn move_to(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn face_top(&mut self) {
        self.vertices.extend([
            vec3!(0, 1, 0),
            vec3!(0, 1, 1),
            vec3!(1, 1, 1),
            vec3!(1, 1, 0),
        ].map(|v| v * VOXEL_SIZE + self.position));
        self.indices.extend([
            0,
            1,
            2,
            2,
            3,
            0,
        ].map(|i| i + self.vertex_count));
        self.normals.extend([vec3!(0, 1, 0); 4]);
        self.vertex_count += 4;
    }

    pub fn face_bottom(&mut self) {
        self.vertices.extend([
            vec3!(0, 0, 0),
            vec3!(1, 0, 0),
            vec3!(1, 0, 1),
            vec3!(0, 0, 1),
        ].map(|v| v * VOXEL_SIZE + self.position));
        self.indices.extend([
            0,
            1,
            2,
            2,
            3,
            0,
        ].map(|i| i + self.vertex_count));
        self.normals.extend([vec3!(0, -1, 0); 4]);
        self.vertex_count += 4;
    }

    pub fn face_front(&mut self) {
        self.vertices.extend([
            vec3!(0, 0, 0),
            vec3!(0, 1, 0),
            vec3!(1, 1, 0),
            vec3!(1, 0, 0),
        ].map(|v| v * VOXEL_SIZE + self.position));
        self.indices.extend([
            0,
            1,
            2,
            2,
            3,
            0,
        ].map(|i| i + self.vertex_count));
        self.normals.extend([vec3!(0, 0, -1); 4]);
        self.vertex_count += 4;
    }

    pub fn face_back(&mut self) {
        self.vertices.extend([
            vec3!(0, 0, 1),
            vec3!(1, 0, 1),
            vec3!(1, 1, 1),
            vec3!(0, 1, 1),
        ].map(|v| v * VOXEL_SIZE + self.position));
        self.indices.extend([
            0,
            1,
            2,
            2,
            3,
            0,
        ].map(|i| i + self.vertex_count));
        self.normals.extend([vec3!(0, 0, 1); 4]);
        self.vertex_count += 4;
    }

    pub fn face_right(&mut self) {
        self.vertices.extend([
            vec3!(0, 0, 0),
            vec3!(0, 0, 1),
            vec3!(0, 1, 1),
            vec3!(0, 1, 0),
        ].map(|v| v * VOXEL_SIZE + self.position));
        self.indices.extend([
            0,
            1,
            2,
            2,
            3,
            0,
        ].map(|i| i + self.vertex_count));
        self.normals.extend([vec3!(-1, 0, 0); 4]);
        self.vertex_count += 4;
    }

    pub fn face_left(&mut self) {
        self.vertices.extend([
            vec3!(1, 0, 0),
            vec3!(1, 1, 0),
            vec3!(1, 1, 1),
            vec3!(1, 0, 1),
        ].map(|v| v * VOXEL_SIZE + self.position));
        self.indices.extend([
            0,
            1,
            2,
            2,
            3,
            0,
        ].map(|i| i + self.vertex_count));
        self.normals.extend([vec3!(1, 0, 0); 4]);
        self.vertex_count += 4;
    }
}
